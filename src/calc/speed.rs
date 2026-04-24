use super::modifiers::Modifier;
use super::stats::calculate_pokemon_stats;
use crate::model::{ResolvedBattleScenario, ResolvedPokemon, Room, Status, TargetRef, Weather};
use std::cmp::Reverse;

#[derive(Debug, Clone)]
pub(crate) struct SpeedResult {
    pub entries: Vec<SpeedEntry>,
}

#[derive(Debug, Clone)]
pub(crate) struct SpeedEntry {
    pub rank: usize,
    pub reference: TargetRef,
    pub calculated_speed: u16,
    pub effective_speed: u32,
}

pub(crate) fn calculate_speed_order(scenario: &ResolvedBattleScenario) -> SpeedResult {
    let trick_room = scenario.field.room == Room::TrickRoom;
    let mut entries = scenario
        .active_pokemon()
        .into_iter()
        .map(|pokemon| speed_entry(scenario, pokemon))
        .collect::<Vec<_>>();

    if trick_room {
        entries.sort_by_key(|entry| entry.effective_speed);
    } else {
        entries.sort_by_key(|entry| Reverse(entry.effective_speed));
    }

    for (index, entry) in entries.iter_mut().enumerate() {
        entry.rank = index + 1;
    }

    SpeedResult { entries }
}

fn speed_entry(scenario: &ResolvedBattleScenario, pokemon: &ResolvedPokemon) -> SpeedEntry {
    let stats = calculate_pokemon_stats(pokemon);
    let mut effective_speed = stats.spe as u32;
    let side_state = scenario.side_state(pokemon.reference.side);

    if side_state.tailwind {
        effective_speed = Modifier::new(2, 1).apply_floor(effective_speed);
    }

    if let Some(ability) = pokemon.set.ability.as_deref() {
        let normalized = ability.trim().to_ascii_lowercase();
        match (normalized.as_str(), scenario.field.weather) {
            ("chlorophyll", Weather::Sun) => {
                effective_speed = Modifier::new(2, 1).apply_floor(effective_speed);
            }
            ("swift swim", Weather::Rain) => {
                effective_speed = Modifier::new(2, 1).apply_floor(effective_speed);
            }
            ("sand rush", Weather::Sand) => {
                effective_speed = Modifier::new(2, 1).apply_floor(effective_speed);
            }
            ("slush rush", Weather::Snow) => {
                effective_speed = Modifier::new(2, 1).apply_floor(effective_speed);
            }
            _ => {}
        }
    }

    if pokemon.set.status == Status::Paralysis {
        effective_speed = Modifier::new(1, 2).apply_floor(effective_speed);
    }

    SpeedEntry {
        rank: 0,
        reference: pokemon.reference,
        calculated_speed: stats.spe,
        effective_speed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BaseStats;
    use crate::model::{
        BattleSide, FieldState, Nature, PokemonData, PokemonSet, PokemonType,
        ResolvedBattleScenario, ResolvedPokemon, Room, SideState, StatPoints, TargetRef, Weather,
    };

    fn resolved(room: Room) -> ResolvedBattleScenario {
        let venusaur = ResolvedPokemon {
            reference: TargetRef {
                side: BattleSide::Player,
                slot: 1,
            },
            set: PokemonSet {
                ability: Some("Chlorophyll".to_string()),
                nature: Nature::Modest,
                stat_points: StatPoints {
                    spe: 32,
                    ..StatPoints::default()
                },
                species: "Venusaur".to_string(),
                ..PokemonSet::new("Venusaur".to_string(), None)
            },
            data: PokemonData {
                base_stats: BaseStats {
                    hp: 80,
                    atk: 82,
                    def: 83,
                    spa: 100,
                    spd: 100,
                    spe: 80,
                },
                types: vec![PokemonType::Grass, PokemonType::Poison],
            },
        };

        let aegislash = ResolvedPokemon {
            reference: TargetRef {
                side: BattleSide::Opponent,
                slot: 1,
            },
            set: PokemonSet {
                nature: Nature::Quiet,
                species: "Aegislash-Shield".to_string(),
                ..PokemonSet::new("Aegislash-Shield".to_string(), None)
            },
            data: PokemonData {
                base_stats: BaseStats {
                    hp: 60,
                    atk: 50,
                    def: 140,
                    spa: 50,
                    spd: 140,
                    spe: 60,
                },
                types: vec![PokemonType::Steel, PokemonType::Ghost],
            },
        };

        ResolvedBattleScenario {
            field: FieldState {
                weather: Weather::Sun,
                room,
                ..FieldState::default()
            },
            player_side: SideState::default(),
            opponent_side: SideState::default(),
            player: vec![venusaur],
            opponent: vec![aegislash],
        }
    }

    #[test]
    fn chlorophyll_doubles_speed_under_sun() {
        let speed = calculate_speed_order(&resolved(Room::None));
        assert_eq!(speed.entries[0].reference.side, BattleSide::Player);
        assert_eq!(speed.entries[0].effective_speed, 264);
    }

    #[test]
    fn trick_room_reverses_order() {
        let speed = calculate_speed_order(&resolved(Room::TrickRoom));
        assert_eq!(speed.entries[0].reference.side, BattleSide::Opponent);
    }

    #[test]
    fn bench_slots_are_not_in_speed_order() {
        let mut scenario = resolved(Room::None);
        let mut bench = scenario.player[0].clone();
        bench.reference.slot = 3;
        bench.set.species = "Benchmon".to_string();
        bench.data.base_stats.spe = 200;
        scenario.player.push(bench);

        let speed = calculate_speed_order(&scenario);
        assert_eq!(speed.entries.len(), 2);
        assert!(speed.entries.iter().all(|entry| entry.reference.slot != 3));
    }
}
