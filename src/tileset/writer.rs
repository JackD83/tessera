use std::path::Path;

use crate::{error::TesseraError, tileset::Tileset};

pub fn write_tileset(tileset: &Tileset, path: &Path, pretty: bool) -> Result<(), TesseraError> {
    let data = if pretty {
        serde_json::to_string_pretty(&tileset).map_err(TesseraError::Json)?
    } else {
        serde_json::to_string(&tileset).map_err(TesseraError::Json)?
    };

    std::fs::write(path, data).map_err(TesseraError::Io)?;

    return Ok(());
}
