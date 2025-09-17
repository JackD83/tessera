use crate::error::{Result, TesseraError};
use crate::tileset::{Tile, Tileset};
use std::fs;
use std::path::Path;

pub fn load_tileset(path: &Path) -> Result<Tileset> {
    let data = fs::read_to_string(path).map_err(TesseraError::Io)?;
    let mut tileset: Tileset = serde_json::from_str(&data)
        .map_err(|e| TesseraError::Tileset(format!("Failed to parse tileset.json: {}", e)))?;

    // set internal IDs for lookup
    let mut current_id = 0;

    fn traverse(tile: &mut Tile, current_id: &mut usize) {
        tile.id = *current_id;
        *current_id += 1;
        tile.children
            .iter_mut()
            .for_each(|child| traverse(child, current_id));
    }

    traverse(&mut tileset.root, &mut current_id);

    return Ok(tileset);
}
