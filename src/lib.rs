use crate::error::TesseraError;
use crate::geometry::Geometry;
use crate::geometry::compare::get_geometric_error_between_geometries;
use crate::maths::sphere::Sphere;
use crate::tile::load_tile_geometry;
use crate::tileset::traverse::{TilesetNode, parse_tileset_nodes};
use crate::tileset::{Tile, Tileset};
use std::collections::HashMap;
use std::path::Path;

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

    // set all leaves to geometric error 0
    for leaf_id in &leaf_ids {
        let leaf_node = node_map.get_mut(&leaf_id).unwrap();
        leaf_node.geometric_error = Some(0.0);
    }

    // traverse tileset from leaves to root to set geometric error per parent node
    for leaf_id in &leaf_ids {
        let leaf_node = node_map.get(&leaf_id).unwrap();

        let mut current_node = leaf_node;

        // TODO: handle error case
        let leaf_geometry_results = load_tile_geometries(&leaf_node, base_dir);
        let leaf_geometries = leaf_geometry_results
            .iter()
            .map(|r| r.as_ref().unwrap())
            .collect::<Vec<_>>();

        while let Some(parent) = current_node.parent_id {
            let parent_node = node_map.get_mut(&parent).unwrap();

            // TODO: handle error case
            let parent_geometries_results = load_tile_geometries(&parent_node, base_dir);
            let parent_geometries = parent_geometries_results
                .iter()
                .map(|r| r.as_ref().unwrap())
                .collect::<Vec<_>>();

            let geometric_error_result =
                get_geometric_error_between_geometries(&leaf_geometries, &parent_geometries);

            if geometric_error_result.is_err() {
                return Err(geometric_error_result.err().unwrap());
            }

            let geometric_error = geometric_error_result.unwrap();

            if parent_node.geometric_error.is_none()
                || parent_node.geometric_error.unwrap() < geometric_error
            {
                parent_node.geometric_error = Some(geometric_error);
            }

            current_node = parent_node;
        }
    }

    println!("Node map: {:?}", node_map);

    // TODO: handle case where root has no content
    let root_geometry = load_tile_geometries(&node_map.get(&root_id).unwrap(), base_dir);

    let root_geometry = root_geometry
        .iter()
        .map(|r| r.as_ref().unwrap())
        .collect::<Vec<_>>();

    let root_bounding_sphere = Sphere::from_points(
        &root_geometry
            .iter()
            .flat_map(|geom| &geom.primitives)
            .flat_map(|p| p.get_vertices())
            .collect(),
    );

    // copy across geometric error values to tileset
    set_tileset_geometric_error(tileset, &node_map)?;

    // use diameter for root tile geometric error as that's the closest we have to
    // error for not rendering the tileset at all
    tileset.root.geometric_error = root_bounding_sphere.radius * 2.0;

    // TODO: implement debug timings, and perhaps try a quick profile to see if anything is obviously slow right now

    // TODO: add a validation step to ensure all tiles have a finite geometric error
    // as we will need to handle cases where a tile has no content and thus would have an infinite
    // shortest distance from above.

    println!("Tileset: {:?}", tileset);

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

fn load_tile_geometries(
    node: &TilesetNode,
    base_dir: &Path,
) -> Vec<Result<Geometry, TesseraError>> {
    return node
        .content
        .iter()
        .map(|uri| load_tile_geometry(base_dir, uri))
        .collect::<Vec<_>>();
}
