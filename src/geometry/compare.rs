use crate::{
    error::TesseraError,
    geometry::{
        Geometry, Primitive,
        delta::{
            get_renderable_delta_between_line_and_point, get_renderable_delta_between_lines,
            get_renderable_delta_between_point_and_line, get_renderable_delta_between_points,
            get_renderable_delta_between_triangles,
        },
    },
};

// TODO: add bounding spheres and other features to be able to cull the search space

pub fn get_geometric_error_between_geometries(
    geometries: &Vec<&Geometry>,
    parent_geometries: &Vec<&Geometry>,
) -> Result<f64, TesseraError> {
    let mut shortest_distance = f64::INFINITY;

    for geometry in geometries {
        for parent_geometry in parent_geometries {
            let distance_result =
                get_renderable_delta_between_geometries(geometry, parent_geometry);

            if distance_result.is_err() {
                return distance_result;
            }

            let distance = distance_result.unwrap();
            if distance < shortest_distance {
                shortest_distance = distance;
            }
        }
    }

    return Ok(shortest_distance);
}

fn get_renderable_delta_between_geometries(
    geometry: &Geometry,
    parent_geometry: &Geometry,
) -> Result<f64, TesseraError> {
    let mut shortest_distance = f64::INFINITY;

    for primitive in &geometry.primitives {
        for parent_primitive in &parent_geometry.primitives {
            let renderable_delta =
                get_renderable_delta_between_primitives(&primitive, &parent_primitive);

            if renderable_delta.is_err() {
                return renderable_delta;
            }

            // in this case, we want the smallest renderable delta across all primitives
            // compared, as many of the primitives will have larger values as they represent
            // other objects. We assume the closest primitive is the one that represents the
            // simplified version of the original object(s).
            shortest_distance = shortest_distance.min(renderable_delta.unwrap());
        }
    }

    return Ok(shortest_distance);
}

fn get_renderable_delta_between_primitives(
    primitive: &Primitive,
    parent_primitive: &Primitive,
) -> Result<f64, TesseraError> {
    let primitive_comparison_type = (primitive, parent_primitive);

    match primitive_comparison_type {
        (Primitive::PointPrimitive(a), Primitive::PointPrimitive(b)) => {
            return get_renderable_delta_between_points(a, b);
        }
        (Primitive::PointPrimitive(a), Primitive::LinePrimitive(b)) => {
            return get_renderable_delta_between_point_and_line(a, b);
        }
        (Primitive::PointPrimitive(a), Primitive::TrianglePrimitive(b)) => {
            return get_renderable_delta_between_point_and_triangle(a, b);
        }
        (Primitive::LinePrimitive(a), Primitive::PointPrimitive(b)) => {
            return get_renderable_delta_between_line_and_point(a, b);
        }
        (Primitive::LinePrimitive(a), Primitive::LinePrimitive(b)) => {
            return get_renderable_delta_between_lines(a, b);
        }
        (Primitive::TrianglePrimitive(a), Primitive::TrianglePrimitive(b)) => {
            return get_renderable_delta_between_triangles(a, b);
        }
        (_, _) => {
            return Err(TesseraError::UnsupportedPrimitiveComparison(format!(
                "{:?}",
                primitive_comparison_type
            )));
        }
    }
}
