mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum CargoCli {
    Heisenberg(HeisenbergCli),
}

#[derive(clap::Args)]
#[command(author, version, about = "Heisenberg build orchestration", long_about = None)]
struct HeisenbergCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize heisenberg.toml with inferred defaults
    Init,
    /// Build frontend assets then run cargo build
    Build {
        /// Additional arguments to pass to cargo build
        #[arg(last = true)]
        cargo_args: Vec<String>,
    },
    /// Start frontend dev server and run cargo run with TUI
    Run {
        /// Additional arguments to pass to cargo run
        #[arg(last = true)]
        cargo_args: Vec<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let CargoCli::Heisenberg(cli) = CargoCli::parse();

    match cli.command {
        Commands::Init => commands::init::run(),
        Commands::Build { cargo_args } => commands::build::run(cargo_args),
        Commands::Run { cargo_args } => commands::run::run(cargo_args),
    }
}
