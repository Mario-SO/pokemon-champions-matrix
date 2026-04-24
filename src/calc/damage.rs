use super::modifiers::Modifier;
use super::stats::calculate_pokemon_stats;
use crate::model::type_chart::effectiveness_against_types;
use crate::model::{
    MoveCategory, MoveData, PokemonType, ResolvedBattleScenario, ResolvedPokemon, SideState, Stats,
    Status, Weather,
};

#[derive(Debug, Clone)]
pub(crate) struct DamageRange {
    pub min_percent: f32,
    pub max_percent: f32,
    pub ko_chance: KoChance,
}

#[derive(Debug, Clone)]
pub(crate) struct KoChance {
    pub ohko: Chance,
    pub two_hko: Chance,
}

#[derive(Debug, Clone)]
pub(crate) struct Chance {
    pub percent: f32,
}

pub(crate) fn calculate_damage_range(
    scenario: &ResolvedBattleScenario,
    attacker: &ResolvedPokemon,
    defender: &ResolvedPokemon,
    move_data: &MoveData,
) -> DamageRange {
    let attacker_stats = calculate_pokemon_stats(attacker);
    let defender_stats = calculate_pokemon_stats(defender);

    let defender_max_hp = defender_stats.hp;
    let current_hp = current_hp(defender_max_hp, defender.set.hp_percent);

    let rolls = if move_data.category == MoveCategory::Status {
        vec![0; 16]
    } else if let Some(power) = move_data.power {
        damage_rolls(
            scenario,
            attacker,
            defender,
            move_data,
            attacker_stats,
            defender_stats,
            power,
        )
    } else {
        vec![0; 16]
    };

    let min_damage = *rolls.iter().min().unwrap_or(&0);
    let max_damage = *rolls.iter().max().unwrap_or(&0);
    let min_percent = percent(min_damage, defender_max_hp);
    let max_percent = percent(max_damage, defender_max_hp);
    let ko_chance = calculate_ko_chance(&rolls, current_hp);

    DamageRange {
        min_percent,
        max_percent,
        ko_chance,
    }
}

#[allow(clippy::too_many_arguments)]
fn damage_rolls(
    scenario: &ResolvedBattleScenario,
    attacker: &ResolvedPokemon,
    defender: &ResolvedPokemon,
    move_data: &MoveData,
    attacker_stats: Stats,
    defender_stats: Stats,
    power: u16,
) -> Vec<u16> {
    let (attack, defense) = match move_data.category {
        MoveCategory::Physical => (attacker_stats.atk, defender_stats.def),
        MoveCategory::Special => (attacker_stats.spa, defender_stats.spd),
        MoveCategory::Status => return vec![0; 16],
    };

    let level = attacker.set.level as u32;
    let base_damage =
        (((((2 * level) / 5 + 2) * power as u32 * attack as u32) / defense as u32) / 50) + 2;

    let attacker_side = scenario.side_state(attacker.reference.side);
    let defender_side = scenario.side_state(defender.reference.side);
    let defender_types = defender_effective_types(defender);
    let type_effectiveness = effectiveness_against_types(move_data.move_type, &defender_types);

    (85..=100)
        .map(|random| {
            let mut damage = base_damage;
            if move_data.spread {
                damage = Modifier::new(3, 4).apply_floor(damage);
            }
            damage = apply_weather(damage, scenario.field.weather, move_data.move_type);
            damage = Modifier::new(random, 100).apply_floor(damage);
            damage = apply_stab(damage, attacker, move_data.move_type);
            damage =
                Modifier::new(type_effectiveness.num, type_effectiveness.den).apply_floor(damage);
            damage = apply_burn(damage, attacker, move_data.category);
            damage = apply_screens(damage, move_data.category, defender_side);
            damage = apply_helping_hand(damage, attacker_side);
            damage as u16
        })
        .collect()
}

fn defender_effective_types(defender: &ResolvedPokemon) -> Vec<PokemonType> {
    if defender.set.tera_active
        && let Some(tera_type) = defender.set.tera_type
    {
        return vec![tera_type];
    }
    defender.data.types.clone()
}

fn apply_weather(damage: u32, weather: Weather, move_type: PokemonType) -> u32 {
    match (weather, move_type) {
        (Weather::Sun, PokemonType::Fire) | (Weather::Rain, PokemonType::Water) => {
            Modifier::new(3, 2).apply_floor(damage)
        }
        (Weather::Sun, PokemonType::Water) | (Weather::Rain, PokemonType::Fire) => {
            Modifier::new(1, 2).apply_floor(damage)
        }
        _ => damage,
    }
}

fn apply_stab(damage: u32, attacker: &ResolvedPokemon, move_type: PokemonType) -> u32 {
    if attacker.set.tera_active {
        if attacker.set.tera_type == Some(move_type) {
            if attacker.data.types.contains(&move_type) {
                Modifier::new(2, 1).apply_floor(damage)
            } else {
                Modifier::new(3, 2).apply_floor(damage)
            }
        } else {
            damage
        }
    } else if attacker.data.types.contains(&move_type) {
        Modifier::new(3, 2).apply_floor(damage)
    } else {
        damage
    }
}

fn apply_burn(damage: u32, attacker: &ResolvedPokemon, category: MoveCategory) -> u32 {
    if attacker.set.status == Status::Burn && category == MoveCategory::Physical {
        Modifier::new(1, 2).apply_floor(damage)
    } else {
        damage
    }
}

fn apply_screens(damage: u32, category: MoveCategory, defender_side: SideState) -> u32 {
    let mut damage = damage;
    if defender_side.aurora_veil {
        damage = Modifier::new(2, 3).apply_floor(damage);
    }
    match category {
        MoveCategory::Physical if defender_side.reflect => Modifier::new(2, 3).apply_floor(damage),
        MoveCategory::Special if defender_side.light_screen => {
            Modifier::new(2, 3).apply_floor(damage)
        }
        _ => damage,
    }
}

fn apply_helping_hand(damage: u32, attacker_side: SideState) -> u32 {
    if attacker_side.helping_hand {
        Modifier::new(3, 2).apply_floor(damage)
    } else {
        damage
    }
}

fn current_hp(max_hp: u16, hp_percent: Option<u8>) -> u16 {
    let percent = hp_percent.unwrap_or(100) as u32;
    (max_hp as u32 * percent).div_ceil(100) as u16
}

fn percent(damage: u16, max_hp: u16) -> f32 {
    if max_hp == 0 {
        0.0
    } else {
        damage as f32 * 100.0 / max_hp as f32
    }
}

fn calculate_ko_chance(rolls: &[u16], hp: u16) -> KoChance {
    let ohko_successes = rolls.iter().filter(|roll| **roll >= hp).count() as u16;
    let ohko = chance(ohko_successes, rolls.len() as u16, "OHKO");

    let mut two_hko_successes = 0u16;
    for first in rolls {
        for second in rolls {
            if *first as u32 + *second as u32 >= hp as u32 {
                two_hko_successes += 1;
            }
        }
    }
    let two_hko = chance(
        two_hko_successes,
        (rolls.len() * rolls.len()) as u16,
        "2HKO",
    );

    KoChance { ohko, two_hko }
}

fn chance(successful_rolls: u16, total_rolls: u16, _label: &str) -> Chance {
    let percent = if total_rolls == 0 {
        0.0
    } else {
        successful_rolls as f32 * 100.0 / total_rolls as f32
    };
    Chance { percent }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BaseStats, BattleSide, FieldState, Nature, PokemonData, PokemonSet, SideState, StatPoints,
        TargetRef,
    };

    fn pokemon(
        side: BattleSide,
        species: &str,
        types: Vec<PokemonType>,
        base_stats: BaseStats,
        nature: Nature,
    ) -> ResolvedPokemon {
        ResolvedPokemon {
            reference: TargetRef { side, slot: 1 },
            set: PokemonSet {
                species: species.to_string(),
                nature,
                stat_points: if species == "Venusaur" {
                    StatPoints {
                        def: 2,
                        spa: 32,
                        spe: 32,
                        ..StatPoints::default()
                    }
                } else {
                    StatPoints {
                        hp: 32,
                        atk: 2,
                        spa: 32,
                        ..StatPoints::default()
                    }
                },
                ..PokemonSet::new(species.to_string(), None)
            },
            data: PokemonData { base_stats, types },
        }
    }

    fn special_move(name: &str, move_type: PokemonType, power: u16) -> MoveData {
        MoveData {
            requested_name: name.to_string(),
            move_type,
            category: MoveCategory::Special,
            power: Some(power),
            spread: false,
        }
    }

    fn physical_move(name: &str, move_type: PokemonType, power: u16) -> MoveData {
        MoveData {
            category: MoveCategory::Physical,
            ..special_move(name, move_type, power)
        }
    }

    fn scenario() -> ResolvedBattleScenario {
        let player = pokemon(
            BattleSide::Player,
            "Venusaur",
            vec![PokemonType::Grass, PokemonType::Poison],
            BaseStats {
                hp: 80,
                atk: 82,
                def: 83,
                spa: 100,
                spd: 100,
                spe: 80,
            },
            Nature::Modest,
        );
        let opponent = pokemon(
            BattleSide::Opponent,
            "Aegislash-Shield",
            vec![PokemonType::Steel, PokemonType::Ghost],
            BaseStats {
                hp: 60,
                atk: 50,
                def: 140,
                spa: 50,
                spd: 140,
                spe: 60,
            },
            Nature::Quiet,
        );
        ResolvedBattleScenario {
            field: FieldState::default(),
            player_side: SideState::default(),
            opponent_side: SideState::default(),
            player: vec![player],
            opponent: vec![opponent],
        }
    }

    fn player_damage(scenario: &ResolvedBattleScenario, move_data: &MoveData) -> DamageRange {
        calculate_damage_range(
            scenario,
            &scenario.player[0],
            &scenario.opponent[0],
            move_data,
        )
    }

    #[test]
    fn damaging_moves_return_damage_range() {
        let scenario = scenario();
        let range = player_damage(
            &scenario,
            &special_move("Flamethrower", PokemonType::Fire, 90),
        );
        assert!(range.min_percent > 0.0);
        assert!(range.min_percent <= range.max_percent);
    }

    #[test]
    fn immune_type_matchup_returns_zero() {
        let mut scenario = scenario();
        scenario.opponent[0].data.types = vec![PokemonType::Normal];
        let range = player_damage(
            &scenario,
            &special_move("Shadow Ball", PokemonType::Ghost, 80),
        );
        assert_eq!(range.max_percent, 0.0);
    }

    #[test]
    fn stab_and_type_effectiveness_modify_damage() {
        let mut stab = scenario();
        stab.opponent[0].data.types = vec![PokemonType::Grass];
        let mut no_stab = stab.clone();
        no_stab.player[0].data.types = vec![PokemonType::Grass];
        let mut neutral = no_stab.clone();
        neutral.opponent[0].data.types = vec![PokemonType::Normal];
        let move_data = special_move("Sludge Bomb", PokemonType::Poison, 90);
        let stab_damage = player_damage(&stab, &move_data).max_percent;
        let no_stab_damage = player_damage(&no_stab, &move_data).max_percent;
        let neutral_damage = player_damage(&neutral, &move_data).max_percent;
        assert!(stab_damage > no_stab_damage);
        assert!(no_stab_damage > neutral_damage);
    }

    #[test]
    fn burn_affects_physical_damage_only() {
        let mut physical = scenario();
        physical.opponent[0].data.types = vec![PokemonType::Normal];
        let tackle = physical_move("Tackle", PokemonType::Normal, 90);
        let unburned = player_damage(&physical, &tackle).max_percent;
        physical.player[0].set.status = Status::Burn;
        let burned = player_damage(&physical, &tackle).max_percent;
        assert!(burned < unburned);

        let mut special = scenario();
        let sludge_bomb = special_move("Sludge Bomb", PokemonType::Poison, 90);
        let unburned = player_damage(&special, &sludge_bomb).max_percent;
        special.player[0].set.status = Status::Burn;
        let burned = player_damage(&special, &sludge_bomb).max_percent;
        assert_eq!(burned, unburned);
    }

    #[test]
    fn sun_modifies_fire_and_water_damage() {
        let mut fire = scenario();
        let flamethrower = special_move("Flamethrower", PokemonType::Fire, 90);
        let neutral_fire = player_damage(&fire, &flamethrower).max_percent;
        fire.field.weather = Weather::Sun;
        let sun_fire = player_damage(&fire, &flamethrower).max_percent;
        assert!(sun_fire > neutral_fire);

        let mut water = scenario();
        let surf = special_move("Surf", PokemonType::Water, 90);
        let neutral_water = player_damage(&water, &surf).max_percent;
        water.field.weather = Weather::Sun;
        let sun_water = player_damage(&water, &surf).max_percent;
        assert!(sun_water < neutral_water);
    }
}
