use crate::error::TesseraError;
use crate::geometry::Geometry;
use crate::geometry::compare::get_geometric_error_between_geometries;
use crate::maths::sphere::Sphere;
use crate::tile::load_tile_geometry_with_transform;
use crate::tileset::traverse::{TilesetNode, parse_tileset_nodes};
use crate::tileset::{Tile, Tileset};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

pub mod error;
pub mod geometry;
pub mod maths;
pub mod tile;
pub mod tileset;
pub mod utils;

pub fn calculate_geometric_error(
    tileset: &mut Tileset,
    base_dir: &Path,
) -> Result<(), TesseraError> {
    let (mut node_map, root_id, leaf_ids) = parse_tileset_nodes(tileset);
    let geometry_cache = load_tileset_geometries(&node_map, base_dir)?;

    // set all leaves to geometric error 0
    for leaf_id in &leaf_ids {
        let leaf_node = node_map.get_mut(&leaf_id).unwrap();
        leaf_node.geometric_error = Some(0.0);
    }

    let total_tiles = node_map.len();
    let mut recalculated_tiles = 0usize;

    // traverse tileset from leaves to root to set geometric error per parent node
    for leaf_id in &leaf_ids {
        let leaf_geometries = geometry_refs(&geometry_cache, *leaf_id)?;
        let mut current_id = *leaf_id;

        while let Some(parent) = node_map.get(&current_id).unwrap().parent_id {
            let parent_geometries = geometry_refs(&geometry_cache, parent)?;

            let geometric_error =
                get_geometric_error_between_geometries(&leaf_geometries, &parent_geometries)?;

            let parent_node = node_map.get_mut(&parent).unwrap();

            if parent_node.geometric_error.is_none()
                || parent_node.geometric_error.unwrap() < geometric_error
            {
                let was_uncalculated = parent_node.geometric_error.is_none();
                parent_node.geometric_error = Some(geometric_error);

                if was_uncalculated {
                    recalculated_tiles += 1;
                    let percent = (recalculated_tiles as f64 / total_tiles as f64) * 100.0;
                    info!(
                        tile_id = parent_node.id,
                        recalculated_tiles,
                        total_tiles,
                        percent = format_args!("{:.2}", percent),
                        original_geometric_error = parent_node.original_geometric_error,
                        calculated_geometric_error = geometric_error,
                        "Recalculated tile geometric error"
                    );
                } else {
                    debug!(
                        tile_id = parent_node.id,
                        original_geometric_error = parent_node.original_geometric_error,
                        calculated_geometric_error = geometric_error,
                        "Updated tile geometric error"
                    );
                }
            }

            current_id = parent;
        }
    }

    debug!(?node_map, "Calculated tileset node map");

    // TODO: handle case where root has no content
    let root_geometry = geometry_refs(&geometry_cache, root_id)?;

    let root_bounding_sphere = Sphere::from_points(
        &root_geometry
            .iter()
            .flat_map(|geom| &geom.primitives)
            .flat_map(|p| p.get_vertices())
            .collect(),
    );

    // copy across geometric error values to tileset
    set_tileset_geometric_error(tileset, &node_map)?;

    for node in node_map.values() {
        if let Some(calculated_geometric_error) = node.geometric_error {
            debug!(
                tile_id = node.id,
                original_geometric_error = node.original_geometric_error,
                calculated_geometric_error,
                "Tile geometric error result"
            );
        }
    }

    // use diameter for root tile geometric error as that's the closest we have to
    // error for not rendering the tileset at all
    tileset.geometric_error = Some(root_bounding_sphere.radius * 2.0);

    // TODO: implement debug timings, and perhaps try a quick profile to see if anything is obviously slow right now

    // TODO: add a validation step to ensure all tiles have a finite geometric error
    // as we will need to handle cases where a tile has no content and thus would have an infinite
    // shortest distance from above.

    debug!(?tileset, "Calculated tileset geometric errors");

    return Ok(());
}

fn set_tileset_geometric_error(
    tileset: &mut Tileset,
    node_map: &HashMap<usize, TilesetNode>,
) -> Result<(), TesseraError> {
    fn traverse_and_set(
        node: &mut Tile,
        node_map: &HashMap<usize, TilesetNode>,
    ) -> Result<(), TesseraError> {
        let tileset_node = node_map.get(&node.id).ok_or(TesseraError::Tileset(format!(
            "Tileset node not found for tile {}",
            node.id
        )))?;

        let Some(calculated_geometric_error) = tileset_node.geometric_error else {
            return Err(TesseraError::Tileset(format!(
                "Geometric error not found for tile {}",
                node.id
            )));
        };

        node.geometric_error = calculated_geometric_error;

        for child in &mut node.children {
            traverse_and_set(child, node_map)?;
        }

        return Ok(());
    }

    return traverse_and_set(&mut tileset.root, &node_map);
}

fn load_tileset_geometries(
    node_map: &HashMap<usize, TilesetNode>,
    base_dir: &Path,
) -> Result<HashMap<usize, Vec<Geometry>>, TesseraError> {
    let mut geometry_cache = HashMap::<usize, Vec<Geometry>>::new();

    for node in node_map.values() {
        let geometries = node
            .content
            .iter()
            .map(|uri| load_tile_geometry_with_transform(base_dir, uri, &node.transform))
            .collect::<Result<Vec<_>, _>>()?;

        geometry_cache.insert(node.id, geometries);
    }

    debug!(tiles = geometry_cache.len(), "Loaded tile geometry cache");

    return Ok(geometry_cache);
}

fn geometry_refs(
    geometry_cache: &HashMap<usize, Vec<Geometry>>,
    tile_id: usize,
) -> Result<Vec<&Geometry>, TesseraError> {
    return geometry_cache
        .get(&tile_id)
        .map(|geometries| geometries.iter().collect::<Vec<_>>())
        .ok_or_else(|| TesseraError::Tileset(format!("Geometry not found for tile {tile_id}")));
}
