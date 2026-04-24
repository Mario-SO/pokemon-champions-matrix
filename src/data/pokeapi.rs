use crate::error::PcError;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde::de::DeserializeOwned;

const DEFAULT_BASE_URL: &str = "https://pokeapi.co/api/v2";

#[derive(Debug, Clone)]
pub(crate) struct PokeApiClient {
    client: Client,
    base_url: String,
}

impl Default for PokeApiClient {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .user_agent("pc-pokemon-champions-cli/0.1")
                .build()
                .expect("reqwest blocking client should build"),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }
}

impl PokeApiClient {
    pub(crate) fn get_pokemon(
        &self,
        lookup_name: &str,
        requested_name: &str,
    ) -> Result<PokeApiPokemon, PcError> {
        self.get_json("pokemon", lookup_name, requested_name)
    }

    pub(crate) fn get_move(
        &self,
        lookup_name: &str,
        requested_name: &str,
    ) -> Result<PokeApiMove, PcError> {
        self.get_json("move", lookup_name, requested_name)
    }

    fn get_json<T: DeserializeOwned>(
        &self,
        kind: &'static str,
        lookup_name: &str,
        requested_name: &str,
    ) -> Result<T, PcError> {
        let url = format!(
            "{}/{}/{}",
            self.base_url.trim_end_matches('/'),
            kind,
            lookup_name
        );
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|source| PcError::PokeApiFetch {
                name: requested_name.to_string(),
                kind,
                source,
            })?;
        let status = response.status();
        if !status.is_success() {
            return Err(PcError::PokeApiStatus {
                name: requested_name.to_string(),
                kind,
                status,
            });
        }
        response
            .json::<T>()
            .map_err(|source| PcError::PokeApiFetch {
                name: requested_name.to_string(),
                kind,
                source,
            })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct NamedApiResource {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PokeApiPokemon {
    pub stats: Vec<PokeApiPokemonStat>,
    pub types: Vec<PokeApiPokemonType>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PokeApiPokemonStat {
    pub base_stat: u16,
    pub stat: NamedApiResource,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PokeApiPokemonType {
    pub slot: u8,
    #[serde(rename = "type")]
    pub pokemon_type: NamedApiResource,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PokeApiMove {
    pub power: Option<u16>,
    #[serde(rename = "type")]
    pub move_type: NamedApiResource,
    pub damage_class: NamedApiResource,
    pub target: NamedApiResource,
}
