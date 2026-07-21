use crate::error::TesseraError;
use crate::geometry::Geometry;
use crate::geometry::compare::get_geometric_error_between_geometries;
use crate::maths::sphere::Sphere;
use crate::tile::load_tile_geometry_with_transform;
use crate::tileset::traverse::{TilesetNode, parse_tileset_nodes};
use crate::tileset::{Tile, Tileset};
use rayon::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

pub mod error;
pub mod geometry;
pub mod maths;
pub mod tile;
pub mod tileset;
pub mod utils;

pub const DEFAULT_GEOMETRY_CACHE_TILES: usize = 256;

pub fn calculate_geometric_error(
    tileset: &mut Tileset,
    base_dir: &Path,
) -> Result<(), TesseraError> {
    return calculate_geometric_error_with_cache_size(
        tileset,
        base_dir,
        DEFAULT_GEOMETRY_CACHE_TILES,
    );
}

pub fn calculate_geometric_error_with_cache_size(
    tileset: &mut Tileset,
    base_dir: &Path,
    cache_tiles: usize,
) -> Result<(), TesseraError> {
    let (mut node_map, root_id, leaf_ids) = parse_tileset_nodes(tileset);
    let geometry_cache = GeometryCache::new(base_dir, cache_tiles);

    // set all leaves to geometric error 0
    for leaf_id in &leaf_ids {
        let leaf_node = node_map.get_mut(&leaf_id).unwrap();
        leaf_node.geometric_error = Some(0.0);
    }

    let candidate_errors = calculate_candidate_errors(&leaf_ids, &node_map, &geometry_cache)?;
    apply_candidate_errors(candidate_errors, &mut node_map);

    debug!(?node_map, "Calculated tileset node map");

    // TODO: handle case where root has no content
    let root_geometry = geometry_cache.load(node_map.get(&root_id).unwrap())?;

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

fn calculate_candidate_errors(
    leaf_ids: &Vec<usize>,
    node_map: &HashMap<usize, TilesetNode>,
    geometry_cache: &GeometryCache,
) -> Result<HashMap<usize, f64>, TesseraError> {
    // TesseraError contains DracoStatus, which is not Send, so the parallel
    // iterator uses String errors internally and converts back at the boundary.
    return leaf_ids
        .par_iter()
        .map(|leaf_id| calculate_leaf_candidate_errors(*leaf_id, node_map, geometry_cache))
        .try_reduce(HashMap::new, |mut acc, candidate_errors| {
            merge_candidate_errors(&mut acc, candidate_errors);
            return Ok(acc);
        })
        .map_err(TesseraError::Processing);
}

fn calculate_leaf_candidate_errors(
    leaf_id: usize,
    node_map: &HashMap<usize, TilesetNode>,
    geometry_cache: &GeometryCache,
) -> Result<HashMap<usize, f64>, String> {
    let leaf_node = node_map.get(&leaf_id).unwrap();
    let leaf_geometries = geometry_cache
        .load(leaf_node)
        .map_err(|e| e.to_string())?;
    let leaf_geometry_refs = geometry_refs(&leaf_geometries);
    let mut current_id = leaf_id;
    let mut candidate_errors = HashMap::<usize, f64>::new();

    while let Some(parent) = node_map.get(&current_id).unwrap().parent_id {
        let parent_node = node_map.get(&parent).unwrap();
        let parent_geometries = geometry_cache
            .load(parent_node)
            .map_err(|e| e.to_string())?;
        let parent_geometry_refs = geometry_refs(&parent_geometries);

        let geometric_error =
            get_geometric_error_between_geometries(&leaf_geometry_refs, &parent_geometry_refs)
                .map_err(|e| e.to_string())?;

        candidate_errors
            .entry(parent)
            .and_modify(|existing| *existing = existing.max(geometric_error))
            .or_insert(geometric_error);
        current_id = parent;
    }

    return Ok(candidate_errors);
}

fn merge_candidate_errors(target: &mut HashMap<usize, f64>, source: HashMap<usize, f64>) {
    for (tile_id, geometric_error) in source {
        target
            .entry(tile_id)
            .and_modify(|existing| *existing = existing.max(geometric_error))
            .or_insert(geometric_error);
    }
}

fn apply_candidate_errors(
    candidate_errors: HashMap<usize, f64>,
    node_map: &mut HashMap<usize, TilesetNode>,
) {
    let total_tiles = node_map.len();
    let mut recalculated_tiles = 0usize;

    for (tile_id, geometric_error) in candidate_errors {
        let parent_node = node_map.get_mut(&tile_id).unwrap();

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
    }
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

struct GeometryCache {
    base_dir: PathBuf,
    max_tiles: usize,
    state: Mutex<GeometryCacheState>,
}

struct GeometryCacheState {
    geometries: HashMap<usize, Arc<Vec<Geometry>>>,
    lru: VecDeque<usize>,
}

impl GeometryCache {
    fn new(base_dir: &Path, max_tiles: usize) -> Self {
        return Self {
            base_dir: base_dir.to_path_buf(),
            max_tiles,
            state: Mutex::new(GeometryCacheState {
                geometries: HashMap::new(),
                lru: VecDeque::new(),
            }),
        };
    }

    fn load(&self, node: &TilesetNode) -> Result<Arc<Vec<Geometry>>, TesseraError> {
        if self.max_tiles > 0 {
            if let Some(geometries) = self.get(node.id) {
                return Ok(geometries);
            }
        }

        let geometries = Arc::new(load_tile_geometries(node, &self.base_dir)?);

        if self.max_tiles > 0 {
            self.insert(node.id, geometries.clone());
        }

        return Ok(geometries);
    }

    fn get(&self, tile_id: usize) -> Option<Arc<Vec<Geometry>>> {
        let mut state = self.state.lock().unwrap();
        let geometries = state.geometries.get(&tile_id)?.clone();
        touch_lru(&mut state.lru, tile_id);
        return Some(geometries);
    }

    fn insert(&self, tile_id: usize, geometries: Arc<Vec<Geometry>>) {
        let mut state = self.state.lock().unwrap();
        state.geometries.insert(tile_id, geometries);
        touch_lru(&mut state.lru, tile_id);

        while state.geometries.len() > self.max_tiles {
            if let Some(evicted_id) = state.lru.pop_front() {
                state.geometries.remove(&evicted_id);
            } else {
                break;
            }
        }

        debug!(
            cached_tiles = state.geometries.len(),
            max_cached_tiles = self.max_tiles,
            "Updated tile geometry cache"
        );
    }
}

fn touch_lru(lru: &mut VecDeque<usize>, tile_id: usize) {
    if let Some(index) = lru.iter().position(|id| *id == tile_id) {
        lru.remove(index);
    }

    lru.push_back(tile_id);
}

fn load_tile_geometries(
    node: &TilesetNode,
    base_dir: &Path,
) -> Result<Vec<Geometry>, TesseraError> {
    return node
        .content
        .iter()
        .map(|uri| load_tile_geometry_with_transform(base_dir, uri, &node.transform))
        .collect::<Result<Vec<_>, _>>();
}

fn geometry_refs(geometries: &Vec<Geometry>) -> Vec<&Geometry> {
    return geometries.iter().collect::<Vec<_>>();
}
