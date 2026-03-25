use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Enable,
    Disable,
    Toggle,
    Status,
    Set { value: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub message: String,
    pub enabled: bool,
    pub brightness: u8,
    pub backend: String,
    pub show_state: bool,
}

pub fn socket_path() -> PathBuf {
    std::env::var_os("NITORA_SOCKET")
        .map(PathBuf::from)
        .or_else(|| dirs::runtime_dir().map(|path| path.join("nitora.sock")))
        .unwrap_or_else(|| PathBuf::from("/tmp/nitora.sock"))
}

pub fn create_listener() -> Result<UnixListener> {
    let socket_path = socket_path();

    if socket_path.exists() {
        fs::remove_file(&socket_path)
            .with_context(|| format!("failed removing stale socket {}", socket_path.display()))?;
    }

    UnixListener::bind(&socket_path)
        .with_context(|| format!("failed binding socket {}", socket_path.display()))
}

pub fn send_request(request: &Request) -> Result<Response> {
    let socket_path = socket_path();
    let mut stream = UnixStream::connect(&socket_path).with_context(|| {
        format!(
            "failed connecting to service socket {}. Is `nitora serve` running?",
            socket_path.display()
        )
    })?;

    write_json_line(&mut stream, request)?;
    read_json_line(stream)
}

pub fn write_json_line<T: Serialize>(stream: &mut UnixStream, value: &T) -> Result<()> {
    let mut encoded = serde_json::to_vec(value)?;
    encoded.push(b'\n');
    stream.write_all(&encoded)?;
    stream.flush()?;
    Ok(())
}

pub fn read_json_line<T: for<'de> Deserialize<'de>>(stream: UnixStream) -> Result<T> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let value = serde_json::from_str::<T>(&line)?;
    Ok(value)
}
