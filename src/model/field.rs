use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Weather {
    None,
    Sun,
    Rain,
    Sand,
    Snow,
}

impl fmt::Display for Weather {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Weather::None => f.write_str("None"),
            Weather::Sun => f.write_str("Sun"),
            Weather::Rain => f.write_str("Rain"),
            Weather::Sand => f.write_str("Sand"),
            Weather::Snow => f.write_str("Snow"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Terrain {
    None,
    Electric,
    Grassy,
    Psychic,
    Misty,
}

impl fmt::Display for Terrain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Terrain::None => f.write_str("None"),
            Terrain::Electric => f.write_str("Electric"),
            Terrain::Grassy => f.write_str("Grassy"),
            Terrain::Psychic => f.write_str("Psychic"),
            Terrain::Misty => f.write_str("Misty"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Room {
    None,
    TrickRoom,
}

impl fmt::Display for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Room::None => f.write_str("None"),
            Room::TrickRoom => f.write_str("Trick Room"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FieldState {
    pub weather: Weather,
    pub terrain: Terrain,
    pub room: Room,
}

impl Default for FieldState {
    fn default() -> Self {
        Self {
            weather: Weather::None,
            terrain: Terrain::None,
            room: Room::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SideState {
    pub tailwind: bool,
    pub reflect: bool,
    pub light_screen: bool,
    pub aurora_veil: bool,
    pub helping_hand: bool,
}
