use crate::error::{Result, TesseraError};
use crate::tileset::Tileset;
use std::fs;
use std::path::Path;

pub fn load_tileset(path: &Path) -> Result<Tileset> {
    let data = fs::read_to_string(path).map_err(TesseraError::Io)?;
    let tileset: Tileset = serde_json::from_str(&data)
        .map_err(|e| TesseraError::Tileset(format!("Failed to parse tileset.json: {}", e)))?;

    return Ok(tileset);
}
