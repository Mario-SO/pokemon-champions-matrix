use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StatName {
    Hp,
    Atk,
    Def,
    SpA,
    SpD,
    Spe,
}

impl StatName {
    pub(crate) const ALL: [StatName; 6] = [
        StatName::Hp,
        StatName::Atk,
        StatName::Def,
        StatName::SpA,
        StatName::SpD,
        StatName::Spe,
    ];
}

impl fmt::Display for StatName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            StatName::Hp => "HP",
            StatName::Atk => "Atk",
            StatName::Def => "Def",
            StatName::SpA => "SpA",
            StatName::SpD => "SpD",
            StatName::Spe => "Spe",
        };
        f.write_str(value)
    }
}

impl FromStr for StatName {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "hp" => Ok(StatName::Hp),
            "atk" | "attack" => Ok(StatName::Atk),
            "def" | "defense" => Ok(StatName::Def),
            "spa" | "sp.atk" | "sp atk" | "special attack" | "special-attack" => Ok(StatName::SpA),
            "spd" | "sp.def" | "sp def" | "special defense" | "special-defense" => {
                Ok(StatName::SpD)
            }
            "spe" | "speed" => Ok(StatName::Spe),
            _ => Err(format!("unknown stat '{value}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BaseStats {
    pub hp: u16,
    pub atk: u16,
    pub def: u16,
    pub spa: u16,
    pub spd: u16,
    pub spe: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stats {
    pub hp: u16,
    pub atk: u16,
    pub def: u16,
    pub spa: u16,
    pub spd: u16,
    pub spe: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct StatPoints {
    pub hp: u8,
    pub atk: u8,
    pub def: u8,
    pub spa: u8,
    pub spd: u8,
    pub spe: u8,
}

impl StatPoints {
    pub(crate) fn total(self) -> u16 {
        self.hp as u16
            + self.atk as u16
            + self.def as u16
            + self.spa as u16
            + self.spd as u16
            + self.spe as u16
    }

    pub(crate) fn get(self, stat: StatName) -> u8 {
        match stat {
            StatName::Hp => self.hp,
            StatName::Atk => self.atk,
            StatName::Def => self.def,
            StatName::SpA => self.spa,
            StatName::SpD => self.spd,
            StatName::Spe => self.spe,
        }
    }

    pub(crate) fn set(&mut self, stat: StatName, value: u8) {
        match stat {
            StatName::Hp => self.hp = value,
            StatName::Atk => self.atk = value,
            StatName::Def => self.def = value,
            StatName::SpA => self.spa = value,
            StatName::SpD => self.spd = value,
            StatName::Spe => self.spe = value,
        }
    }

    pub(crate) fn validate(self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.total() > 66 {
            errors.push(format!("total SPs must be <= 66, found {}", self.total()));
        }
        for stat in StatName::ALL {
            let value = self.get(stat);
            if value > 32 {
                errors.push(format!("{stat} SPs must be <= 32, found {value}"));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub(crate) fn parse(input: &str) -> Result<Self, String> {
        let mut points = StatPoints::default();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(points);
        }

        for part in trimmed.split('/') {
            let mut pieces = part.split_whitespace();
            let value = pieces
                .next()
                .ok_or_else(|| format!("missing SP value in '{part}'"))?
                .parse::<u8>()
                .map_err(|_| format!("invalid SP value in '{part}'"))?;
            let stat_text = pieces.collect::<Vec<_>>().join(" ");
            if stat_text.is_empty() {
                return Err(format!("missing stat name in '{part}'"));
            }
            let stat = stat_text.parse::<StatName>()?;
            points.set(stat, value);
        }

        Ok(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stat_points() {
        let points = StatPoints::parse("2 Def / 32 SpA / 32 Spe").unwrap();
        assert_eq!(points.hp, 0);
        assert_eq!(points.atk, 0);
        assert_eq!(points.def, 2);
        assert_eq!(points.spa, 32);
        assert_eq!(points.spd, 0);
        assert_eq!(points.spe, 32);
    }

    #[test]
    fn validates_total_and_individual_stat_points() {
        assert!(
            StatPoints {
                hp: 32,
                atk: 2,
                def: 32,
                ..StatPoints::default()
            }
            .validate()
            .is_ok()
        );
        assert!(
            StatPoints {
                hp: 32,
                atk: 3,
                def: 32,
                ..StatPoints::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            StatPoints {
                hp: 33,
                ..StatPoints::default()
            }
            .validate()
            .is_err()
        );
    }
}
