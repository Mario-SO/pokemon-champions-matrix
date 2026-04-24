use super::{BattleSide, FieldState, ResolvedPokemon, SideState};

#[derive(Debug, Clone)]
pub(crate) struct ResolvedBattleScenario {
    pub field: FieldState,
    pub player_side: SideState,
    pub opponent_side: SideState,
    pub player: Vec<ResolvedPokemon>,
    pub opponent: Vec<ResolvedPokemon>,
}

impl ResolvedBattleScenario {
    pub(crate) fn side_state(&self, side: BattleSide) -> SideState {
        match side {
            BattleSide::Player => self.player_side,
            BattleSide::Opponent => self.opponent_side,
        }
    }

    pub(crate) fn active_pokemon(&self) -> Vec<&ResolvedPokemon> {
        self.player
            .iter()
            .chain(self.opponent.iter())
            .filter(|pokemon| pokemon.reference.slot <= 2)
            .collect()
    }
}
