use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum PcError {
    #[error("Could not read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Parse error on line {line}: {message}")]
    Parse { line: usize, message: String },

    #[error("Validation failed: {message}")]
    Validation { message: String },

    #[error(
        "Could not fetch PokéAPI data for {name}. Check the {kind} name and your internet connection."
    )]
    PokeApiFetch {
        name: String,
        kind: &'static str,
        #[source]
        source: reqwest::Error,
    },

    #[error(
        "Could not fetch PokéAPI data for {name}. Check the {kind} name and your internet connection. HTTP status: {status}"
    )]
    PokeApiStatus {
        name: String,
        kind: &'static str,
        status: reqwest::StatusCode,
    },

    #[error("Could not parse PokéAPI data for {name}: {message}")]
    PokeApiData { name: String, message: String },
}
