# Tessera

Tessera is a Rust library and command line tool for calculating accurate geometric error values for 3D Tiles tilesets. It traverses tile content, compares child geometry against parent geometry, and writes recalculated `geometricError` values back to a tileset JSON.

![Example before/after using Tessera to process a tileset](assets/tessera-example.gif)

Most tileset producers approximate geometric error from bounding-volume size, for example with a diagonal length. That does not necessarily describe the actual geometric difference introduced by simplification. Tessera calculates geometric error from the source geometry itself so renderers can make better screen-space-error decisions.

Tessera is released under the MIT License and provided "as is". Please use it at your own risk, with no guarantees or professional support.

## Why did we build it?

Accurate geometric error gives finer control over 3D Tiles renderer quality and performance. Sensat uses this to tune rendering behavior across different hardware, screen resolutions, and power/performance profiles.

## Getting started

Tessera can be used as a command line tool or as a Rust library.

### Build from source

For normal use, build in release mode:

```sh
cargo build --release
```

The release binary will be available at:

```sh
./target/release/tessera
```

For local benchmarking, you can also enable CPU-specific optimizations:

```sh
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Command line usage

### Recalculate geometric errors

```sh
tessera recalculate -i <your_tileset.json> -o <new_tileset.json>
```

Using the locally built release binary:

```sh
./target/release/tessera recalculate \
  -i <your_tileset.json> \
  -o <new_tileset.json>
```

Pretty-print the output JSON:

```sh
./target/release/tessera recalculate \
  -i <your_tileset.json> \
  -o <new_tileset.json> \
  --pretty
```

### Compare two tileset outputs

Tessera includes a parsed JSON comparison helper for validating optimized outputs against a baseline. It requires non-`geometricError` structure and values to match exactly, while comparing `geometricError` values with an absolute tolerance.

Default tolerance: `1e-9`

```sh
./target/release/tessera compare \
  --expected baseline.json \
  --actual optimized.json
```

Custom tolerance:

```sh
./target/release/tessera compare \
  --expected baseline.json \
  --actual optimized.json \
  --tolerance 1e-9
```

### Logging and progress

High-level timing logs:

```sh
./target/release/tessera --timings recalculate \
  -i <your_tileset.json> \
  -o <new_tileset.json>
```

Verbose mode also enables high-level timing and progress logs:

```sh
./target/release/tessera --verbose recalculate \
  -i <your_tileset.json> \
  -o <new_tileset.json>
```

Debug mode emits detailed per-tile cache and comparison logs:

```sh
./target/release/tessera --debug recalculate \
  -i <your_tileset.json> \
  -o <new_tileset.json>
```

During the candidate-error phase, verbose/timing logs include progress such as:

- completed leaf count
- completed leaf-to-parent comparison count
- slow geometry comparisons
- geometry and primitive counts for large/slow comparisons
- point/line/triangle element counts
- estimated comparison workload

This is useful because Tessera's progress is not always linear. A small number of dense leaf-to-parent comparisons can take much longer than many lightweight comparisons.

## Performance tuning

### Geometry cache size

Tessera caches prepared tile geometry. The default cache size is 256 tiles. If you have enough RAM, increasing the cache size can reduce repeated tile loading/preparation without changing calculated values.

```sh
./target/release/tessera --timings recalculate \
  -i <your_tileset.json> \
  -o <new_tileset.json> \
  --cache-tiles 5000
```

Use `--cache-tiles 0` to disable caching.

### Thread count

Tessera uses Rayon for parallel leaf processing. By default, Rayon chooses the worker count, and it also respects `RAYON_NUM_THREADS`.

Environment variable:

```sh
RAYON_NUM_THREADS=8 ./target/release/tessera recalculate \
  -i <your_tileset.json> \
  -o <new_tileset.json>
```

CLI flag:

```sh
./target/release/tessera recalculate \
  -i <your_tileset.json> \
  -o <new_tileset.json> \
  --threads 8
```

Try values such as `4`, `8`, `16`, and your physical core count. More threads are not always faster if the workload is memory-heavy or if cache contention dominates.

### Repeatable benchmark helper

A benchmark helper is available for one or more local tilesets:

```sh
scripts/benchmark.sh --cache-tiles 5000 --threads 8 <your_tileset.json>
```

Native CPU optimized benchmark:

```sh
scripts/benchmark.sh --native --cache-tiles 5000 --threads 8 <your_tileset.json>
```

Benchmark different thread counts:

```sh
scripts/benchmark.sh --output-dir bench-t4  --cache-tiles 5000 --threads 4  <your_tileset.json>
scripts/benchmark.sh --output-dir bench-t8  --cache-tiles 5000 --threads 8  <your_tileset.json>
scripts/benchmark.sh --output-dir bench-t16 --cache-tiles 5000 --threads 16 <your_tileset.json>
```

The script writes:

- recalculated output tilesets
- per-run logs
- build log
- `summary.tsv`
- max RSS memory when `/usr/bin/time` is available

### Profiling

`cargo flamegraph` can help identify hot paths in release builds:

```sh
cargo install flamegraph
cargo flamegraph -- recalculate -i <your_tileset.json> -o <new_tileset.json>
```

## Current optimization status

The current implementation includes several exact/conservative optimizations:

- prepared geometry cache
- resolved point/line/triangle primitive representations
- removal of hot-loop temporary allocation in line/triangle delta calculations
- concrete primitive iterators instead of boxed dynamic iterators
- conservative primitive-pair bounding-sphere pruning
- exact KD-tree lookup for point-to-point comparisons
- AABB lower-bound ordered traversal for point/line/triangle spatial comparisons
- progress and timing instrumentation

Correctness rules used by these optimizations:

- no approximate nearest-neighbor search
- no heuristic pruning
- pruning only with conservative lower bounds
- tie handling is preserved by continuing while candidates can tie the current closest distance
- vertex order, repeated indices, and incomplete primitive chunk behavior are preserved

## Library usage

Tessera can also be used as a library. The main entry points are:

```rust
use tessera::calculate_geometric_error;
use tessera::calculate_geometric_error_with_cache_size;
```

See `src/lib.rs` for the current API.

## Validation workflow

Recommended validation before comparing performance:

```sh
cargo fmt -- --check
cargo clippy
cargo test
cargo build --release
```

Then run a baseline/optimized output comparison:

```sh
./target/release/tessera compare \
  --expected baseline.json \
  --actual optimized.json
```

## Features

✅ Accurate geometric error calculation from tileset geometry

✅ Handles point, line, and triangle primitive comparisons, including mixed primitive-type comparisons

✅ Support for reading GLB, B3DM, and PNTS tile content

✅ Support for Draco compression in PNTS files

✅ Support for `KHR_draco_mesh_compression` in GLTF/GLB mesh primitives

✅ Tile transforms are applied to loaded geometry

✅ External tileset JSON content is inlined for local files

✅ Exact/conservative geometry-comparison optimizations

✅ CLI output comparison helper

✅ Benchmark helper script

❌ Implicit tiling

❌ Fetching remote tilesets and tile content

## Contributing

Community contributions are welcome and encouraged. If you feel there is a feature missing or encounter a bug, please submit a GitHub issue. Pull requests are welcome.

## Disclaimer

While we are happy to collaborate with the community, Sensat cannot provide professional support or guarantees. Use of Tessera is at your own risk, and any reliance on its output is ultimately your responsibility.

## Attributions

The Tessera project is supported by:

![Sensat](assets/sensat-logo-negative.png)
