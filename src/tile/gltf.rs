use std::path::{Path, PathBuf};

use tracing::debug;

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

pub fn load_tile_gltf(base_dir: &Path, uris: &[String]) -> Vec<Result<GltfAsset, TesseraError>> {
    let mut results = Vec::new();

    for uri in uris {
        let path = resolve_uri(base_dir, uri);

        if !is_gltf_like(&path) {
            debug!("Skipping non-GLTF content: {:?}", path);
            continue;
        }

        match gltf::import(&path) {
            Ok((document, buffers, images)) => {
                results.push(Ok(GltfAsset {
                    source_path: path,
                    document,
                    buffers,
                    images,
                }));
            }
            Err(e) => {
                results.push(Err(TesseraError::Processing(format!(
                    "Failed to load GLTF from {:?}: {}",
                    path, e
                ))));
            }
        }
    }

    return results;
}
