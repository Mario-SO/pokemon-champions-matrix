use super::PokemonType;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MoveCategory {
    Physical,
    Special,
    Status,
}

impl fmt::Display for MoveCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveCategory::Physical => f.write_str("Physical"),
            MoveCategory::Special => f.write_str("Special"),
            MoveCategory::Status => f.write_str("Status"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MoveData {
    pub requested_name: String,
    pub move_type: PokemonType,
    pub category: MoveCategory,
    pub power: Option<u16>,
    pub spread: bool,
}
