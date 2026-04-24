use super::aliases::move_lookup_name;
use super::pokeapi::{PokeApiClient, PokeApiMove};
use crate::error::PcError;
use crate::model::{MoveCategory, MoveData, PokemonType};

pub(crate) struct MoveRepository {
    client: PokeApiClient,
}

impl MoveRepository {
    pub(crate) fn new(client: PokeApiClient) -> Self {
        Self { client }
    }

    pub(crate) fn get_move(&self, name: &str) -> Result<MoveData, PcError> {
        let lookup_name = move_lookup_name(name);
        let response = self.client.get_move(&lookup_name, name)?;
        map_move_response(name, response)
    }
}

pub(super) fn map_move_response(
    requested_name: &str,
    response: PokeApiMove,
) -> Result<MoveData, PcError> {
    let move_type = response
        .move_type
        .name
        .parse::<PokemonType>()
        .map_err(|message| PcError::PokeApiData {
            name: requested_name.to_string(),
            message,
        })?;

    let category = match response.damage_class.name.as_str() {
        "physical" => MoveCategory::Physical,
        "special" => MoveCategory::Special,
        "status" => MoveCategory::Status,
        other => {
            return Err(PcError::PokeApiData {
                name: requested_name.to_string(),
                message: format!("unknown move damage class '{other}'"),
            });
        }
    };

    let target = response.target.name;
    let spread = matches!(
        target.as_str(),
        "all-opponents" | "all-other-pokemon" | "all-pokemon"
    );

    Ok(MoveData {
        requested_name: requested_name.to_string(),
        move_type,
        category,
        power: response.power,
        spread,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::pokeapi::PokeApiMove;

    #[test]
    fn maps_move_fixture() {
        let json = include_str!("../../tests/fixtures/pokeapi/move/sludge-bomb.json");
        let response: PokeApiMove = serde_json::from_str(json).unwrap();
        let data = map_move_response("Sludge Bomb", response).unwrap();
        assert_eq!(data.move_type, PokemonType::Poison);
        assert_eq!(data.category, MoveCategory::Special);
        assert_eq!(data.power, Some(90));
        assert!(!data.spread);
    }

    #[test]
    fn maps_shadow_ball_fixture() {
        let json = include_str!("../../tests/fixtures/pokeapi/move/shadow-ball.json");
        let response: PokeApiMove = serde_json::from_str(json).unwrap();
        let data = map_move_response("Shadow Ball", response).unwrap();
        assert_eq!(data.move_type, PokemonType::Ghost);
        assert_eq!(data.category, MoveCategory::Special);
        assert_eq!(data.power, Some(80));
    }
}
