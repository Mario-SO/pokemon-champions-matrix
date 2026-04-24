mod engine;
mod showdown;
mod tui;

use crate::error::PcError;
use std::path::Path;

pub(crate) fn run_matrix(team_path: &Path, opponents_path: &Path) -> Result<(), PcError> {
    tui::run(team_path, opponents_path)
}
