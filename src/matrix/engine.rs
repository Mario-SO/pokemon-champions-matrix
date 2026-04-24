use super::showdown::ShowdownPokemon;
use crate::calc::{damage, speed, stats};
use crate::data::pokeapi::PokeApiClient;
use crate::data::{MoveRepository, PokemonRepository};
use crate::error::PcError;
use crate::model::{
    BattleSide, FieldState, MoveCategory, MoveData, PokemonData, PokemonSet, PokemonType,
    ResolvedBattleScenario, ResolvedPokemon, SideState, Stats, Status, TargetRef,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MatrixMode {
    Offensive,
    Defensive,
    Speed,
}

impl MatrixMode {
    pub(super) fn title(self) -> &'static str {
        match self {
            MatrixMode::Offensive => "Offensive",
            MatrixMode::Defensive => "Defensive",
            MatrixMode::Speed => "Speed",
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct MatrixPokemon {
    pub set: PokemonSet,
    pub data: PokemonData,
    pub moves: Vec<MoveData>,
}

#[derive(Debug, Clone)]
pub(super) struct MatrixConditions {
    pub field: FieldState,
    pub player_side: SideState,
    pub opponent_side: SideState,
    pub player_statuses: Vec<Status>,
    pub opponent_statuses: Vec<Status>,
}

impl MatrixConditions {
    pub(super) fn with_sizes(player_count: usize, opponent_count: usize) -> Self {
        Self {
            field: FieldState::default(),
            player_side: SideState::default(),
            opponent_side: SideState::default(),
            player_statuses: vec![Status::None; player_count],
            opponent_statuses: vec![Status::None; opponent_count],
        }
    }

    pub(super) fn resize(&mut self, player_count: usize, opponent_count: usize) {
        self.player_statuses.resize(player_count, Status::None);
        self.opponent_statuses.resize(opponent_count, Status::None);
    }
}

#[derive(Debug, Clone)]
pub(super) struct MatrixCard {
    pub opponent_index: usize,
    pub name: String,
    pub item: Option<String>,
    pub ability: Option<String>,
    pub types: Vec<PokemonType>,
    pub rows: Vec<MatrixRow>,
    pub speed: Option<SpeedMatrixRow>,
}

#[derive(Debug, Clone)]
pub(super) struct MatrixRow {
    pub move_name: String,
    pub min_percent: f32,
    pub max_percent: f32,
    pub ohko_percent: f32,
    pub two_hko_percent: f32,
}

#[derive(Debug, Clone)]
pub(super) struct SpeedMatrixRow {
    pub player_speed: u32,
    pub opponent_speed: u32,
    pub player_raw_speed: u16,
    pub opponent_raw_speed: u16,
    pub outcome: SpeedOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SpeedOutcome {
    PlayerFirst,
    OpponentFirst,
    Tie,
}

pub(super) struct MatrixResolver {
    pokemon_repository: PokemonRepository,
    move_repository: MoveRepository,
    pokemon_cache: HashMap<String, PokemonData>,
    move_cache: HashMap<String, MoveData>,
}

impl MatrixResolver {
    pub(super) fn new() -> Self {
        let client = PokeApiClient::default();
        Self {
            pokemon_repository: PokemonRepository::new(client.clone()),
            move_repository: MoveRepository::new(client),
            pokemon_cache: HashMap::new(),
            move_cache: HashMap::new(),
        }
    }

    pub(super) fn resolve_team(
        &mut self,
        team: &[ShowdownPokemon],
    ) -> Result<Vec<MatrixPokemon>, PcError> {
        team.iter()
            .map(|pokemon| {
                let data = self.resolve_pokemon(&pokemon.set.species)?;
                let moves = pokemon
                    .moves
                    .iter()
                    .map(|move_name| self.resolve_move(move_name))
                    .collect::<Result<Vec<_>, PcError>>()?;
                Ok(MatrixPokemon {
                    set: pokemon.set.clone(),
                    data,
                    moves,
                })
            })
            .collect()
    }

    fn resolve_pokemon(&mut self, name: &str) -> Result<PokemonData, PcError> {
        let key = name.trim().to_ascii_lowercase();
        if let Some(data) = self.pokemon_cache.get(&key) {
            return Ok(data.clone());
        }
        let data = self.pokemon_repository.get_pokemon(name)?;
        self.pokemon_cache.insert(key, data.clone());
        Ok(data)
    }

    fn resolve_move(&mut self, name: &str) -> Result<MoveData, PcError> {
        let key = name.trim().to_ascii_lowercase();
        if let Some(data) = self.move_cache.get(&key) {
            return Ok(data.clone());
        }
        let data = self.move_repository.get_move(name)?;
        self.move_cache.insert(key, data.clone());
        Ok(data)
    }
}

impl Default for MatrixResolver {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn build_cards(
    mode: MatrixMode,
    player: &MatrixPokemon,
    player_index: usize,
    opponents: &[MatrixPokemon],
    conditions: &MatrixConditions,
) -> Result<Vec<MatrixCard>, PcError> {
    opponents
        .iter()
        .enumerate()
        .map(|(opponent_index, opponent)| {
            calculate_card(
                mode,
                player,
                player_index,
                opponent,
                opponent_index,
                conditions,
            )
        })
        .collect()
}

pub(super) fn calculate_card(
    mode: MatrixMode,
    player: &MatrixPokemon,
    player_index: usize,
    opponent: &MatrixPokemon,
    opponent_index: usize,
    conditions: &MatrixConditions,
) -> Result<MatrixCard, PcError> {
    let mut rows = Vec::new();
    let mut speed_row = None;

    match mode {
        MatrixMode::Offensive => {
            for move_data in player
                .moves
                .iter()
                .filter(|move_data| is_damaging(move_data))
            {
                rows.push(damage_row(
                    player,
                    player_index,
                    opponent,
                    opponent_index,
                    move_data,
                    true,
                    conditions,
                )?);
            }
        }
        MatrixMode::Defensive => {
            for move_data in opponent
                .moves
                .iter()
                .filter(|move_data| is_damaging(move_data))
            {
                rows.push(damage_row(
                    player,
                    player_index,
                    opponent,
                    opponent_index,
                    move_data,
                    false,
                    conditions,
                )?);
            }
        }
        MatrixMode::Speed => {
            speed_row = Some(speed_row_for(
                player,
                player_index,
                opponent,
                opponent_index,
                conditions,
            ));
        }
    }

    Ok(MatrixCard {
        opponent_index,
        name: opponent.set.display_name().to_string(),
        item: opponent.set.item.clone(),
        ability: opponent.set.ability.clone(),
        types: opponent.data.types.clone(),
        rows,
        speed: speed_row,
    })
}

fn is_damaging(move_data: &MoveData) -> bool {
    move_data.category != MoveCategory::Status && move_data.power.is_some()
}

fn damage_row(
    player: &MatrixPokemon,
    player_index: usize,
    opponent: &MatrixPokemon,
    opponent_index: usize,
    move_data: &MoveData,
    player_attacks: bool,
    conditions: &MatrixConditions,
) -> Result<MatrixRow, PcError> {
    let scenario = scenario_for(player, player_index, opponent, opponent_index, conditions);
    let player_resolved = &scenario.player[0];
    let opponent_resolved = &scenario.opponent[0];
    let (attacker, defender) = if player_attacks {
        (player_resolved, opponent_resolved)
    } else {
        (opponent_resolved, player_resolved)
    };
    let range = damage::calculate_damage_range(&scenario, attacker, defender, move_data);
    Ok(MatrixRow {
        move_name: move_data.requested_name.clone(),
        min_percent: range.min_percent,
        max_percent: range.max_percent,
        ohko_percent: range.ko_chance.ohko.percent,
        two_hko_percent: range.ko_chance.two_hko.percent,
    })
}

fn speed_row_for(
    player: &MatrixPokemon,
    player_index: usize,
    opponent: &MatrixPokemon,
    opponent_index: usize,
    conditions: &MatrixConditions,
) -> SpeedMatrixRow {
    let scenario = scenario_for(player, player_index, opponent, opponent_index, conditions);
    let result = speed::calculate_speed_order(&scenario);
    let player_ref = TargetRef {
        side: BattleSide::Player,
        slot: 1,
    };
    let opponent_ref = TargetRef {
        side: BattleSide::Opponent,
        slot: 1,
    };
    let player_entry = result
        .entries
        .iter()
        .find(|entry| entry.reference == player_ref)
        .expect("pair scenario always contains Player.1");
    let opponent_entry = result
        .entries
        .iter()
        .find(|entry| entry.reference == opponent_ref)
        .expect("pair scenario always contains Opponent.1");

    let outcome = if player_entry.effective_speed == opponent_entry.effective_speed {
        SpeedOutcome::Tie
    } else if result.entries.first().map(|entry| entry.reference) == Some(player_ref) {
        SpeedOutcome::PlayerFirst
    } else {
        SpeedOutcome::OpponentFirst
    };

    SpeedMatrixRow {
        player_speed: player_entry.effective_speed,
        opponent_speed: opponent_entry.effective_speed,
        player_raw_speed: player_entry.calculated_speed,
        opponent_raw_speed: opponent_entry.calculated_speed,
        outcome,
    }
}

fn scenario_for(
    player: &MatrixPokemon,
    player_index: usize,
    opponent: &MatrixPokemon,
    opponent_index: usize,
    conditions: &MatrixConditions,
) -> ResolvedBattleScenario {
    ResolvedBattleScenario {
        field: conditions.field,
        player_side: conditions.player_side,
        opponent_side: conditions.opponent_side,
        player: vec![resolved_pokemon(
            player,
            BattleSide::Player,
            conditions
                .player_statuses
                .get(player_index)
                .copied()
                .unwrap_or(Status::None),
        )],
        opponent: vec![resolved_pokemon(
            opponent,
            BattleSide::Opponent,
            conditions
                .opponent_statuses
                .get(opponent_index)
                .copied()
                .unwrap_or(Status::None),
        )],
    }
}

fn resolved_pokemon(pokemon: &MatrixPokemon, side: BattleSide, status: Status) -> ResolvedPokemon {
    let mut set = pokemon.set.clone();
    set.status = status;
    ResolvedPokemon {
        reference: TargetRef { side, slot: 1 },
        set,
        data: pokemon.data.clone(),
    }
}

pub(super) fn final_stats(pokemon: &MatrixPokemon, side: BattleSide) -> Stats {
    let resolved = ResolvedPokemon {
        reference: TargetRef { side, slot: 1 },
        set: pokemon.set.clone(),
        data: pokemon.data.clone(),
    };
    stats::calculate_pokemon_stats(&resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BaseStats, Nature};

    #[test]
    fn offensive_mode_returns_damaging_player_moves() {
        let player = fixture_pokemon(
            "Milotic",
            vec![PokemonType::Water],
            BaseStats {
                hp: 95,
                atk: 60,
                def: 79,
                spa: 100,
                spd: 125,
                spe: 81,
            },
            vec![
                special_move("Muddy Water", PokemonType::Water, 90),
                status_move("Recover"),
            ],
        );
        let opponent = fixture_pokemon(
            "Gardevoir",
            vec![PokemonType::Psychic, PokemonType::Fairy],
            BaseStats {
                hp: 68,
                atk: 65,
                def: 65,
                spa: 125,
                spd: 115,
                spe: 80,
            },
            vec![special_move("Moonblast", PokemonType::Fairy, 95)],
        );
        let conditions = MatrixConditions::with_sizes(1, 1);
        let card =
            calculate_card(MatrixMode::Offensive, &player, 0, &opponent, 0, &conditions).unwrap();
        assert_eq!(card.rows.len(), 1);
        assert_eq!(card.rows[0].move_name, "Muddy Water");
    }

    #[test]
    fn defensive_mode_returns_damaging_opponent_moves() {
        let player = fixture_pokemon("Milotic", vec![PokemonType::Water], milotic_stats(), vec![]);
        let opponent = fixture_pokemon(
            "Gardevoir",
            vec![PokemonType::Psychic, PokemonType::Fairy],
            gardevoir_stats(),
            vec![
                special_move("Moonblast", PokemonType::Fairy, 95),
                status_move("Trick Room"),
            ],
        );
        let conditions = MatrixConditions::with_sizes(1, 1);
        let card =
            calculate_card(MatrixMode::Defensive, &player, 0, &opponent, 0, &conditions).unwrap();
        assert_eq!(card.rows.len(), 1);
        assert_eq!(card.rows[0].move_name, "Moonblast");
    }

    #[test]
    fn speed_mode_respects_tailwind_and_trick_room() {
        let player = fixture_pokemon("Milotic", vec![PokemonType::Water], milotic_stats(), vec![]);
        let opponent = fixture_pokemon(
            "Gardevoir",
            vec![PokemonType::Psychic, PokemonType::Fairy],
            gardevoir_stats(),
            vec![],
        );
        let mut conditions = MatrixConditions::with_sizes(1, 1);
        conditions.player_side.tailwind = true;
        let card =
            calculate_card(MatrixMode::Speed, &player, 0, &opponent, 0, &conditions).unwrap();
        assert_eq!(card.speed.unwrap().outcome, SpeedOutcome::PlayerFirst);

        conditions.field.room = crate::model::Room::TrickRoom;
        let card =
            calculate_card(MatrixMode::Speed, &player, 0, &opponent, 0, &conditions).unwrap();
        assert_eq!(card.speed.unwrap().outcome, SpeedOutcome::OpponentFirst);
    }

    #[test]
    fn empty_opponents_returns_no_cards() {
        let player = fixture_pokemon("Milotic", vec![PokemonType::Water], milotic_stats(), vec![]);
        let conditions = MatrixConditions::with_sizes(1, 0);
        let cards = build_cards(MatrixMode::Offensive, &player, 0, &[], &conditions).unwrap();
        assert!(cards.is_empty());
    }

    fn fixture_pokemon(
        name: &str,
        types: Vec<PokemonType>,
        base_stats: BaseStats,
        moves: Vec<MoveData>,
    ) -> MatrixPokemon {
        MatrixPokemon {
            set: PokemonSet {
                species: name.to_string(),
                nature: Nature::Hardy,
                ..PokemonSet::new(name.to_string(), None)
            },
            data: PokemonData { base_stats, types },
            moves,
        }
    }

    fn milotic_stats() -> BaseStats {
        BaseStats {
            hp: 95,
            atk: 60,
            def: 79,
            spa: 100,
            spd: 125,
            spe: 81,
        }
    }

    fn gardevoir_stats() -> BaseStats {
        BaseStats {
            hp: 68,
            atk: 65,
            def: 65,
            spa: 125,
            spd: 115,
            spe: 80,
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

    fn status_move(name: &str) -> MoveData {
        MoveData {
            requested_name: name.to_string(),
            move_type: PokemonType::Normal,
            category: MoveCategory::Status,
            power: None,
            spread: false,
        }
    }
}
