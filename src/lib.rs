use crate::error::TesseraError;
use crate::geometry::Geometry;
use crate::geometry::compare::get_shortest_distance;
use crate::tile::load_tile_geometry;
use crate::tileset::Tileset;
use crate::tileset::traverse::{TilesetNode, parse_tileset_nodes};
use std::path::Path;

pub mod error;
pub mod geometry;
pub mod maths;
pub mod tile;
pub mod tileset;
pub mod utils;

pub fn calculate_geometric_error(tileset: &Tileset, base_dir: &Path) -> Result<(), TesseraError> {
    let (mut node_map, leaf_ids) = parse_tileset_nodes(tileset);

    for leaf_id in leaf_ids {
        let leaf_node = node_map.get(&leaf_id).unwrap();
        let mut current_node = leaf_node;

        // TODO: handle error case
        let leaf_geometry_results = load_tile_geometries(&leaf_node, base_dir);
        let leaf_geometries = leaf_geometry_results
            .iter()
            .map(|r| r.as_ref().unwrap())
            .collect::<Vec<_>>();

        while let Some(parent) = current_node.parent_key {
            let parent_node = node_map.get_mut(&parent).unwrap();

            // TODO: handle error case
            let parent_geometries_results = load_tile_geometries(&parent_node, base_dir);
            let parent_geometries = parent_geometries_results
                .iter()
                .map(|r| r.as_ref().unwrap())
                .collect::<Vec<_>>();

            let geometric_error_result =
                get_shortest_distance(&leaf_geometries, &parent_geometries);

            if geometric_error_result.is_err() {
                return Err(geometric_error_result.err().unwrap());
            }

            let geometric_error = geometric_error_result.unwrap();

            if parent_node.geometric_error_lower_bound.is_none()
                || parent_node.geometric_error_lower_bound.unwrap() < geometric_error
            {
                // todo: is this upper or lower bound?
                parent_node.geometric_error_lower_bound = Some(geometric_error);
            }

            current_node = parent_node;
        }
    }

    println!("Node map: {:?}", node_map);

    // TODO: calculate root geometric error and export data back to tileset.json

    println!("Base directory: {:?}", base_dir);
    return Ok(());
}

fn load_tile_geometries(
    node: &TilesetNode,
    base_dir: &Path,
) -> Vec<Result<Geometry, TesseraError>> {
    // TODO: fix this extraction to make it cleaner
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

    return tile_content_uris
        .iter()
        .map(|uri| load_tile_geometry(base_dir, uri))
        .collect::<Vec<_>>();
}
