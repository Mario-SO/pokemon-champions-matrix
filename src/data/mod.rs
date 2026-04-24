mod aliases;
mod move_repository;
pub(crate) mod pokeapi;
mod pokemon_repository;

pub(crate) use move_repository::MoveRepository;
pub(crate) use pokemon_repository::PokemonRepository;
