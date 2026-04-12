#[cfg(not(target_os = "macos"))]
compile_error!("nitora only supports macOS");

mod cli;
mod device;
mod display;
mod gamma;
mod ipc;
mod launchd;
mod overlay;
mod server;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use ipc::Request;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Serve { auto_enable, brightness } => server::run(auto_enable, brightness),
        Command::Enable => print_response(ipc::send_request(&Request::Enable)?),
        Command::Disable => print_response(ipc::send_request(&Request::Disable)?),
        Command::Toggle => print_response(ipc::send_request(&Request::Toggle)?),
        Command::Status => print_response(ipc::send_request(&Request::Status)?),
        Command::Set { value } => print_response(ipc::send_request(&Request::Set { value })?),
        Command::PrintLaunchd { program_path } => {
            let program_path = launchd::resolve_program_path(program_path)?;
            print!(
                "{}",
                launchd::render_plist(&program_path, &ipc::socket_path(), launchd::SERVICE_LABEL)
            );
            Ok(())
        }
        Command::InstallLaunchd { program_path } => {
            let path = launchd::install(program_path)?;
            println!("Installed launchd agent at {}", path.display());
            println!(
                "Load it with: launchctl bootstrap gui/$(id -u) {}",
                path.display()
            );
            Ok(())
        }
        Command::UninstallLaunchd => {
            let path = launchd::uninstall()?;
            println!("Removed launchd agent at {}", path.display());
            println!(
                "If it is loaded, unload it with: launchctl bootout gui/$(id -u) {}",
                path.display()
            );
            Ok(())
        }
    }
}

fn print_response(response: ipc::Response) -> Result<()> {
    println!("{}", response.message);

    if response.show_state {
        println!("Enabled: {}", response.enabled);
        println!("Brightness: {}", response.brightness);
        println!("Backend: {}", response.backend);
    }

    Ok(())
}
