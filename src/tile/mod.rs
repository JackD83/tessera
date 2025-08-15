use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::{
    error::TesseraError,
    geometry::Geometry,
    tile::gltf::{is_gltf_like, load_tile_gltf},
};

pub mod gltf;

#[derive(Debug)]
enum TileType {
    GLTF,
    B3DM,
    PNTS,
}

fn get_loader_for_uri(uri: PathBuf) -> Result<TileType, TesseraError> {
    if is_gltf_like(&uri) {
        return Ok(TileType::GLTF);
    } else {
        let uri_as_str = if let Some(uri_str) = uri.to_str() {
            uri_str.to_string()
        } else {
            "Invalid URI".to_string()
        };

        return Err(TesseraError::UnsupportedTileType(uri_as_str));
    }
}

pub fn load_tile_geometry(
    base_dir: &Path,
    content_uri: &String,
) -> Vec<Result<Geometry, TesseraError>> {
    let full_content_uri = Path::new(base_dir).join(content_uri);
    let tile_type = get_loader_for_uri(full_content_uri);
    let mut results = Vec::<Result<Geometry, TesseraError>>::new();

    // TODO: Add B3DM loader (check 28 byte header then skip to correct position and load as slice)
    match tile_type {
        Ok(TileType::GLTF) => {
            let _gltf_asset = load_tile_gltf(base_dir, content_uri);
        }
        Err(e) => {
            results.push(Err(e));
        }
        _ => {
            results.push(Err(TesseraError::UnsupportedTileType(format!(
                "{:?}",
                tile_type.unwrap()
            ))));
        }
    }

    return results;
}
