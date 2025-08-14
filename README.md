# tessera
WIP Geometric Error CLI Tool


rough structure intended:

main.rs -> CLI entry point
lib.rs -> for anyone wishing to integrate directly or via FFI

tileset/ -> all tileset structs and loading code
tile/ -> all tile structs and tile loading code (one loader per type eventually: glb, b3dm, etc)

geometry/ -> internal representation of geometry data and impls to call the right code to calculate distance for each possible type (point <-> point, point <-> line, triangle <-> triangle, etc)
maths/ -> code for calculating distances


rough flow:
- load tileset to memory
- calculate_geometric_error for tileset
    - find all leaf tiles (max LOD), must have error = 0
    - for each leaf tile
        - load it into memory as geometry
        - for each parent in hierarchy
            - compare geometry to leaf geometry
            - get shortest distance between geometry primitives (e.g. shortest distance from triangle to triangle)
            - return largest shortest distance
    - aggregate results per node (max error from children) to find error for each node

possible optimisations required:
- balance having to re-load tiles from disk over and over vs memory usage of loading entire model
    - probably opt for an LRU cache with fixed memory limit and iterate in a way that keeps siblings together
- can't just compute for all leaf primitives to all parent primitives
    - can leverage bounding volumes to skip useless comparisons
        - need to consider all possibilities
        - leaf volume fully contains parent volume
        - parent volume fully contains leaf volume
        - volumes partially overlap
        - volumes do not intersect at all (this would be very weird but we can still calculate the distance)
        - but can probably construct cases where volume overlap is not the closest point
    - can we make a dynamic volume to effectively cull?
        - e.g. current shortest distance as radius from source and then calc a bounding sphere of the target primitive, if the two spheres do not intersect, the target should be further than our current closest distance
        - worst case is still checking all possibilities but average case is likely a lot better
        - maybe it can be carried over from primitive to primitive? we can generate bounds to skip computation that is too close too because a previous primitive distance was further (and thus would be the worst case distance for these nodes)
    - should we trust volumes in tileset or calculate them ourselves? (CLI option?)