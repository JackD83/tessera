use crate::error::TesseraError;
use crate::tileset::Tileset;
use std::path::Path;

pub mod error;
pub mod tile;
pub mod tileset;
pub mod utils;

pub fn calculate_geometric_error(tileset: &Tileset, base_dir: &Path) -> Result<(), TesseraError> {
    println!("Calculating geometric error for tileset: {:?}", tileset);
    println!("Base directory: {:?}", base_dir);
    return Ok(());
}
