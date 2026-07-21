use crate::error::TesseraError;
use crate::geometry::Geometry;
use crate::geometry::compare::get_geometric_error_between_prepared_tile_geometries;
use crate::geometry::prepared::{PreparedPrimitive, PreparedTileGeometry};
use crate::maths::sphere::Sphere;
use crate::tile::load_tile_geometry_with_transform;
use crate::tileset::traverse::{TilesetNode, parse_tileset_nodes};
use crate::tileset::{Tile, Tileset};
use rayon::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
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
    let total_start = Instant::now();

    let parse_start = Instant::now();
    let (mut node_map, root_id, leaf_ids) = parse_tileset_nodes(tileset);
    info!(
        elapsed_ms = parse_start.elapsed().as_millis(),
        tiles = node_map.len(),
        leaves = leaf_ids.len(),
        "Parsed tileset nodes"
    );

    let geometry_cache = GeometryCache::new(base_dir, cache_tiles);

    // set all leaves to geometric error 0
    for leaf_id in &leaf_ids {
        let leaf_node = node_map.get_mut(&leaf_id).unwrap();
        leaf_node.geometric_error = Some(0.0);
    }

    let candidate_start = Instant::now();
    let candidate_errors = calculate_candidate_errors(&leaf_ids, &node_map, &geometry_cache)?;
    info!(
        elapsed_ms = candidate_start.elapsed().as_millis(),
        "Calculated candidate errors"
    );

    let apply_start = Instant::now();
    apply_candidate_errors(candidate_errors, &mut node_map);
    info!(
        elapsed_ms = apply_start.elapsed().as_millis(),
        "Applied candidate errors"
    );

    debug!(?node_map, "Calculated tileset node map");

    // TODO: handle case where root has no content
    let root_geometry_start = Instant::now();
    let root_geometry = geometry_cache.load(node_map.get(&root_id).unwrap())?;
    info!(
        elapsed_ms = root_geometry_start.elapsed().as_millis(),
        root_id, "Loaded root tile geometry"
    );

    let root_bounding_sphere_start = Instant::now();
    let root_bounding_sphere = Sphere::from_points(
        &root_geometry
            .geometries
            .iter()
            .flat_map(|geom| &geom.primitives)
            .flat_map(prepared_primitive_vertices)
            .collect(),
    );
    info!(
        elapsed_ms = root_bounding_sphere_start.elapsed().as_millis(),
        "Calculated root bounding sphere"
    );

    // copy across geometric error values to tileset
    let set_start = Instant::now();
    set_tileset_geometric_error(tileset, &node_map)?;
    info!(
        elapsed_ms = set_start.elapsed().as_millis(),
        "Copied geometric errors to tileset"
    );

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

    info!(
        elapsed_ms = total_start.elapsed().as_millis(),
        "Calculated tileset geometric errors"
    );

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
    let progress = CandidateErrorProgress::new(leaf_ids.len());
    info!(
        leaves = leaf_ids.len(),
        tiles = node_map.len(),
        "Started candidate error calculation"
    );

    // TesseraError contains DracoStatus, which is not Send, so the parallel
    // iterator uses String errors internally and converts back at the boundary.
    return leaf_ids
        .par_iter()
        .map(|leaf_id| {
            calculate_leaf_candidate_errors(*leaf_id, node_map, geometry_cache, &progress)
        })
        .try_reduce(HashMap::new, |mut acc, candidate_errors| {
            merge_candidate_errors(&mut acc, candidate_errors);
            return Ok(acc);
        })
        .map_err(TesseraError::Processing);
}

struct CandidateErrorProgress {
    total_leaves: usize,
    completed_leaves: AtomicUsize,
    completed_comparisons: AtomicUsize,
    start: Instant,
}

impl CandidateErrorProgress {
    fn new(total_leaves: usize) -> Self {
        Self {
            total_leaves,
            completed_leaves: AtomicUsize::new(0),
            completed_comparisons: AtomicUsize::new(0),
            start: Instant::now(),
        }
    }

    fn record_comparison(&self, leaf_id: usize, parent_id: usize, geometric_error: f64) {
        let completed = self.completed_comparisons.fetch_add(1, Ordering::Relaxed) + 1;

        if completed == 1 || completed % 500 == 0 {
            info!(
                completed_comparisons = completed,
                completed_leaves = self.completed_leaves.load(Ordering::Relaxed),
                total_leaves = self.total_leaves,
                elapsed_ms = self.start.elapsed().as_millis(),
                leaf_id,
                parent_id,
                geometric_error,
                "Candidate error comparison progress"
            );
        }
    }

    fn record_leaf_complete(&self, leaf_id: usize, comparisons_for_leaf: usize) {
        let completed = self.completed_leaves.fetch_add(1, Ordering::Relaxed) + 1;
        let interval = (self.total_leaves / 100).max(1);

        if completed == 1 || completed == self.total_leaves || completed % interval == 0 {
            let percent = if self.total_leaves == 0 {
                100.0
            } else {
                (completed as f64 / self.total_leaves as f64) * 100.0
            };
            info!(
                completed_leaves = completed,
                total_leaves = self.total_leaves,
                percent = format_args!("{:.2}", percent),
                completed_comparisons = self.completed_comparisons.load(Ordering::Relaxed),
                comparisons_for_leaf,
                elapsed_ms = self.start.elapsed().as_millis(),
                leaf_id,
                "Candidate error leaf progress"
            );
        }
    }
}

fn calculate_leaf_candidate_errors(
    leaf_id: usize,
    node_map: &HashMap<usize, TilesetNode>,
    geometry_cache: &GeometryCache,
    progress: &CandidateErrorProgress,
) -> Result<HashMap<usize, f64>, String> {
    let leaf_node = node_map.get(&leaf_id).unwrap();
    let leaf_geometries = geometry_cache.load(leaf_node).map_err(|e| e.to_string())?;
    let mut current_id = leaf_id;
    let mut candidate_errors = HashMap::<usize, f64>::new();
    let mut comparisons_for_leaf = 0usize;
    let leaf_start = Instant::now();

    while let Some(parent) = node_map.get(&current_id).unwrap().parent_id {
        let parent_node = node_map.get(&parent).unwrap();
        let parent_geometries = geometry_cache
            .load(parent_node)
            .map_err(|e| e.to_string())?;
        let leaf_summary = prepared_tile_geometry_summary(&leaf_geometries);
        let parent_summary = prepared_tile_geometry_summary(&parent_geometries);
        let estimated_work = leaf_summary.total_elements() * parent_summary.total_elements();

        if estimated_work >= 5_000_000 {
            info!(
                leaf_id,
                parent_id = parent,
                leaf_content = ?leaf_node.content,
                parent_content = ?parent_node.content,
                leaf_geometries = leaf_summary.geometries,
                leaf_primitives = leaf_summary.primitives,
                leaf_points = leaf_summary.points,
                leaf_lines = leaf_summary.lines,
                leaf_triangles = leaf_summary.triangles,
                parent_geometries = parent_summary.geometries,
                parent_primitives = parent_summary.primitives,
                parent_points = parent_summary.points,
                parent_lines = parent_summary.lines,
                parent_triangles = parent_summary.triangles,
                estimated_work,
                "Starting large tile geometry comparison"
            );
        }

        let comparison_start = Instant::now();
        let geometric_error = get_geometric_error_between_prepared_tile_geometries(
            &leaf_geometries,
            &parent_geometries,
        )
        .map_err(|e| e.to_string())?;
        let comparison_elapsed = comparison_start.elapsed();
        debug!(
            elapsed_ms = comparison_elapsed.as_millis(),
            leaf_id,
            parent_id = parent,
            geometric_error,
            "Compared tile geometries"
        );
        if comparison_elapsed >= Duration::from_secs(10) {
            info!(
                elapsed_ms = comparison_elapsed.as_millis(),
                leaf_id,
                parent_id = parent,
                leaf_content = ?leaf_node.content,
                parent_content = ?parent_node.content,
                leaf_geometries = leaf_summary.geometries,
                leaf_primitives = leaf_summary.primitives,
                leaf_points = leaf_summary.points,
                leaf_lines = leaf_summary.lines,
                leaf_triangles = leaf_summary.triangles,
                parent_geometries = parent_summary.geometries,
                parent_primitives = parent_summary.primitives,
                parent_points = parent_summary.points,
                parent_lines = parent_summary.lines,
                parent_triangles = parent_summary.triangles,
                estimated_work,
                geometric_error,
                "Slow tile geometry comparison completed"
            );
        }
        comparisons_for_leaf += 1;
        progress.record_comparison(leaf_id, parent, geometric_error);

        candidate_errors
            .entry(parent)
            .and_modify(|existing| *existing = existing.max(geometric_error))
            .or_insert(geometric_error);
        current_id = parent;
    }

    debug!(
        elapsed_ms = leaf_start.elapsed().as_millis(),
        leaf_id, comparisons_for_leaf, "Calculated leaf candidate errors"
    );
    progress.record_leaf_complete(leaf_id, comparisons_for_leaf);

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
    geometries: HashMap<usize, Arc<PreparedTileGeometry>>,
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

    fn load(&self, node: &TilesetNode) -> Result<Arc<PreparedTileGeometry>, TesseraError> {
        if self.max_tiles > 0 {
            if let Some(geometries) = self.get(node.id) {
                debug!(tile_id = node.id, "Tile geometry cache hit");
                return Ok(geometries);
            }
        }

        debug!(tile_id = node.id, "Tile geometry cache miss");
        let load_start = Instant::now();
        let decoded_geometries = load_tile_geometries(node, &self.base_dir)?;
        let geometries = Arc::new(PreparedTileGeometry::from_geometries(&decoded_geometries));
        debug!(
            elapsed_ms = load_start.elapsed().as_millis(),
            tile_id = node.id,
            geometries = geometries.geometries.len(),
            "Loaded and prepared tile geometry"
        );

        if self.max_tiles > 0 {
            self.insert(node.id, geometries.clone());
        }

        return Ok(geometries);
    }

    fn get(&self, tile_id: usize) -> Option<Arc<PreparedTileGeometry>> {
        let mut state = self.state.lock().unwrap();
        let geometries = state.geometries.get(&tile_id)?.clone();
        touch_lru(&mut state.lru, tile_id);
        return Some(geometries);
    }

    fn insert(&self, tile_id: usize, geometries: Arc<PreparedTileGeometry>) {
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

#[derive(Debug, Default, Clone, Copy)]
struct PreparedTileGeometrySummary {
    geometries: usize,
    primitives: usize,
    points: usize,
    lines: usize,
    triangles: usize,
}

impl PreparedTileGeometrySummary {
    fn total_elements(&self) -> usize {
        self.points + self.lines + self.triangles
    }
}

fn prepared_tile_geometry_summary(geometry: &PreparedTileGeometry) -> PreparedTileGeometrySummary {
    let mut summary = PreparedTileGeometrySummary {
        geometries: geometry.geometries.len(),
        ..Default::default()
    };

    for geometry in &geometry.geometries {
        summary.primitives += geometry.primitives.len();

        for primitive in &geometry.primitives {
            match primitive {
                PreparedPrimitive::Points(primitive) => summary.points += primitive.points.len(),
                PreparedPrimitive::Lines(primitive) => summary.lines += primitive.lines.len(),
                PreparedPrimitive::Triangles(primitive) => {
                    summary.triangles += primitive.triangles.len()
                }
            }
        }
    }

    summary
}

fn prepared_primitive_vertices(primitive: &PreparedPrimitive) -> Vec<&[f32; 3]> {
    match primitive {
        PreparedPrimitive::Points(primitive) => primitive.points.iter().collect(),
        PreparedPrimitive::Lines(primitive) => {
            primitive.lines.iter().flat_map(|(a, b)| [a, b]).collect()
        }
        PreparedPrimitive::Triangles(primitive) => primitive
            .triangles
            .iter()
            .flat_map(|(a, b, c)| [a, b, c])
            .collect(),
    }
}
