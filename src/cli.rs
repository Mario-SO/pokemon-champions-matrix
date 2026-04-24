use crate::error::PcError;
use crate::matrix;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "pc")]
#[command(about = "Pokémon Champions matchup matrix")]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Open the matchup matrix TUI.
    Matrix {
        #[arg(long, default_value = "examples/my-team.txt")]
        team: PathBuf,
        #[arg(long, default_value = "examples/opponents.txt")]
        opponents: PathBuf,
    },
}

pub(crate) fn run() -> Result<(), PcError> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Matrix { team, opponents } => matrix::run_matrix(&team, &opponents),
    }
}
