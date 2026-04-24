use crate::model::{
    BaseStats, Nature, NatureMultiplier, ResolvedPokemon, StatName, StatPoints, Stats,
};

pub(crate) fn calculate_pokemon_stats(pokemon: &ResolvedPokemon) -> Stats {
    calculate_final_stats(
        pokemon.data.base_stats,
        pokemon.set.stat_points,
        pokemon.set.nature,
        pokemon.set.level,
        &pokemon.set.species,
    )
}

pub(crate) fn calculate_final_stats(
    base: BaseStats,
    sp: StatPoints,
    nature: Nature,
    level: u8,
    species: &str,
) -> Stats {
    // Pokémon Champions stat model assumption:
    // 1 SP is treated as 8 EV-equivalent, and Pokémon's formula floors EV / 4,
    // so each SP contributes 2 points inside the stat formula. IV-equivalent
    // baseline is assumed to be 31 until official Champions mechanics differ.
    let level = level as u32;
    let hp = if species.eq_ignore_ascii_case("shedinja") {
        1
    } else {
        (((2 * base.hp as u32 + 31 + sp.hp as u32 * 2) * level) / 100 + level + 10) as u16
    };

    Stats {
        hp,
        atk: calculate_non_hp(base.atk, sp.atk, nature, StatName::Atk, level),
        def: calculate_non_hp(base.def, sp.def, nature, StatName::Def, level),
        spa: calculate_non_hp(base.spa, sp.spa, nature, StatName::SpA, level),
        spd: calculate_non_hp(base.spd, sp.spd, nature, StatName::SpD, level),
        spe: calculate_non_hp(base.spe, sp.spe, nature, StatName::Spe, level),
    }
}

fn calculate_non_hp(base: u16, sp: u8, nature: Nature, stat: StatName, level: u32) -> u16 {
    let raw = ((2 * base as u32 + 31 + sp as u32 * 2) * level) / 100 + 5;
    match nature.multiplier(stat) {
        NatureMultiplier::Boosted => (raw * 110 / 100) as u16,
        NatureMultiplier::Lowered => (raw * 90 / 100) as u16,
        NatureMultiplier::Neutral => raw as u16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Nature;

    fn venusaur() -> BaseStats {
        BaseStats {
            hp: 80,
            atk: 82,
            def: 83,
            spa: 100,
            spd: 100,
            spe: 80,
        }
    }

    #[test]
    fn calculates_venusaur_speed_with_sps() {
        let stats = calculate_final_stats(
            venusaur(),
            StatPoints {
                spe: 32,
                ..StatPoints::default()
            },
            Nature::Modest,
            50,
            "Venusaur",
        );
        assert_eq!(stats.spe, 132);
    }

    #[test]
    fn calculates_venusaur_spa_with_modest() {
        let stats = calculate_final_stats(
            venusaur(),
            StatPoints {
                spa: 32,
                ..StatPoints::default()
            },
            Nature::Modest,
            50,
            "Venusaur",
        );
        assert_eq!(stats.spa, 167);
    }

    #[test]
    fn calculates_aegislash_shield_hp() {
        let stats = calculate_final_stats(
            BaseStats {
                hp: 60,
                atk: 50,
                def: 140,
                spa: 50,
                spd: 140,
                spe: 60,
            },
            StatPoints {
                hp: 32,
                ..StatPoints::default()
            },
            Nature::Quiet,
            50,
            "Aegislash-Shield",
        );
        assert_eq!(stats.hp, 167);
    }
}
