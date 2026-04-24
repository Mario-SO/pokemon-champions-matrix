use super::StatName;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Nature {
    Hardy,
    Lonely,
    Brave,
    Adamant,
    Naughty,
    Bold,
    Docile,
    Relaxed,
    Impish,
    Lax,
    Timid,
    Hasty,
    Serious,
    Jolly,
    Naive,
    Modest,
    Mild,
    Quiet,
    Bashful,
    Rash,
    Calm,
    Gentle,
    Sassy,
    Careful,
    Quirky,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NatureMultiplier {
    Boosted,
    Lowered,
    Neutral,
}

impl Nature {
    pub(crate) fn boosted_stat(self) -> Option<StatName> {
        match self {
            Nature::Lonely | Nature::Brave | Nature::Adamant | Nature::Naughty => {
                Some(StatName::Atk)
            }
            Nature::Bold | Nature::Relaxed | Nature::Impish | Nature::Lax => Some(StatName::Def),
            Nature::Timid | Nature::Hasty | Nature::Jolly | Nature::Naive => Some(StatName::Spe),
            Nature::Modest | Nature::Mild | Nature::Quiet | Nature::Rash => Some(StatName::SpA),
            Nature::Calm | Nature::Gentle | Nature::Sassy | Nature::Careful => Some(StatName::SpD),
            Nature::Hardy | Nature::Docile | Nature::Serious | Nature::Bashful | Nature::Quirky => {
                None
            }
        }
    }

    pub(crate) fn lowered_stat(self) -> Option<StatName> {
        match self {
            Nature::Bold | Nature::Timid | Nature::Modest | Nature::Calm => Some(StatName::Atk),
            Nature::Lonely | Nature::Hasty | Nature::Mild | Nature::Gentle => Some(StatName::Def),
            Nature::Adamant | Nature::Impish | Nature::Jolly | Nature::Careful => {
                Some(StatName::SpA)
            }
            Nature::Naughty | Nature::Lax | Nature::Naive | Nature::Rash => Some(StatName::SpD),
            Nature::Brave | Nature::Relaxed | Nature::Quiet | Nature::Sassy => Some(StatName::Spe),
            Nature::Hardy | Nature::Docile | Nature::Serious | Nature::Bashful | Nature::Quirky => {
                None
            }
        }
    }

    pub(crate) fn multiplier(self, stat: StatName) -> NatureMultiplier {
        if self.boosted_stat() == Some(stat) {
            NatureMultiplier::Boosted
        } else if self.lowered_stat() == Some(stat) {
            NatureMultiplier::Lowered
        } else {
            NatureMultiplier::Neutral
        }
    }
}

impl fmt::Display for Nature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for Nature {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "hardy" => Ok(Nature::Hardy),
            "lonely" => Ok(Nature::Lonely),
            "brave" => Ok(Nature::Brave),
            "adamant" => Ok(Nature::Adamant),
            "naughty" => Ok(Nature::Naughty),
            "bold" => Ok(Nature::Bold),
            "docile" => Ok(Nature::Docile),
            "relaxed" => Ok(Nature::Relaxed),
            "impish" => Ok(Nature::Impish),
            "lax" => Ok(Nature::Lax),
            "timid" => Ok(Nature::Timid),
            "hasty" => Ok(Nature::Hasty),
            "serious" => Ok(Nature::Serious),
            "jolly" => Ok(Nature::Jolly),
            "naive" => Ok(Nature::Naive),
            "modest" => Ok(Nature::Modest),
            "mild" => Ok(Nature::Mild),
            "quiet" => Ok(Nature::Quiet),
            "bashful" => Ok(Nature::Bashful),
            "rash" => Ok(Nature::Rash),
            "calm" => Ok(Nature::Calm),
            "gentle" => Ok(Nature::Gentle),
            "sassy" => Ok(Nature::Sassy),
            "careful" => Ok(Nature::Careful),
            "quirky" => Ok(Nature::Quirky),
            _ => Err(format!("unknown nature '{value}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nature_effects_are_standard() {
        assert_eq!(Nature::Modest.boosted_stat(), Some(StatName::SpA));
        assert_eq!(Nature::Modest.lowered_stat(), Some(StatName::Atk));
        assert_eq!(Nature::Quiet.boosted_stat(), Some(StatName::SpA));
        assert_eq!(Nature::Quiet.lowered_stat(), Some(StatName::Spe));
        assert_eq!(Nature::Hardy.boosted_stat(), None);
        assert_eq!(Nature::Hardy.lowered_stat(), None);
    }
}
