mod calc;
mod cli;
mod data;
mod error;
mod matrix;
mod model;

pub use error::PcError;

pub fn run() -> Result<(), PcError> {
    cli::run()
}
