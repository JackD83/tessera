use gltf::mesh::{Mode, util::ReadIndices};
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use crate::{
    error::TesseraError,
    geometry::{Geometry, LinePrimitive, PointPrimitive, Primitive, TrianglePrimitive},
    maths::{matrix::Mat4, vec::Vec3},
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
        Ok((document, buffers, _images)) => {
            return gltf_to_geometry(&uri, &document, &buffers);
        }
        Err(e) => {
            return Err(TesseraError::Processing(format!(
                "Failed to load GLTF from {:?}: {}",
                path, e
            )));
        }
    }
}

// TODO: add unit tests for this to ensure transform is correctly applied in edge cases
pub fn gltf_to_geometry(
    name: &String,
    document: &gltf::Document,
    buffers: &Vec<gltf::buffer::Data>,
) -> Result<Geometry, TesseraError> {
    let mut geometry = Geometry::new(name.to_string());

    for scene in document.scenes() {
        let mut transform_stack = Vec::<Mat4>::new();
        let mut nodes = scene.nodes().collect::<VecDeque<_>>();

        while !nodes.is_empty() {
            let node = nodes.pop_front().unwrap();

            let node_transform = Mat4::from_column_major_nested_array(&node.transform().matrix());
            transform_stack.push(node_transform);

            // depth-first to match our transform stack
            node.children().for_each(|child| nodes.push_front(child));

            if node.mesh().is_none() {
                // nothing else to do at this node if there's no mesh
                continue;
            }

            let mesh = node.mesh().unwrap();
            // find the current transform by multiplying all the transforms in the stack
            let current_transform = transform_stack
                .iter()
                .fold(Mat4::identity(), |acc, x| acc * x);

            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let mut geometry_primitive = create_primitive_from_gltf_primitive(&primitive)?;

                // TODO: consider not applying the transforms directly and instead storing them
                // as part of the geometry, if the on-the-fly computation cost is acceptable
                // for the reduction in memory usage.
                let transformed_vertices = reader
                    .read_positions()
                    .unwrap()
                    .map(|v| {
                        let as_vec = Vec3::from_array(&v);
                        let transformed = current_transform * as_vec;
                        return transformed.to_array();
                    })
                    .collect();
                geometry_primitive.set_vertices(transformed_vertices);

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

fn create_primitive_from_gltf_primitive(
    primitive: &gltf::Primitive,
) -> Result<Primitive, TesseraError> {
    match primitive.mode() {
        Mode::Triangles => Ok(Primitive::TrianglePrimitive(TrianglePrimitive::new())),
        Mode::Lines => Ok(Primitive::LinePrimitive(LinePrimitive::new())),
        Mode::Points => Ok(Primitive::PointPrimitive(PointPrimitive::new())),
        // TODO: Consider supporting loops/strips/fans by expanding them, or just tuple_windows()
        _ => {
            return Err(TesseraError::UnsuportedGltfPrimitiveType(format!(
                "{:?}",
                primitive
            )));
        }
    }
}
