#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/benchmark.sh [OPTIONS] <tileset.json> [<tileset.json> ...]

Builds tessera in release mode, runs `recalculate` for each tileset, and records
repeatable timing logs and outputs under a benchmark output directory.

Options:
  --output-dir DIR       Directory for benchmark outputs (default: benchmark-output)
  --cache-tiles COUNT    Geometry cache tile count passed to tessera (default: 256)
  --threads COUNT        Rayon worker thread count passed to tessera (optional)
  --native               Build with RUSTFLAGS="-C target-cpu=native"
  --pretty               Pretty-print output tilesets
  -h, --help             Show this help

Examples:
  scripts/benchmark.sh data/small/tileset.json data/large/tileset.json
  scripts/benchmark.sh --native --threads 8 --cache-tiles 5000 data/tileset.json
EOF
}

output_dir="benchmark-output"
cache_tiles="256"
threads=""
native="false"
pretty="false"
tilesets=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-dir)
      output_dir="${2:?--output-dir requires a value}"
      shift 2
      ;;
    --cache-tiles)
      cache_tiles="${2:?--cache-tiles requires a value}"
      shift 2
      ;;
    --threads)
      threads="${2:?--threads requires a value}"
      shift 2
      ;;
    --native)
      native="true"
      shift
      ;;
    --pretty)
      pretty="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      tilesets+=("$@")
      break
      ;;
    -*)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
    *)
      tilesets+=("$1")
      shift
      ;;
  esac
done

if [[ ${#tilesets[@]} -eq 0 ]]; then
  echo "At least one tileset path is required" >&2
  usage >&2
  exit 2
fi

mkdir -p "$output_dir"

build_log="$output_dir/build.log"
if [[ "$native" == "true" ]]; then
  echo "Building release binary with native CPU optimizations..." | tee "$build_log"
  RUSTFLAGS="-C target-cpu=native" cargo build --release 2>&1 | tee -a "$build_log"
else
  echo "Building release binary..." | tee "$build_log"
  cargo build --release 2>&1 | tee -a "$build_log"
fi

summary="$output_dir/summary.tsv"
printf "tileset\toutput\tcache_tiles\tthreads\tnative\telapsed_seconds\tmax_rss_kb\tstatus\n" > "$summary"

for tileset in "${tilesets[@]}"; do
  if [[ ! -f "$tileset" ]]; then
    echo "Skipping missing tileset: $tileset" >&2
    printf "%s\t\t%s\t%s\t%s\t\t\tskipped_missing\n" "$tileset" "$cache_tiles" "${threads:-default}" "$native" >> "$summary"
    continue
  fi

  safe_name="$(echo "$tileset" | sed 's#[^A-Za-z0-9_.-]#_#g')"
  output="$output_dir/${safe_name}.out.json"
  log="$output_dir/${safe_name}.log"

  command=("./target/release/tessera" "--timings" "recalculate" "-i" "$tileset" "-o" "$output" "--cache-tiles" "$cache_tiles")
  if [[ -n "$threads" ]]; then
    command+=("--threads" "$threads")
  fi
  if [[ "$pretty" == "true" ]]; then
    command+=("--pretty")
  fi

  echo "Running: ${command[*]}" | tee "$log"

  start_ns="$(date +%s%N)"
  status="ok"
  max_rss=""

  if command -v /usr/bin/time >/dev/null 2>&1; then
    time_file="$output_dir/${safe_name}.time"
    if ! /usr/bin/time -f "max_rss_kb=%M" -o "$time_file" "${command[@]}" 2>&1 | tee -a "$log"; then
      status="failed"
    fi
    max_rss="$(sed -n 's/^max_rss_kb=//p' "$time_file" | tail -1)"
  else
    if ! "${command[@]}" 2>&1 | tee -a "$log"; then
      status="failed"
    fi
  fi

  end_ns="$(date +%s%N)"
  elapsed_seconds="$(awk "BEGIN { printf \"%.3f\", ($end_ns - $start_ns) / 1000000000 }")"

  printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n" \
    "$tileset" "$output" "$cache_tiles" "${threads:-default}" "$native" "$elapsed_seconds" "${max_rss:-}" "$status" >> "$summary"

  if [[ "$status" != "ok" ]]; then
    exit 1
  fi
done

echo "Benchmark summary written to $summary"
