use crate::error::{Result, TesseraError};
use crate::utils::{is_gltf_like, strip_query_and_fragment};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct Tileset {
    pub root: Tile,
    #[serde(default)]
    pub asset: Option<Asset>,
}

#[derive(Debug, Deserialize)]
pub struct Asset {
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Tile {
    #[serde(default)]
    pub content: Option<Content>,
    #[serde(default)]
    pub contents: Option<Vec<Content>>, // 3D Tiles 1.1
    #[serde(default)]
    pub children: Vec<Tile>,
    #[serde(default)]
    pub geometricError: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Content {
    #[serde(alias = "url", alias = "uri", alias = "href")]
    pub uri: String,
}

#[derive(Debug)]
pub struct LoadedGltfAsset {
    pub source_path: PathBuf,
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

pub fn load_tileset(path: &Path) -> Result<Tileset> {
    let data = fs::read_to_string(path).map_err(TesseraError::Io)?;
    let tileset: Tileset = serde_json::from_str(&data)
        .map_err(|e| TesseraError::Tileset(format!("Failed to parse tileset.json: {}", e)))?;

    return Ok(tileset);
}

pub fn collect_content_uris(tileset: &Tileset) -> Vec<String> {
    let mut uris = Vec::new();

    fn visit(tile: &Tile, out: &mut Vec<String>) {
        if let Some(c) = &tile.content {
            out.push(c.uri.clone());
        }
        if let Some(multi) = &tile.contents {
            for c in multi {
                out.push(c.uri.clone());
            }
        }
        for child in &tile.children {
            visit(child, out);
        }
    }

    visit(&tileset.root, &mut uris);

    return uris;
}

pub fn resolve_uri(base_dir: &Path, uri: &str) -> PathBuf {
    // Naively resolve relative URIs against base directory and strip query/fragment
    let trimmed = strip_query_and_fragment(uri);
    let p = Path::new(trimmed);

    if p.is_absolute() {
        return p.to_path_buf();
    }
    return base_dir.join(p);
}

pub fn load_gltf_assets(base_dir: &Path, uris: &[String]) -> Vec<Result<LoadedGltfAsset>> {
    let mut results = Vec::new();

    for uri in uris {
        let path = resolve_uri(base_dir, uri);

        if !is_gltf_like(&path) {
            debug!("Skipping non-GLTF content: {:?}", path);
            continue;
        }

        match gltf::import(&path) {
            Ok((document, buffers, images)) => {
                results.push(Ok(LoadedGltfAsset {
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

pub fn summarize_gltf(doc: &gltf::Document) -> (usize, usize, usize) {
    let mesh_count = doc.meshes().count();
    let node_count = doc.nodes().count();
    let primitive_count = doc.meshes().flat_map(|m| m.primitives()).count();

    return (mesh_count, node_count, primitive_count);
}
