use crate::config::pc_config_dir;
use crate::error::PcError;
use reqwest::blocking::Client;
use rusqlite::{Connection, params};
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
    cache_path: Option<PathBuf>,
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
            cache_path: pc_config_dir().ok().map(|dir| dir.join("pc.sqlite")),
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
        self.write_cached_json(kind, lookup_name, &body)?;
        Ok(parsed)
    }

    fn read_cached_json<T: DeserializeOwned>(
        &self,
        kind: &'static str,
        lookup_name: &str,
    ) -> Result<Option<T>, PcError> {
        let Some(body) = self.cached_response_json(kind, lookup_name)? else {
            return Ok(None);
        };
        match serde_json::from_str::<T>(&body) {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => {
                self.delete_cached_json(kind, lookup_name)?;
                Ok(None)
            }
        }
    }

    fn write_cached_json(
        &self,
        kind: &'static str,
        lookup_name: &str,
        body: &str,
    ) -> Result<(), PcError> {
        let Some(connection) = self.cache_connection()? else {
            return Ok(());
        };
        connection
            .execute(
                "INSERT INTO pokeapi_cache (kind, lookup_name, response_json)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(kind, lookup_name) DO UPDATE SET
                   response_json = excluded.response_json,
                   fetched_at = CURRENT_TIMESTAMP",
                params![kind, lookup_name, body],
            )
            .map_err(|source| self.cache_error(source))?;
        Ok(())
    }

    fn cached_response_json(
        &self,
        kind: &'static str,
        lookup_name: &str,
    ) -> Result<Option<String>, PcError> {
        let Some(path) = self.cache_path.as_ref() else {
            return Ok(None);
        };
        if !path.exists() {
            return Ok(None);
        }
        let Some(connection) = self.cache_connection()? else {
            return Ok(None);
        };
        match connection.query_row(
            "SELECT response_json FROM pokeapi_cache WHERE kind = ?1 AND lookup_name = ?2",
            params![kind, lookup_name],
            |row| row.get(0),
        ) {
            Ok(body) => Ok(Some(body)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(source) => Err(self.cache_error(source)),
        }
    }

    fn delete_cached_json(&self, kind: &'static str, lookup_name: &str) -> Result<(), PcError> {
        let Some(connection) = self.cache_connection()? else {
            return Ok(());
        };
        connection
            .execute(
                "DELETE FROM pokeapi_cache WHERE kind = ?1 AND lookup_name = ?2",
                params![kind, lookup_name],
            )
            .map_err(|source| self.cache_error(source))?;
        Ok(())
    }

    fn cache_connection(&self) -> Result<Option<Connection>, PcError> {
        let Some(path) = self.cache_path.as_ref() else {
            return Ok(None);
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| PcError::Io {
                path: parent.display().to_string(),
                source,
            })?;
        }
        let connection = Connection::open(path).map_err(|source| self.cache_error(source))?;
        initialize_cache(&connection).map_err(|source| self.cache_error(source))?;
        Ok(Some(connection))
    }

    fn cache_error(&self, source: rusqlite::Error) -> PcError {
        PcError::Cache {
            path: self
                .cache_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "<disabled>".to_string()),
            source,
        }
    }
}

fn initialize_cache(connection: &Connection) -> Result<(), rusqlite::Error> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS pokeapi_cache (
           kind TEXT NOT NULL,
           lookup_name TEXT NOT NULL,
           response_json TEXT NOT NULL,
           fetched_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
           PRIMARY KEY (kind, lookup_name)
         )",
        [],
    )?;
    Ok(())
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
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn reads_pokemon_from_sqlite_cache() {
        let cache_dir = tempfile::tempdir().unwrap();
        let client = PokeApiClient {
            client: Client::builder().timeout(REQUEST_TIMEOUT).build().unwrap(),
            base_url: "http://127.0.0.1:9".to_string(),
            cache_path: Some(cache_dir.path().join("pc.sqlite")),
        };

        client
            .write_cached_json(
                "pokemon",
                "venusaur",
                include_str!("../../tests/fixtures/pokeapi/pokemon/venusaur.json"),
            )
            .unwrap();

        let pokemon = client.get_pokemon("venusaur", "Venusaur").unwrap();
        assert_eq!(pokemon.stats.len(), 6);
        assert_eq!(pokemon.types.len(), 2);
    }

    #[test]
    fn cache_miss_fetches_and_stores_response() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache_path = cache_dir.path().join("pc.sqlite");
        let base_url = serve_one_response(include_str!(
            "../../tests/fixtures/pokeapi/pokemon/venusaur.json"
        ));
        let fetch_client = PokeApiClient {
            client: Client::builder().timeout(REQUEST_TIMEOUT).build().unwrap(),
            base_url,
            cache_path: Some(cache_path.clone()),
        };

        let fetched = fetch_client.get_pokemon("venusaur", "Venusaur").unwrap();
        assert_eq!(fetched.stats.len(), 6);

        let cached_client = PokeApiClient {
            client: Client::builder().timeout(REQUEST_TIMEOUT).build().unwrap(),
            base_url: "http://127.0.0.1:9".to_string(),
            cache_path: Some(cache_path),
        };
        let cached = cached_client.get_pokemon("venusaur", "Venusaur").unwrap();
        assert_eq!(cached.stats.len(), 6);
    }

    #[test]
    fn corrupt_cached_json_is_replaced_after_fetch() {
        let cache_dir = tempfile::tempdir().unwrap();
        let cache_path = cache_dir.path().join("pc.sqlite");
        let base_url = serve_one_response(include_str!(
            "../../tests/fixtures/pokeapi/pokemon/venusaur.json"
        ));
        let client = PokeApiClient {
            client: Client::builder().timeout(REQUEST_TIMEOUT).build().unwrap(),
            base_url,
            cache_path: Some(cache_path.clone()),
        };
        client
            .write_cached_json("pokemon", "venusaur", "{not valid json")
            .unwrap();

        let pokemon = client.get_pokemon("venusaur", "Venusaur").unwrap();
        assert_eq!(pokemon.stats.len(), 6);

        let connection = Connection::open(cache_path).unwrap();
        let body: String = connection
            .query_row(
                "SELECT response_json FROM pokeapi_cache WHERE kind = 'pokemon' AND lookup_name = 'venusaur'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(serde_json::from_str::<PokeApiPokemon>(&body).is_ok());
    }

    fn serve_one_response(body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0; 1024];
            let _ = stream.read(&mut buffer).unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        format!("http://{address}")
    }
}
