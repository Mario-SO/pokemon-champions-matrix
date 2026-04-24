use super::{BaseStats, Nature, StatPoints};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PokemonType {
    Normal,
    Fire,
    Water,
    Grass,
    Electric,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

impl fmt::Display for PokemonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            PokemonType::Normal => "Normal",
            PokemonType::Fire => "Fire",
            PokemonType::Water => "Water",
            PokemonType::Grass => "Grass",
            PokemonType::Electric => "Electric",
            PokemonType::Ice => "Ice",
            PokemonType::Fighting => "Fighting",
            PokemonType::Poison => "Poison",
            PokemonType::Ground => "Ground",
            PokemonType::Flying => "Flying",
            PokemonType::Psychic => "Psychic",
            PokemonType::Bug => "Bug",
            PokemonType::Rock => "Rock",
            PokemonType::Ghost => "Ghost",
            PokemonType::Dragon => "Dragon",
            PokemonType::Dark => "Dark",
            PokemonType::Steel => "Steel",
            PokemonType::Fairy => "Fairy",
        };
        f.write_str(value)
    }
}

impl FromStr for PokemonType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "normal" => Ok(PokemonType::Normal),
            "fire" => Ok(PokemonType::Fire),
            "water" => Ok(PokemonType::Water),
            "grass" => Ok(PokemonType::Grass),
            "electric" => Ok(PokemonType::Electric),
            "ice" => Ok(PokemonType::Ice),
            "fighting" => Ok(PokemonType::Fighting),
            "poison" => Ok(PokemonType::Poison),
            "ground" => Ok(PokemonType::Ground),
            "flying" => Ok(PokemonType::Flying),
            "psychic" => Ok(PokemonType::Psychic),
            "bug" => Ok(PokemonType::Bug),
            "rock" => Ok(PokemonType::Rock),
            "ghost" => Ok(PokemonType::Ghost),
            "dragon" => Ok(PokemonType::Dragon),
            "dark" => Ok(PokemonType::Dark),
            "steel" => Ok(PokemonType::Steel),
            "fairy" => Ok(PokemonType::Fairy),
            _ => Err(format!("unknown type '{value}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BattleSide {
    Player,
    Opponent,
}

impl fmt::Display for BattleSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BattleSide::Player => f.write_str("Player"),
            BattleSide::Opponent => f.write_str("Opponent"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TargetRef {
    pub side: BattleSide,
    pub slot: u8,
}

impl fmt::Display for TargetRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.side, self.slot)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    None,
    Burn,
    Paralysis,
    Poison,
    Toxic,
    Sleep,
    Freeze,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Status::None => "None",
            Status::Burn => "Burn",
            Status::Paralysis => "Paralysis",
            Status::Poison => "Poison",
            Status::Toxic => "Toxic",
            Status::Sleep => "Sleep",
            Status::Freeze => "Freeze",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PokemonSet {
    pub nickname: Option<String>,
    pub species: String,
    pub item: Option<String>,
    pub ability: Option<String>,
    pub tera_type: Option<PokemonType>,
    pub tera_active: bool,
    pub nature: Nature,
    pub level: u8,
    pub stat_points: StatPoints,
    pub hp_percent: Option<u8>,
    pub status: Status,
}

impl PokemonSet {
    pub(crate) fn new(species: String, item: Option<String>) -> Self {
        Self {
            nickname: None,
            species,
            item,
            ability: None,
            tera_type: None,
            tera_active: false,
            nature: Nature::Hardy,
            level: 50,
            stat_points: StatPoints::default(),
            hp_percent: None,
            status: Status::None,
        }
    }

    pub(crate) fn display_name(&self) -> &str {
        self.nickname.as_deref().unwrap_or(&self.species)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PokemonData {
    pub base_stats: BaseStats,
    pub types: Vec<PokemonType>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedPokemon {
    pub reference: TargetRef,
    pub set: PokemonSet,
    pub data: PokemonData,
}
