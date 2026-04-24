use crate::error::PcError;
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://pokeapi.co/api/v2";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub(crate) struct PokeApiClient {
    client: Client,
    base_url: String,
    cache_dir: Option<PathBuf>,
}

impl Default for PokeApiClient {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .user_agent("pc-pokemon-champions-cli/0.1")
                .timeout(REQUEST_TIMEOUT)
                .build()
                .expect("reqwest blocking client should build"),
            base_url: DEFAULT_BASE_URL.to_string(),
            cache_dir: pc_cache_dir().ok(),
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
        if let Some(cached) = self.read_cached_json(kind, lookup_name)? {
            return Ok(cached);
        }

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
        let body = response.text().map_err(|source| PcError::PokeApiFetch {
            name: requested_name.to_string(),
            kind,
            source,
        })?;
        let parsed = serde_json::from_str::<T>(&body).map_err(|source| PcError::PokeApiData {
            name: requested_name.to_string(),
            message: source.to_string(),
        })?;
        self.write_cached_json(kind, lookup_name, &body);
        Ok(parsed)
    }

    fn read_cached_json<T: DeserializeOwned>(
        &self,
        kind: &'static str,
        lookup_name: &str,
    ) -> Result<Option<T>, PcError> {
        let Some(path) = self.cache_path(kind, lookup_name) else {
            return Ok(None);
        };
        if !path.exists() {
            return Ok(None);
        }
        let body = fs::read_to_string(&path).map_err(|source| PcError::Io {
            path: path.display().to_string(),
            source,
        })?;
        match serde_json::from_str::<T>(&body) {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => {
                let _ = fs::remove_file(path);
                Ok(None)
            }
        }
    }

    fn write_cached_json(&self, kind: &'static str, lookup_name: &str, body: &str) {
        let Some(path) = self.cache_path(kind, lookup_name) else {
            return;
        };
        let Some(parent) = path.parent() else {
            return;
        };
        if fs::create_dir_all(parent).is_ok() {
            let _ = fs::write(path, body);
        }
    }

    fn cache_path(&self, kind: &'static str, lookup_name: &str) -> Option<PathBuf> {
        self.cache_dir.as_ref().map(|dir| {
            dir.join("pokeapi")
                .join(kind)
                .join(cache_file_name(lookup_name))
        })
    }
}

fn cache_file_name(lookup_name: &str) -> String {
    let safe_name = lookup_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    format!("{safe_name}.json")
}

fn pc_cache_dir() -> Result<PathBuf, PcError> {
    if let Some(value) = std::env::var_os("PC_CACHE_DIR") {
        return Ok(PathBuf::from(value));
    }
    if let Some(value) = std::env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(value).join("pc"));
    }
    if let Some(value) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(value).join(".cache").join("pc"));
    }
    Err(PcError::Validation {
        message: "could not determine a cache directory; set PC_CACHE_DIR".to_string(),
    })
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct NamedApiResource {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PokeApiPokemon {
    pub stats: Vec<PokeApiPokemonStat>,
    pub types: Vec<PokeApiPokemonType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PokeApiPokemonStat {
    pub base_stat: u16,
    pub stat: NamedApiResource,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PokeApiPokemonType {
    pub slot: u8,
    #[serde(rename = "type")]
    pub pokemon_type: NamedApiResource,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct PokeApiMove {
    pub power: Option<u16>,
    #[serde(rename = "type")]
    pub move_type: NamedApiResource,
    pub damage_class: NamedApiResource,
    pub target: NamedApiResource,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_file_names_are_filesystem_safe() {
        assert_eq!(cache_file_name("mr-mime"), "mr-mime.json");
        assert_eq!(cache_file_name("type/null"), "type-null.json");
    }

    #[test]
    fn reads_pokemon_from_disk_cache() {
        let cache_dir = tempfile::tempdir().unwrap();
        let client = PokeApiClient {
            client: Client::builder().timeout(REQUEST_TIMEOUT).build().unwrap(),
            base_url: "http://127.0.0.1:9".to_string(),
            cache_dir: Some(cache_dir.path().to_path_buf()),
        };
        let path = cache_dir
            .path()
            .join("pokeapi")
            .join("pokemon")
            .join("venusaur.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            include_str!("../../tests/fixtures/pokeapi/pokemon/venusaur.json"),
        )
        .unwrap();

        let pokemon = client.get_pokemon("venusaur", "Venusaur").unwrap();
        assert_eq!(pokemon.stats.len(), 6);
        assert_eq!(pokemon.types.len(), 2);
    }
}
