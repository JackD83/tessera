use autocxx::prelude::*;
use draco_rs::prelude::{DecoderBuffer, GetDracoInner, StatusOr, ffi};
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
                let mut geometry_primitive = create_primitive_from_gltf_primitive(&primitive)?;

                let (vertices, indices) =
                    read_primitive_positions_and_indices(&primitive, buffers, &current_transform)?;
                geometry_primitive.set_vertices(vertices);

                if let Some(indices) = indices {
                    geometry_primitive.set_indices(indices);
                };

                geometry.add_primitive(geometry_primitive);
            }
        }
    }

    return Ok(geometry);
}

fn read_primitive_positions_and_indices(
    primitive: &gltf::Primitive,
    buffers: &Vec<gltf::buffer::Data>,
    transform: &Mat4,
) -> Result<(Vec<[f32; 3]>, Option<Vec<u32>>), TesseraError> {
    if let Some(draco) = primitive.extension_value("KHR_draco_mesh_compression") {
        return read_draco_primitive_positions_and_indices(draco, buffers, transform);
    }

    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let vertices = reader
        .read_positions()
        .ok_or_else(|| {
            TesseraError::Processing("GLTF primitive missing POSITION attribute".to_string())
        })?
        .map(|v| transform_position(v, transform))
        .collect();

    let indices = reader.read_indices().map(|indices| match indices {
        ReadIndices::U8(is) => is.map(|x| x as u32).collect(),
        ReadIndices::U16(is) => is.map(|x| x as u32).collect(),
        ReadIndices::U32(is) => is.collect(),
    });

    Ok((vertices, indices))
}

fn read_draco_primitive_positions_and_indices(
    draco: &serde_json::Value,
    buffers: &Vec<gltf::buffer::Data>,
    transform: &Mat4,
) -> Result<(Vec<[f32; 3]>, Option<Vec<u32>>), TesseraError> {
    let view = draco
        .get("bufferView")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            TesseraError::Processing("KHR_draco_mesh_compression missing bufferView".to_string())
        })? as usize;
    let position_attr = draco
        .get("attributes")
        .and_then(|v| v.get("POSITION"))
        .and_then(|v| v.as_i64())
        .ok_or_else(|| {
            TesseraError::Processing(
                "KHR_draco_mesh_compression missing POSITION attribute".to_string(),
            )
        })? as i32;

    let buffer_view = buffers.get(view).ok_or_else(|| {
        TesseraError::Processing(format!("Invalid Draco bufferView index {}", view))
    })?;
    let mut decoder_buffer = DecoderBuffer::from_buffer(buffer_view);
    let mut decoder = ffi::draco::Decoder::new().within_unique_ptr();
    let mut status_or = unsafe {
        decoder
            .pin_mut()
            .DecodeMeshFromBuffer(decoder_buffer.get_inner_mut().as_mut_ptr())
    };
    if !status_or.ok() {
        return Err(TesseraError::DracoError(
            status_or.status().within_unique_ptr().into(),
        ));
    }

    let mut mesh = status_or.pin_mut().value();
    let mesh_pin = mesh.as_mut().ok_or_else(|| {
        TesseraError::Processing("Draco decoder returned an empty mesh".to_string())
    })?;
    let mesh_ref = mesh_pin.as_ref().get_ref();
    let point_cloud = <ffi::draco::Mesh as AsRef<ffi::draco::PointCloud>>::as_ref(mesh_ref);
    let position_attribute = point_cloud.GetAttributeByUniqueId(position_attr as u32);
    if position_attribute.is_null() {
        return Err(TesseraError::Processing(format!(
            "Draco POSITION attribute {} not found",
            position_attr
        )));
    }

    let vertices = (0..point_cloud.num_points())
        .map(|i| {
            let mut vertex = [0.0; 3];
            unsafe {
                (*position_attribute).GetMappedValue(
                    ffi::draco::PointIndexIndexType { val: i },
                    vertex.as_mut_ptr() as *mut autocxx::c_void,
                );
            }
            transform_position(vertex, transform)
        })
        .collect();

    let indices = (0..mesh_ref.num_faces() * 3)
        .map(|i| {
            mesh_ref
                .CornerToPointId1(ffi::draco::CornerIndexIndexType { val: i })
                .val
        })
        .collect();

    Ok((vertices, Some(indices)))
}

fn transform_position(position: [f32; 3], transform: &Mat4) -> [f32; 3] {
    let as_vec = Vec3::from_array(&position);
    let transformed = *transform * as_vec;
    transformed.to_array()
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
