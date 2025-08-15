use crate::error::TesseraError;
use crate::tile::load_tile_geometry;
use crate::tileset::traverse::parse_tileset_nodes;
use crate::tileset::{Content, Tileset};
use std::path::Path;

pub mod error;
pub mod geometry;
pub mod tile;
pub mod tileset;
pub mod utils;

pub fn calculate_geometric_error(tileset: &Tileset, base_dir: &Path) -> Result<(), TesseraError> {
    let node_map = parse_tileset_nodes(tileset);

    node_map
        .into_values()
        .filter(|node| node.is_leaf())
        .for_each(|node| {
            let tile_content_uris: Vec<String> = if node.tile.content.is_some() {
                vec![node.tile.content.as_ref().unwrap().uri.clone()]
            } else {
                node.tile
                    .contents
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|c| c.uri.clone())
                    .collect()
            };

            let tile_geometries = tile_content_uris
                .iter()
                .map(|uri| load_tile_geometry(base_dir, uri))
                .collect::<Vec<_>>();

            println!("leaf node: {:?}", tile_geometries);
        });

    println!("Base directory: {:?}", base_dir);
    return Ok(());
}
