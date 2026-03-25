use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

pub const SERVICE_LABEL: &str = "dev.nitora.daemon";

pub fn resolve_program_path(program_path: Option<PathBuf>) -> Result<PathBuf> {
    match program_path {
        Some(path) => Ok(path),
        None => std::env::current_exe().context("failed to determine current executable path"),
    }
}

pub fn render_plist(program_path: &Path, socket_path: &Path, label: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{program}</string>
    <string>serve</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>NITORA_SOCKET</key>
    <string>{socket}</string>
  </dict>
  <key>KeepAlive</key>
  <true/>
  <key>RunAtLoad</key>
  <true/>
  <key>ProcessType</key>
  <string>Interactive</string>
  <key>StandardOutPath</key>
  <string>/tmp/nitora.stdout.log</string>
  <key>StandardErrorPath</key>
  <string>/tmp/nitora.stderr.log</string>
</dict>
</plist>
"#,
        label = xml_escape(label),
        program = xml_escape(&program_path.display().to_string()),
        socket = xml_escape(&socket_path.display().to_string())
    )
}

pub fn install(program_path: Option<PathBuf>) -> Result<PathBuf> {
    let program_path = resolve_program_path(program_path)?;
    let path = agent_plist_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating {}", parent.display()))?;
    }

    fs::write(
        &path,
        render_plist(&program_path, &crate::ipc::socket_path(), SERVICE_LABEL),
    )
    .with_context(|| format!("failed writing {}", path.display()))?;

    Ok(path)
}

pub fn uninstall() -> Result<PathBuf> {
    let path = agent_plist_path()?;

    if !path.exists() {
        bail!("launchd plist not found at {}", path.display());
    }

    fs::remove_file(&path).with_context(|| format!("failed removing {}", path.display()))?;
    Ok(path)
}

fn agent_plist_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("failed to resolve home directory")?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{SERVICE_LABEL}.plist")))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
