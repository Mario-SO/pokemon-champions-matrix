use crate::config::pc_config_dir;
use crate::error::PcError;
use crate::matrix;
use clap::{Parser, Subcommand};
use std::fs;
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
    /// Create sample team files in the user config directory.
    Init {
        /// Overwrite existing sample files.
        #[arg(long)]
        force: bool,
    },

    /// Open the matchup matrix TUI.
    Matrix {
        #[arg(long)]
        team: Option<PathBuf>,
        #[arg(long)]
        opponents: Option<PathBuf>,
    },
}

pub(crate) fn run() -> Result<(), PcError> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { force } => init_config(force),
        Commands::Matrix { team, opponents } => {
            let paths = TeamPaths::resolve(team, opponents)?;
            if !paths.team.exists() || !paths.opponents.exists() {
                return Err(PcError::MissingTeamFiles {
                    team_path: paths.team.display().to_string(),
                    opponents_path: paths.opponents.display().to_string(),
                });
            }
            matrix::run_matrix(&paths.team, &paths.opponents)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TeamPaths {
    team: PathBuf,
    opponents: PathBuf,
}

impl TeamPaths {
    fn resolve(team: Option<PathBuf>, opponents: Option<PathBuf>) -> Result<Self, PcError> {
        let config_dir = pc_config_dir()?;
        Ok(Self {
            team: team.unwrap_or_else(|| config_dir.join("my-team.txt")),
            opponents: opponents.unwrap_or_else(|| config_dir.join("opponents.txt")),
        })
    }
}

fn init_config(force: bool) -> Result<(), PcError> {
    let config_dir = pc_config_dir()?;
    fs::create_dir_all(&config_dir).map_err(|source| PcError::Io {
        path: config_dir.display().to_string(),
        source,
    })?;
    write_sample_file(
        &config_dir.join("my-team.txt"),
        include_str!("../examples/my-team.txt"),
        force,
    )?;
    write_sample_file(
        &config_dir.join("opponents.txt"),
        include_str!("../examples/opponents.txt"),
        force,
    )?;
    println!("Created sample files in {}", config_dir.display());
    println!("Run `pc matrix` to open the matchup matrix.");
    Ok(())
}

fn write_sample_file(path: &PathBuf, contents: &str, force: bool) -> Result<(), PcError> {
    if path.exists() && !force {
        return Err(PcError::Validation {
            message: format!(
                "{} already exists; pass --force to overwrite it",
                path.display()
            ),
        });
    }
    fs::write(path, contents).map_err(|source| PcError::Io {
        path: path.display().to_string(),
        source,
    })
}
