use std::path::{Path, PathBuf};

use crate::{error::TesseraError, utils::resolve_uri};

pub struct GltfAsset {
    pub source_path: PathBuf,
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

pub fn is_gltf_like(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext.to_lowercase().as_str(), "gltf" | "glb")
    } else {
        false
    }
}

pub fn load_tile_gltf(base_dir: &Path, uri: &String) -> Result<GltfAsset, TesseraError> {
    let path = resolve_uri(base_dir, uri);

    if !is_gltf_like(&path) {
        return Err(TesseraError::InvalidGltfFile(uri.to_string()));
    }

    match gltf::import(&path) {
        Ok((document, buffers, images)) => {
            // TODO: write a function to convert gltfAsset to Geometry and update return type
            return Ok(GltfAsset {
                source_path: path,
                document,
                buffers,
                images,
            });
        }
        Err(e) => {
            return Err(TesseraError::Processing(format!(
                "Failed to load GLTF from {:?}: {}",
                path, e
            )));
        }
    }
}
