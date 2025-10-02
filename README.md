# Tessera
Tessera is a library for accurate geometric error calculation in 3D tiles. Written in Rust, it takes your existing 3D tiles tilesets and computes the geometric error for each tile by traversing all of the content.


![Example before/after using Tessera to process a tileset](assets/tessera-example.gif)

Most tileset producers will approximate geometric error using something like the diagonal size of the bounding volume. However, this does not encode how meshes are typically simplified. Tessera was developed to ensure an accurate screen-space error can be calculated at runtime which starts with the geometric error. 

Tessera is released under the MIT License and provided "as is". Please use at your own risk, with no guarantees or professional support.


### Why did we build it?


Accurate geometric error gives you fine-grained control over the quality and performance characteristics of 3D tiles renderers. Sensat uses this to guarantee the performance of our web platform across a range of devices and hardware types. For example, quality settings can be automatically tweaked depending on the device resolution, available compute power, or whether it’s a portable device where battery life is a concern.


## Getting Started

Tessera can be used as a library or via command line interface. 


### Command Line Interface


Download the precompiled binary, or build from source, then run the following command:

```sh
tessera recalculate -i <your_tileset.json> -o <new_tileset.json> 
```

To see a list of available command line flags, run:

```
tessera help
```

### Library

Install via `cargo` with `cargo add <placeholder>` then see the `calculate_geometric_error` function in `lib.rs`

## Features

We built Tessera to improve our own tileset rendering, but realised it could be useful for the community. It has been widely tested against our own tilesets for point and mesh rendering, but we have not yet implemented the features we do not use in production.


✅ Accurate Geometric Error calculation from tileset geometry

✅ High performance geometric comparisons

✅ Handles comparing any combination of points, lines, and triangles, even when the simplification algorithm changed the geometry type

✅ Support for reading GLB, B3DM, and PNTS files in tilesets

✅ Support for DRACO compression in PNTS files

❌ Support for DRACO compression in GLB files

❌ Tileset transformations

❌ External tilesets (tiles with content pointing to another tileset.json)

❌ Implicit Tiling

❌ Fetching tilesets and their data


## Contributing


Community contributions are welcome and encouraged. 
If you feel there is a feature missing or encounter a bug, please submit a GitHub issue (PRs welcome too). 


## Disclaimer


Please note that while we are happy to collaborate with the community, Sensat cannot provide professional support or guarantees. Use of Tessera is at your own risk, and any reliance on its output is ultimately your responsibility. 


## Attributions


The Tessera project is supported by Sensat.
