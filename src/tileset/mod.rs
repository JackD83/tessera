pub mod loader;
pub(crate) mod traverse;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Tileset {
    // Metadata about the entire tileset.
    #[serde(default)]
    pub asset: Asset,

    // A dictionary object of metadata about per-feature properties.
    #[serde(default)]
    pub properties: Option<HashMap<String, Property>>,

    // 3D Metadata schema (3D Tiles 1.1). Left untyped for flexibility.
    #[serde(default)]
    pub schema: Option<serde_json::Value>,

    // Root tile of the tileset hierarchy.
    pub root: Tile,

    // Global geometric error (optional in 1.1, common in 1.0 examples).
    #[serde(default, rename = "geometricError")]
    pub geometric_error: Option<f64>,

    // Dictionary object with extension-specific objects.
    #[serde(default)]
    pub extensions: Option<HashMap<String, serde_json::Value>>,

    // Application-specific data.
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Asset {
    // 3D Tiles version (e.g. "1.0" or "1.1").
    #[serde(default)]
    pub version: String,

    // Application-specific tileset version.
    #[serde(default, rename = "tilesetVersion")]
    pub tileset_version: Option<String>,

    // Dictionary object with extension-specific objects.
    #[serde(default)]
    pub extensions: Option<HashMap<String, serde_json::Value>>,

    // Application-specific data.
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

// Per-feature property.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Property {
    //The maximum value of this property of all the features in the tileset.
    #[serde(default)]
    pub minimum: serde_json::Value,

    // The minimum value of this property of all the features in the tileset.
    #[serde(default)]
    pub maximum: serde_json::Value,

    // Dictionary object with extension-specific objects.
    #[serde(default)]
    pub extensions: Option<HashMap<String, serde_json::Value>>,

    // Application-specific data.
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tile {
    #[serde(skip)]
    pub id: usize,

    // The bounding volume that encloses the tile.
    #[serde(rename = "boundingVolume")]
    pub bounding_volume: BoundingVolume,

    // Optional bounding volume that defines the volume the viewer shall be
    // inside of before the tile’s content will be requested and before the
    // tile will be refined based on geometricError.
    #[serde(default, rename = "viewerRequestVolume")]
    pub viewer_request_volume: Option<BoundingVolume>,

    // The error, in meters, introduced if this tile is rendered and its
    // children are not. At runtime, the geometric error is used to compute
    // screen space error (SSE), i.e., the error measured in pixels.
    #[serde(default, rename = "geometricError")]
    pub geometric_error: f64,

    // Specifies if additive or replacement refinement is used when
    // traversing the tileset for rendering. This property is required for
    // the root tile of a tileset; it is optional for all other tiles. The
    // default is to inherit from the parent tile.
    #[serde(default)]
    pub refine: Option<Refine>,

    // A floating-point 4×4 affine transformation matrix, stored in
    // column-major order, that transforms the tile’s content—​i.e., its
    // features as well as content.boundingVolume, boundingVolume, and
    // viewerRequestVolume—​from the tile’s local coordinate system to the
    // parent tile’s coordinate system, or, in the case of a root tile,
    // from the tile’s local coordinate system to the tileset’s coordinate
    // system. transform does not apply to any volume property when the
    // volume is a region, defined in EPSG:4979 coordinates. transform
    // scales the geometricError by the maximum scaling factor from the matrix.
    #[serde(default)]
    pub transform: Option<[f64; 16]>,

    // Metadata about the tile’s content and a link to the content. When
    // this is omitted the tile is just used for culling. When this is
    //defined, then contents shall be undefined.
    #[serde(default)]
    pub content: Option<Content>,

    // An array of contents. When this is defined, then content shall
    // be undefined.
    #[serde(default)]
    pub contents: Option<Vec<Content>>,

    // A metadata entity that is associated with this tile.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,

    // An object that describes the implicit subdivision of this tile.
    #[serde(default, rename = "implicitTiling")]
    pub implicit_tiling: Option<ImplicitTiling>,

    // An array of objects that define child tiles. Each child tile content
    // is fully enclosed by its parent tile’s bounding volume and, generally,
    // has a geometricError less than its parent tile’s geometricError.
    // For leaf tiles, the length of this array is zero, and children may not
    // be defined.
    #[serde(default)]
    pub children: Vec<Tile>,

    // Dictionary object with extension-specific objects.
    #[serde(default)]
    pub extensions: Option<HashMap<String, serde_json::Value>>,

    // Application-specific data.
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Content {
    // An optional bounding volume that tightly encloses tile content.
    // tile.boundingVolume provides spatial coherence and
    // tile.content.boundingVolume enables tight view frustum culling.
    // When this is omitted, tile.boundingVolume is used.
    #[serde(default, rename = "boundingVolume")]
    pub bounding_volume: Option<BoundingVolume>,

    // A uri that points to tile content. When the uri is relative,
    // it is relative to the referring tileset JSON file.
    #[serde(default)]
    pub uri: String,

    // Metadata that is associated with this content.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,

    // The group this content belongs to. The value is an index
    // into the array of groups that is defined for the containing tileset.
    #[serde(default)]
    pub group: Option<u64>,

    // Dictionary object with extension-specific objects.
    #[serde(default)]
    pub extensions: Option<HashMap<String, serde_json::Value>>,

    // Application-specific data.
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BoundingVolume {
    // An array of 12 numbers that define an oriented bounding box.
    // The first three elements define the x, y, and z values for the
    // center of the box. The next three elements (with indices 3, 4, and 5)
    // define the x axis direction and half-length. The next three elements
    // (indices 6, 7, and 8) define the y axis direction and half-length.
    // The last three elements (indices 9, 10, and 11) define the z axis
    // direction and half-length.
    #[serde(default, rename = "box")]
    pub box_: Option<[f64; 12]>,

    // An array of six numbers that define a bounding geographic region in
    // EPSG:4979 coordinates with the order
    // [west, south, east, north, minimum height, maximum height].
    // Longitudes and latitudes are in radians, and heights are in meters
    // above (or below) the WGS84 ellipsoid.
    #[serde(default)]
    pub region: Option<[f64; 6]>,

    // An array of four numbers that define a bounding sphere. The first
    // three elements define the x, y, and z values for the center of the
    // sphere. The last element (with index 3) defines the radius in meters.
    #[serde(default)]
    pub sphere: Option<[f64; 4]>,

    // Dictionary object with extension-specific objects.
    #[serde(default)]
    pub extensions: Option<HashMap<String, serde_json::Value>>,

    // Application-specific data.
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum Refine {
    // Specifies that the tile’s children are to be added to the tile’s
    // content.
    ADD,

    // Specifies that the tile’s children are to replace the tile’s content.
    REPLACE,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum SubdivisionScheme {
    // Specifies that the tile is subdivided into four children, each of which
    // is a square.
    QUADTREE,

    // Specifies that the tile is subdivided into eight children, each of which
    // is a cube.
    OCTREE,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImplicitTiling {
    // A string describing the subdivision scheme used within the tileset.
    #[serde(rename = "subdivisionScheme")]
    pub subdivision_scheme: SubdivisionScheme,

    // The number of distinct levels in each subtree. For example, a quadtree
    // with subtreeLevels = 2 will have subtrees with 5 nodes (one root and 4
    // children).
    #[serde(rename = "subtreeLevels")]
    pub subtree_levels: u32,

    // The numbers of the levels in the tree with available tiles.
    #[serde(default, rename = "availableLevels")]
    pub available_levels: Option<u32>,

    // An object describing the location of subtree files.
    #[serde(default)]
    pub subtrees: Option<Subtree>,

    // Dictionary object with extension-specific objects.
    #[serde(default)]
    pub extensions: Option<HashMap<String, serde_json::Value>>,

    // Application-specific data.
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Subtree {
    // A template URI pointing to subtree files. A subtree is a fixed-depth
    // (defined by subtreeLevels) portion of the tree to keep memory use
    // bounded. The URI of each file is substituted with the subtree root’s
    // global level, x, and y. For subdivision scheme OCTREE, z shall also
    // be given. Relative paths are relative to the tileset JSON.
    pub uri: String,

    // Dictionary object with extension-specific objects.
    #[serde(default)]
    pub extensions: Option<HashMap<String, serde_json::Value>>,

    // Application-specific data.
    #[serde(default)]
    pub extras: Option<serde_json::Value>,
}
