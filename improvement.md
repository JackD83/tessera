# Tessera Performance Improvement Plan

This plan focuses on improving runtime while preserving the same calculated geometric error values. Optimizations should be introduced in small, measurable phases and validated against the current implementation after every phase.

## Goals

1. Preserve calculated values.
   - No approximate nearest-neighbor search.
   - No heuristic pruning.
   - Only exact or conservative optimizations.
2. Reduce runtime.
   - Avoid repeated decoding/reloading.
   - Reduce allocation in hot loops.
   - Reduce dynamic dispatch in primitive iteration.
   - Add exact spatial acceleration for expensive comparisons.
3. Make performance measurable.
   - Add timings and benchmarks.
   - Compare old and new outputs on representative tilesets.

## Implementation Checklist

Use this checklist to track progress. Each phase should be validated before moving to the next phase.

### Phase 0: Baseline and Safety Checks

- [x] Add repeatable release-mode benchmark workflow.
- [ ] Select representative benchmark tilesets.
- [x] Add output comparison helper for parsed tileset JSON.
- [x] Define acceptable numeric tolerance for geometric error comparison.
- [x] Add timing instrumentation around major processing stages.
- [ ] Record baseline runtime, memory, and output values before optimization.

### Phase 1: Build and Runtime Configuration

- [x] Document `cargo build --release` usage.
- [x] Document `RUSTFLAGS="-C target-cpu=native" cargo build --release` usage.
- [x] Document larger `--cache-tiles` values for speed when memory allows.
- [x] Document `RAYON_NUM_THREADS` tuning.
- [ ] Benchmark at least three thread-count settings.

### Phase 2: Remove Hot-Loop Allocations

- [x] Refactor `get_renderable_delta_between_lines` to avoid `closest_representations` allocation.
- [x] Refactor `get_renderable_delta_between_line_and_triangle` to avoid `closest_representations` allocation.
- [x] Refactor `get_renderable_delta_between_triangle_and_line` to avoid `closest_representations` allocation.
- [x] Refactor `get_renderable_delta_between_triangles` to avoid `closest_representations` allocation.
- [x] Preserve exact tie handling in all four functions.
- [x] Add or update equivalence tests for the refactored delta functions.
- [ ] Compare old and new output values on representative tilesets.
- [ ] Record runtime improvement.

### Phase 3: Reduce Dynamic Iterator Overhead

- [x] Identify all hot paths using `Box<dyn Iterator>` primitive iterators.
- [x] Decide whether to replace boxed iterators directly or defer to prepared geometry.
- [x] If replacing directly, add static/custom iterator types.
- [x] Validate that vertex/index interpretation is unchanged.
- [ ] Compare old and new output values.
- [ ] Record runtime improvement.

### Phase 4: Prepared Geometry Cache

- [x] Add `PreparedTileGeometry` type.
- [x] Add `PreparedGeometry` type.
- [x] Add `PreparedPrimitive` enum.
- [x] Add prepared point primitive representation.
- [x] Add prepared line primitive representation.
- [x] Add prepared triangle primitive representation.
- [x] Implement conversion from existing `Geometry` to prepared geometry.
- [x] Preserve vertex order, repeated indices, and incomplete primitive chunk behavior.
- [x] Change `GeometryCache` to cache prepared geometry.
- [x] Update comparison functions to consume prepared geometry.
- [x] Add tests comparing prepared geometry output against current iterator output.
- [ ] Compare old and new output values on representative tilesets.
- [ ] Record runtime improvement.

### Phase 5: Conservative Bounding Sphere Pruning

- [x] Add primitive bounding sphere accessors if needed.
- [x] Implement minimum distance between bounding spheres.
- [x] Add conservative primitive-pair pruning in geometry comparison.
- [x] Ensure pruning only happens when the pair cannot improve the current best result.
- [x] Add tests for pruned and non-pruned primitive pairs.
- [ ] Compare old and new output values.
- [ ] Record number of skipped primitive comparisons.
- [ ] Record runtime improvement.

### Phase 6: Exact KD-tree for Point Comparisons

- [ ] Choose KD-tree dependency, e.g. `kiddo` or `rstar`.
- [x] Add KD-tree/index field to prepared point primitive.
- [x] Build point index once during geometry preparation.
- [x] Replace brute-force point-to-point parent scan with exact nearest-neighbor lookup.
- [x] Ensure exact nearest search is used, not approximate search.
- [x] Add brute-force vs KD-tree unit tests.
- [x] Test fixed-seed random point clouds.
- [x] Test repeated indices and duplicate points.
- [ ] Compare old and new output values.
- [ ] Record runtime improvement for point-heavy tilesets.

### Phase 7: Exact Spatial Index for Lines and Triangles

- [ ] Choose spatial index dependency, e.g. `rstar` or a BVH crate.
- [x] Add AABB representation for indexed lines.
- [x] Add AABB representation for indexed triangles.
- [x] Build line spatial index during geometry preparation.
- [x] Build triangle spatial index during geometry preparation.
- [x] Implement exact lower-bound ordered candidate traversal.
- [x] Stop traversal only when remaining candidates cannot improve the exact best distance.
- [x] Preserve tie handling by continuing while lower bounds can tie the current closest distance.
- [x] Apply indexing to point-to-line comparison.
- [x] Apply indexing to point-to-triangle comparison.
- [x] Apply indexing to line-to-line comparison.
- [x] Apply indexing to line-to-triangle comparison.
- [x] Apply indexing to triangle-to-line comparison.
- [x] Apply indexing to triangle-to-triangle comparison.
- [x] Add brute-force vs indexed unit tests for all affected comparison types.
- [x] Test degenerate lines and triangles.
- [ ] Compare old and new output values.
- [ ] Record runtime improvement for mesh-heavy tilesets.

### Phase 8: Parallelism and Cache Contention

- [ ] Profile cache lock contention.
- [ ] Decide whether current `Mutex` cache is sufficient.
- [ ] If needed, add concurrent cache dependency such as `moka`.
- [ ] Replace single-lock cache with concurrent cache.
- [ ] Avoid duplicate tile loads on concurrent cache misses.
- [ ] Validate cache capacity behavior.
- [ ] Compare old and new output values.
- [ ] Benchmark scalability across thread counts.

### Phase 9: Optional CLI Improvements

- [x] Add optional `--threads` CLI flag.
- [x] Configure Rayon global thread pool before first Rayon use.
- [x] Add optional `--timings` CLI flag if timing logs should be user-controlled.
- [x] Add profiling guidance to README.
- [x] Document flamegraph usage.

### Final Validation

- [x] Run `cargo fmt`.
- [x] Run `cargo clippy` if configured.
- [x] Run `cargo test`.
- [x] Build with `cargo build --release`.
- [ ] Compare optimized output against baseline output on all benchmark tilesets.
- [ ] Confirm no intentional value changes were introduced.
- [ ] Record final performance numbers.
- [ ] Update README with measured recommendations.

## Phase 0: Baseline and Safety Checks

### 0.1 Add a repeatable benchmark workflow

Build and run in release mode:

```sh
cargo build --release
time ./target/release/tessera recalculate -i input/tileset.json -o output.json
```

Benchmark at least:

- Small tileset
- Medium tileset
- Large production-like tileset
- Point-heavy tileset
- Triangle-heavy tileset
- Mixed geometry tileset

### 0.2 Add an output comparison helper

Compare parsed JSON rather than raw files. Verify:

- Tile structure matches.
- Every tile `geometricError` matches within tolerance.
- Root `geometricError` matches within tolerance.

Recommended tolerance:

```text
absolute difference <= 1e-9
```

### 0.3 Add timing instrumentation

Add optional timing logs around:

- Tileset loading
- Node parsing
- Tile geometry loading
- Candidate error calculation
- Geometry comparison
- Output writing

Example:

```rust
let start = std::time::Instant::now();
// work
info!(elapsed_ms = start.elapsed().as_millis(), "Calculated candidate errors");
```

## Phase 1: Build and Runtime Configuration

### 1.1 Document release-mode usage

Recommend:

```sh
cargo build --release
```

For local CPU-specific optimization:

```sh
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### 1.2 Tune geometry cache usage

Current default:

```rust
pub const DEFAULT_GEOMETRY_CACHE_TILES: usize = 256;
```

Document use of larger cache sizes when memory allows:

```sh
tessera recalculate -i tileset.json -o out.json --cache-tiles 5000
```

This should preserve values because it only changes caching behavior.

### 1.3 Document Rayon thread tuning

Users can benchmark different thread counts:

```sh
RAYON_NUM_THREADS=8 tessera recalculate -i tileset.json -o out.json
```

Try values such as:

```text
4, 8, 16, physical_core_count
```

## Phase 2: Remove Hot-Loop Allocations

### 2.1 Remove `closest_representations` vectors

In `src/geometry/delta.rs`, these functions allocate temporary vectors inside hot loops:

- `get_renderable_delta_between_lines`
- `get_renderable_delta_between_line_and_triangle`
- `get_renderable_delta_between_triangle_and_line`
- `get_renderable_delta_between_triangles`

Current pattern:

```rust
let mut closest_representations: Vec<...> = vec![];
let mut closest_distance = f64::INFINITY;

for candidate in parent.iter_vertices() {
    let distance = ...;

    if distance < closest_distance {
        closest_representations = vec![candidate];
        closest_distance = distance;
    } else if distance == closest_distance {
        closest_representations.push(candidate);
    }
}

let mut min_renderable_delta = f64::INFINITY;
for candidate in closest_representations {
    let renderable_delta = ...;
    min_renderable_delta = min_renderable_delta.min(renderable_delta);
}
```

Replace with direct tracking:

```rust
let mut closest_distance = f64::INFINITY;
let mut min_renderable_delta_for_closest = f64::INFINITY;

for candidate in parent.iter_vertices() {
    let distance = ...;

    if distance < closest_distance {
        closest_distance = distance;
        min_renderable_delta_for_closest = ...;
    } else if distance == closest_distance {
        let renderable_delta = ...;
        min_renderable_delta_for_closest =
            min_renderable_delta_for_closest.min(renderable_delta);
    }
}
```

This preserves tie handling while avoiding repeated allocations.

### 2.2 Validate

Run old and new implementations on representative tilesets. Expected result:

- Same `geometricError` values.
- Less allocation pressure.
- Small to moderate speedup.

## Phase 3: Reduce Dynamic Iterator Overhead

Current primitive iterators return boxed trait objects:

```rust
pub fn iter_vertices(&self) -> Box<dyn Iterator<Item = ...> + '_>
```

This causes heap allocation and dynamic dispatch in hot loops.

Short-term option:

- Replace boxed iterators with custom enum/static iterators.

Preferred option:

- Introduce prepared geometry in Phase 4 and resolve indices once.

## Phase 4: Prepared Geometry Cache

Instead of caching only decoded `Geometry`, cache a processed representation optimized for comparison.

### 4.1 Introduce prepared geometry types

Example structure:

```rust
pub struct PreparedTileGeometry {
    pub geometries: Vec<PreparedGeometry>,
}

pub struct PreparedGeometry {
    pub name: String,
    pub primitives: Vec<PreparedPrimitive>,
}

pub enum PreparedPrimitive {
    Points(PreparedPointPrimitive),
    Lines(PreparedLinePrimitive),
    Triangles(PreparedTrianglePrimitive),
}

pub struct PreparedPointPrimitive {
    pub points: Vec<[f32; 3]>,
    pub bounding_sphere: Sphere,
}

pub struct PreparedLinePrimitive {
    pub lines: Vec<([f32; 3], [f32; 3])>,
    pub bounding_sphere: Sphere,
}

pub struct PreparedTrianglePrimitive {
    pub triangles: Vec<([f32; 3], [f32; 3], [f32; 3])>,
    pub bounding_sphere: Sphere,
}
```

### 4.2 Change cache storage

Current cache:

```rust
HashMap<usize, Arc<Vec<Geometry>>>
```

Target cache:

```rust
HashMap<usize, Arc<PreparedTileGeometry>>
```

### 4.3 Convert from existing geometry

Add conversion logic that exactly preserves current vertex/index interpretation.

Important rules:

- Preserve vertex order.
- Preserve index behavior.
- Preserve repeated indices.
- Drop incomplete line/triangle chunks exactly as current iterators do.

### 4.4 Update comparison functions

Current:

```rust
get_geometric_error_between_geometries(
    geometries: &Vec<&Geometry>,
    parent_geometries: &Vec<&Geometry>,
)
```

Target:

```rust
get_geometric_error_between_prepared_geometries(
    geometries: &[PreparedGeometry],
    parent_geometries: &[PreparedGeometry],
)
```

### 4.5 Validate

Expected result:

- Same calculated values.
- Moderate speedup.
- Cleaner foundation for spatial indexing.

## Phase 5: Conservative Bounding Sphere Pruning

Each primitive already has a bounding sphere. Use it to skip primitive pairs that cannot improve the current best result.

### 5.1 Add sphere lower-bound distance

Implement:

```rust
fn min_distance_between_spheres(a: &Sphere, b: &Sphere) -> f64 {
    let center_distance = ...;
    let min_distance = center_distance - a.radius - b.radius;
    min_distance.max(0.0)
}
```

Use squared distance if possible to avoid square roots.

### 5.2 Apply pruning in geometry comparison

Before comparing primitive pairs:

```rust
let lower_bound = min_distance_between_spheres(
    primitive.bounding_sphere(),
    parent_primitive.bounding_sphere(),
);

if lower_bound > shortest_distance {
    continue;
}
```

This is safe because the pair cannot improve the current best distance.

### 5.3 Correctness rule

Safe:

```rust
if lower_bound > current_best {
    skip
}
```

Unsafe:

```rust
if lower_bound seems large {
    skip
}
```

Only conservative pruning is allowed.

## Phase 6: Exact KD-tree for Point Comparisons

This is the first major algorithmic optimization.

### 6.1 Add dependency

Recommended:

```toml
kiddo = "5"
```

Alternative:

```toml
rstar = "0.12"
```

### 6.2 Add KD-tree to prepared point primitive

```rust
pub struct PreparedPointPrimitive {
    pub points: Vec<[f32; 3]>,
    pub bounding_sphere: Sphere,
    pub kdtree: Option<...>,
}
```

Build the KD-tree once when preparing geometry.

### 6.3 Optimize point-to-point comparison

Current brute-force scan:

```rust
for a_point in leaf.iter_vertices() {
    let mut closest_distance = f64::INFINITY;

    for b_point in parent.iter_vertices() {
        let distance = point_distance_squared(a_point, b_point);
        closest_distance = closest_distance.min(distance);
    }

    max_renderable_delta = max_renderable_delta.max(closest_distance);
}
```

Replace the parent scan with an exact nearest-neighbor query:

```rust
for a_point in &leaf.points {
    let closest_distance = parent.kdtree.nearest_one(a_point);
    max_renderable_delta = max_renderable_delta.max(closest_distance);
}
```

Use exact nearest search only.

### 6.4 Validate

Add unit tests comparing:

- Brute-force point comparison
- KD-tree point comparison

Use fixed-seed random point clouds and edge cases.

## Phase 7: Exact Spatial Index for Lines and Triangles

This is more complex than point indexing.

### 7.1 Use R-tree or BVH

Recommended:

```toml
rstar = "0.12"
```

Represent each line or triangle by an AABB:

```rust
struct IndexedLine {
    index: usize,
    aabb: AABB<[f64; 3]>,
}

struct IndexedTriangle {
    index: usize,
    aabb: AABB<[f64; 3]>,
}
```

### 7.2 Use the spatial index conservatively

Do not query only the nearest N candidates. That would be approximate.

Exact approach:

1. Keep current brute-force implementation as fallback.
2. Traverse candidates ordered by AABB lower-bound distance.
3. Compute exact distance for each candidate.
4. Stop only when the next candidate lower bound is greater than the current best exact distance.

Pseudo-code:

```text
current_best = infinity

for candidate in candidates_ordered_by_aabb_lower_bound:
    if candidate.lower_bound > current_best:
        break

    exact_distance = exact_distance_to_candidate(...)
    current_best = min(current_best, exact_distance)
```

### 7.3 Apply to expensive combinations

Highest priority:

- Point-to-triangle
- Point-to-line
- Line-to-line
- Line-to-triangle
- Triangle-to-line
- Triangle-to-triangle

### 7.4 Preserve tie handling

For functions that first find the closest representation and then calculate renderable delta, ties must still be considered.

Safe rule:

```text
continue processing candidates while lower_bound <= closest_distance
```

Do not stop on:

```text
lower_bound >= closest_distance
```

because exact ties may still matter.

### 7.5 Validate

Compare brute force vs indexed comparison for:

- Points
- Lines
- Triangles
- Mixed primitive types
- Degenerate geometry
- Equal-distance ties
- Empty primitives

## Phase 8: Parallelism and Cache Contention

Current parallelism is leaf-based:

```rust
leaf_ids.par_iter()
```

This is reasonable, but the current cache has a single `Mutex`:

```rust
state: Mutex<GeometryCacheState>
```

This can serialize cache access across worker threads.

### 8.1 Consider a concurrent cache

Recommended crate:

```toml
moka = { version = "0.12", features = ["sync"] }
```

`moka` supports concurrent cache access and capacity limits.

Possible target:

```rust
Cache<usize, Arc<PreparedTileGeometry>>
```

### 8.2 Prevent duplicate loads

Current behavior may duplicate work:

1. Thread A misses tile X.
2. Thread B misses tile X.
3. Both load tile X.
4. Both insert tile X.

A concurrent cache with `get_with`-style loading can avoid this.

### 8.3 Validate

Expected result:

- Same values.
- Better scalability on many-core machines.

## Phase 9: Optional CLI Improvements

### 9.1 Add performance-related CLI flags

Possible flags:

```sh
--cache-tiles 5000
--threads 8
--timings
```

A `--threads` flag can configure Rayon explicitly:

```rust
rayon::ThreadPoolBuilder::new()
    .num_threads(threads)
    .build_global()
```

This must happen before Rayon is first used.

### 9.2 Add profiling guidance

Document flamegraph usage:

```sh
cargo install flamegraph
cargo flamegraph -- recalculate -i tileset.json -o out.json
```

## Suggested Milestones

### Milestone 1: Low risk, exact

1. Add timing instrumentation.
2. Add output comparison tests.
3. Document release build and cache tuning.
4. Remove `closest_representations` allocations.

Expected result:

- Same values.
- Small to moderate speedup.

### Milestone 2: Prepared geometry

1. Add `PreparedGeometry`.
2. Resolve indices once.
3. Cache prepared geometry.
4. Replace boxed iterators in comparison code.

Expected result:

- Same values.
- Moderate speedup.
- Better foundation for spatial indexing.

### Milestone 3: Conservative pruning

1. Add primitive bounding sphere accessors.
2. Add sphere lower-bound pruning.
3. Validate against brute force.

Expected result:

- Same values.
- Good speedup on spatially separated geometry.

### Milestone 4: KD-tree for points

1. Add KD-tree to `PreparedPointPrimitive`.
2. Optimize point-to-point exact nearest search.
3. Add brute-force comparison tests.

Expected result:

- Same values.
- Large speedup for point clouds.

### Milestone 5: R-tree/BVH for lines and triangles

1. Add AABB spatial indices.
2. Implement exact candidate traversal.
3. Preserve tie handling.
4. Add extensive brute-force equivalence tests.

Expected result:

- Same values.
- Potentially very large speedup for dense meshes.

### Milestone 6: Concurrent cache

1. Replace `Mutex` LRU cache with `moka` or another concurrent cache.
2. Prevent duplicate tile loads.
3. Benchmark thread scalability.

Expected result:

- Same values.
- Better multicore performance.

## Correctness Test Plan

For every milestone, run:

```sh
cargo test
cargo build --release
```

Then compare old and new Tessera output on known tilesets.

### Unit tests to add

Point tests:

- Simple identical points
- Offset points
- Repeated indices
- Empty parent
- Empty leaf
- Large random cloud brute-force vs indexed

Line tests:

- Identical lines
- Parallel lines
- Intersecting lines
- Degenerate zero-length lines
- Repeated indexed lines
- Tie-distance cases

Triangle tests:

- Identical triangles
- Offset triangles
- Intersecting triangles
- Degenerate triangles
- Mixed triangle/line/point comparisons
- Tie-distance cases

Full tileset tests:

- Root-only tileset
- Deep hierarchy
- Wide hierarchy
- Multiple content URIs
- GLB
- B3DM
- PNTS
- Draco PNTS

## Performance Metrics to Track

Track:

```text
total runtime
tiles processed/sec
geometry load time
geometry comparison time
cache hit rate
cache miss rate
peak memory
number of primitive comparisons skipped by pruning
number of exact distance calls
```

## Correctness Rules

1. No approximate nearest-neighbor search.
2. Only prune using conservative lower bounds.
3. Preserve tie handling.
4. Do not change vertex/index interpretation.
5. Compare old and new outputs after every phase.

## Recommended First Pull Request

The first implementation PR should contain only:

1. Timing logs.
2. README performance notes.
3. Removal of `closest_representations` allocations.
4. Equivalence tests for the modified delta functions.

This is small, safe, and should improve performance without changing calculated values.
