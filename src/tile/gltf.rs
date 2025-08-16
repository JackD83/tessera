use gltf::mesh::{Mode, util::ReadIndices};
use std::path::{Path, PathBuf};

use crate::{
    error::TesseraError,
    geometry::{Geometry, Primitive, PrimitiveType},
    utils::resolve_uri,
};

pub struct GltfAsset {
    pub source_path: PathBuf,
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

pub fn is_gltf_like(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext.to_lowercase().as_str(), "gltf" | "glb")
    } else {
        false
    }
}

pub fn load_tile_gltf(base_dir: &Path, uri: &String) -> Result<Geometry, TesseraError> {
    let path = resolve_uri(base_dir, uri);

    if !is_gltf_like(&path) {
        return Err(TesseraError::InvalidGltfFile(uri.to_string()));
    }

    match gltf::import(&path) {
        Ok((document, buffers, images)) => {
            return gltf_to_geometry(&uri, &document, &buffers, &images);
        }
        Err(e) => {
            return Err(TesseraError::Processing(format!(
                "Failed to load GLTF from {:?}: {}",
                path, e
            )));
        }
    }
}

pub fn gltf_to_geometry(
    name: &String,
    document: &gltf::Document,
    buffers: &Vec<gltf::buffer::Data>,
    images: &Vec<gltf::image::Data>,
) -> Result<Geometry, TesseraError> {
    let mut geometry = Geometry::new(name.to_string());

    for scene in document.scenes() {
        for node in scene.nodes() {
            if node.mesh().is_none() {
                continue;
            }

            let mesh = node.mesh().unwrap();

            // TODO: Add transform support, currently only direct vertex data is considered
            // TODO: Add bounding spheres per primitive (as easily transformed) so we can
            // avoid comparing when we know they are too far away
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let primitive_type = gltf_primitive_type_to_geometry_primitive_type(&primitive)?;

                let mut geometry_primitive = Primitive::new(primitive_type);

                geometry_primitive.set_vertices(reader.read_positions().unwrap().collect());

                if let Some(indices) = reader.read_indices() {
                    geometry_primitive.set_indices(match indices {
                        ReadIndices::U8(is) => is.map(|x| x as u32).collect(),
                        ReadIndices::U16(is) => is.map(|x| x as u32).collect(),
                        ReadIndices::U32(is) => is.collect(),
                    });
                };

                geometry.add_primitive(geometry_primitive);
            }
        }
    }

    return Ok(geometry);
}

fn gltf_primitive_type_to_geometry_primitive_type(
    primitive: &gltf::Primitive,
) -> Result<PrimitiveType, TesseraError> {
    match primitive.mode() {
        Mode::Triangles => Ok(PrimitiveType::Triangle),
        Mode::Lines => Ok(PrimitiveType::Line),
        Mode::Points => Ok(PrimitiveType::Point),
        // TODO: Consider supporting loops/strips/fans by expanding them
        _ => {
            return Err(TesseraError::UnsuportedGltfPrimitiveType(format!(
                "{:?}",
                primitive
            )));
        }
    }
}
