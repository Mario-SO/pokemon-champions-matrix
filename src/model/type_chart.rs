use super::PokemonType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TypeEffectiveness {
    pub num: u32,
    pub den: u32,
}

impl TypeEffectiveness {
    pub(crate) const IMMUNE: Self = Self { num: 0, den: 1 };
    pub(crate) const HALF: Self = Self { num: 1, den: 2 };
    pub(crate) const NORMAL: Self = Self { num: 1, den: 1 };
    pub(crate) const DOUBLE: Self = Self { num: 2, den: 1 };

    pub(crate) fn multiply(self, other: Self) -> Self {
        Self {
            num: self.num * other.num,
            den: self.den * other.den,
        }
    }
}

pub(crate) fn effectiveness_against_types(
    attacking: PokemonType,
    defending: &[PokemonType],
) -> TypeEffectiveness {
    defending
        .iter()
        .copied()
        .fold(TypeEffectiveness::NORMAL, |acc, defender| {
            acc.multiply(effectiveness(attacking, defender))
        })
}

pub(crate) fn effectiveness(attacking: PokemonType, defending: PokemonType) -> TypeEffectiveness {
    use PokemonType::*;
    match attacking {
        Normal => match defending {
            Rock | Steel => TypeEffectiveness::HALF,
            Ghost => TypeEffectiveness::IMMUNE,
            _ => TypeEffectiveness::NORMAL,
        },
        Fire => match defending {
            Fire | Water | Rock | Dragon => TypeEffectiveness::HALF,
            Grass | Ice | Bug | Steel => TypeEffectiveness::DOUBLE,
            _ => TypeEffectiveness::NORMAL,
        },
        Water => match defending {
            Water | Grass | Dragon => TypeEffectiveness::HALF,
            Fire | Ground | Rock => TypeEffectiveness::DOUBLE,
            _ => TypeEffectiveness::NORMAL,
        },
        Grass => match defending {
            Fire | Grass | Poison | Flying | Bug | Dragon | Steel => TypeEffectiveness::HALF,
            Water | Ground | Rock => TypeEffectiveness::DOUBLE,
            _ => TypeEffectiveness::NORMAL,
        },
        Electric => match defending {
            Electric | Grass | Dragon => TypeEffectiveness::HALF,
            Water | Flying => TypeEffectiveness::DOUBLE,
            Ground => TypeEffectiveness::IMMUNE,
            _ => TypeEffectiveness::NORMAL,
        },
        Ice => match defending {
            Fire | Water | Ice | Steel => TypeEffectiveness::HALF,
            Grass | Ground | Flying | Dragon => TypeEffectiveness::DOUBLE,
            _ => TypeEffectiveness::NORMAL,
        },
        Fighting => match defending {
            Poison | Flying | Psychic | Bug | Fairy => TypeEffectiveness::HALF,
            Normal | Ice | Rock | Dark | Steel => TypeEffectiveness::DOUBLE,
            Ghost => TypeEffectiveness::IMMUNE,
            _ => TypeEffectiveness::NORMAL,
        },
        Poison => match defending {
            Poison | Ground | Rock | Ghost => TypeEffectiveness::HALF,
            Grass | Fairy => TypeEffectiveness::DOUBLE,
            Steel => TypeEffectiveness::IMMUNE,
            _ => TypeEffectiveness::NORMAL,
        },
        Ground => match defending {
            Grass | Bug => TypeEffectiveness::HALF,
            Fire | Electric | Poison | Rock | Steel => TypeEffectiveness::DOUBLE,
            Flying => TypeEffectiveness::IMMUNE,
            _ => TypeEffectiveness::NORMAL,
        },
        Flying => match defending {
            Electric | Rock | Steel => TypeEffectiveness::HALF,
            Grass | Fighting | Bug => TypeEffectiveness::DOUBLE,
            _ => TypeEffectiveness::NORMAL,
        },
        Psychic => match defending {
            Psychic | Steel => TypeEffectiveness::HALF,
            Fighting | Poison => TypeEffectiveness::DOUBLE,
            Dark => TypeEffectiveness::IMMUNE,
            _ => TypeEffectiveness::NORMAL,
        },
        Bug => match defending {
            Fire | Fighting | Poison | Flying | Ghost | Steel | Fairy => TypeEffectiveness::HALF,
            Grass | Psychic | Dark => TypeEffectiveness::DOUBLE,
            _ => TypeEffectiveness::NORMAL,
        },
        Rock => match defending {
            Fighting | Ground | Steel => TypeEffectiveness::HALF,
            Fire | Ice | Flying | Bug => TypeEffectiveness::DOUBLE,
            _ => TypeEffectiveness::NORMAL,
        },
        Ghost => match defending {
            Dark => TypeEffectiveness::HALF,
            Psychic | Ghost => TypeEffectiveness::DOUBLE,
            Normal => TypeEffectiveness::IMMUNE,
            _ => TypeEffectiveness::NORMAL,
        },
        Dragon => match defending {
            Steel => TypeEffectiveness::HALF,
            Dragon => TypeEffectiveness::DOUBLE,
            Fairy => TypeEffectiveness::IMMUNE,
            _ => TypeEffectiveness::NORMAL,
        },
        Dark => match defending {
            Fighting | Dark | Fairy => TypeEffectiveness::HALF,
            Psychic | Ghost => TypeEffectiveness::DOUBLE,
            _ => TypeEffectiveness::NORMAL,
        },
        Steel => match defending {
            Fire | Water | Electric | Steel => TypeEffectiveness::HALF,
            Ice | Rock | Fairy => TypeEffectiveness::DOUBLE,
            _ => TypeEffectiveness::NORMAL,
        },
        Fairy => match defending {
            Fire | Poison | Steel => TypeEffectiveness::HALF,
            Fighting | Dragon | Dark => TypeEffectiveness::DOUBLE,
            _ => TypeEffectiveness::NORMAL,
        },
    }
}
