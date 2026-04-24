use super::aliases::pokemon_lookup_name;
use super::pokeapi::{PokeApiClient, PokeApiPokemon};
use crate::error::PcError;
use crate::model::{BaseStats, PokemonData, PokemonType};

pub(crate) struct PokemonRepository {
    client: PokeApiClient,
}

impl PokemonRepository {
    pub(crate) fn new(client: PokeApiClient) -> Self {
        Self { client }
    }

    pub(crate) fn get_pokemon(&self, name: &str) -> Result<PokemonData, PcError> {
        let lookup_name = pokemon_lookup_name(name);
        let response = self.client.get_pokemon(&lookup_name, name)?;
        map_pokemon_response(name, response)
    }
}

pub(super) fn map_pokemon_response(
    requested_name: &str,
    response: PokeApiPokemon,
) -> Result<PokemonData, PcError> {
    let mut hp = None;
    let mut atk = None;
    let mut def = None;
    let mut spa = None;
    let mut spd = None;
    let mut spe = None;

    for stat in &response.stats {
        match stat.stat.name.as_str() {
            "hp" => hp = Some(stat.base_stat),
            "attack" => atk = Some(stat.base_stat),
            "defense" => def = Some(stat.base_stat),
            "special-attack" => spa = Some(stat.base_stat),
            "special-defense" => spd = Some(stat.base_stat),
            "speed" => spe = Some(stat.base_stat),
            _ => {}
        }
    }

    let base_stats = BaseStats {
        hp: hp.ok_or_else(|| missing_stat(requested_name, "hp"))?,
        atk: atk.ok_or_else(|| missing_stat(requested_name, "attack"))?,
        def: def.ok_or_else(|| missing_stat(requested_name, "defense"))?,
        spa: spa.ok_or_else(|| missing_stat(requested_name, "special-attack"))?,
        spd: spd.ok_or_else(|| missing_stat(requested_name, "special-defense"))?,
        spe: spe.ok_or_else(|| missing_stat(requested_name, "speed"))?,
    };

    let mut api_types = response.types;
    api_types.sort_by_key(|slot| slot.slot);
    let types = api_types
        .into_iter()
        .map(|slot| {
            slot.pokemon_type
                .name
                .parse::<PokemonType>()
                .map_err(|message| PcError::PokeApiData {
                    name: requested_name.to_string(),
                    message,
                })
        })
        .collect::<Result<Vec<_>, PcError>>()?;

    if types.is_empty() {
        return Err(PcError::PokeApiData {
            name: requested_name.to_string(),
            message: "Pokémon response did not include typing".to_string(),
        });
    }

    Ok(PokemonData { base_stats, types })
}

fn missing_stat(requested_name: &str, stat: &str) -> PcError {
    PcError::PokeApiData {
        name: requested_name.to_string(),
        message: format!("Pokémon response did not include base stat '{stat}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::pokeapi::PokeApiPokemon;

    #[test]
    fn maps_pokemon_fixture() {
        let json = include_str!("../../tests/fixtures/pokeapi/pokemon/venusaur.json");
        let response: PokeApiPokemon = serde_json::from_str(json).unwrap();
        let data = map_pokemon_response("Venusaur", response).unwrap();
        assert_eq!(data.base_stats.hp, 80);
        assert_eq!(data.base_stats.spa, 100);
        assert_eq!(data.types.len(), 2);
    }

    #[test]
    fn maps_form_species_from_pokemon_fixture() {
        let json = include_str!("../../tests/fixtures/pokeapi/pokemon/aegislash-shield.json");
        let response: PokeApiPokemon = serde_json::from_str(json).unwrap();
        let data = map_pokemon_response("Aegislash-Shield", response).unwrap();
        assert_eq!(data.base_stats.def, 140);
        assert_eq!(data.types.len(), 2);
    }
}
