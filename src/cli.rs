use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "nitora",
    version,
    about = "Rust CLI and launchd-friendly service scaffold for Nitora"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Serve {
        /// Automatically enable XDR brightness when the service starts.
        #[arg(long)]
        auto_enable: bool,

        /// Initial brightness level (0-100).
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100), default_value_t = 100)]
        brightness: u8,
    },
    Enable,
    Disable,
    Toggle,
    Status,
    Set {
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        value: u8,
    },
    PrintLaunchd {
        #[arg(long)]
        program_path: Option<PathBuf>,
    },
    InstallLaunchd {
        #[arg(long)]
        program_path: Option<PathBuf>,
    },
    UninstallLaunchd,
}
