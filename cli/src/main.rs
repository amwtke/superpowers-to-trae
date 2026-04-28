//! superpowers-trae: One-shot installer for the Superpowers methodology in Trae IDE projects.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod embed;
mod agents_md;
mod rollback;
mod addons;
mod commands;

#[derive(Parser, Debug)]
#[command(name = "superpowers-trae", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// First-time install of Superpowers methodology into a project
    Init {
        #[arg(short, long, default_value = ".")]
        dir: String,
        #[arg(short, long)]
        force: bool,
        #[arg(long, value_delimiter = ',')]
        addons: Vec<String>,
    },
    /// Refresh an already-installed project to the binary's embedded Superpowers version
    Upgrade {
        #[arg(short, long, default_value = ".")]
        dir: String,
        #[arg(long)]
        no_backup: bool,
        #[arg(long, value_delimiter = ',')]
        addons: Vec<String>,
    },
    /// Check installation status of a project
    Status {
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { dir, force, addons } => {
            commands::init::run(&dir, force, &addons)?;
        }
        Commands::Upgrade { dir, no_backup, addons } => {
            commands::upgrade::run(&dir, no_backup, &addons)?;
        }
        Commands::Status { dir } => {
            commands::status::run(&dir)?;
        }
    }
    Ok(())
}
