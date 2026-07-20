use std::path::{Path, PathBuf};

use crate::{
    error::TesseraError,
    geometry::Geometry,
    maths::matrix::Mat4,
    tile::{
        b3dm::{is_b3dm_like, load_tile_b3dm},
        gltf::{is_gltf_like, load_tile_gltf},
        pnts::{is_pnts_like, load_tile_pnts},
    },
};

pub mod b3dm;
pub mod gltf;
pub mod pnts;

#[derive(Debug)]
enum TileType {
    GLTF,
    B3DM,
    PNTS,
}

fn get_loader_for_uri(uri: PathBuf) -> Result<TileType, TesseraError> {
    if is_gltf_like(&uri) {
        return Ok(TileType::GLTF);
    } else if is_b3dm_like(&uri) {
        return Ok(TileType::B3DM);
    } else if is_pnts_like(&uri) {
        return Ok(TileType::PNTS);
    } else {
        let uri_as_str = if let Some(uri_str) = uri.to_str() {
            uri_str.to_string()
        } else {
            "Invalid URI".to_string()
        };

        return Err(TesseraError::UnsupportedTileType(uri_as_str));
    }
}

pub fn load_tile_geometry(base_dir: &Path, content_uri: &String) -> Result<Geometry, TesseraError> {
    return load_tile_geometry_with_transform(base_dir, content_uri, &Mat4::identity());
}

pub fn load_tile_geometry_with_transform(
    base_dir: &Path,
    content_uri: &String,
    transform: &Mat4,
) -> Result<Geometry, TesseraError> {
    let full_content_uri = Path::new(base_dir).join(content_uri);
    let tile_type = get_loader_for_uri(full_content_uri);

    let mut geometry = match tile_type {
        Ok(TileType::GLTF) => load_tile_gltf(base_dir, content_uri)?,
        Ok(TileType::B3DM) => load_tile_b3dm(base_dir, content_uri)?,
        Ok(TileType::PNTS) => load_tile_pnts(base_dir, content_uri)?,
        Err(e) => return Err(e),
    };

    geometry.apply_transform(transform);

    return Ok(geometry);
}
