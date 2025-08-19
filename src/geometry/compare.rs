use crate::{
    error::TesseraError,
    geometry::{Geometry, Primitive},
    maths::point::get_shortest_distance_between_points,
};

// TODO: add bounding spheres and other features to be able to cull the search space

pub fn get_shortest_distance(
    geometries: &Vec<&Geometry>,
    parent_geometries: &Vec<&Geometry>,
) -> Result<f64, TesseraError> {
    let mut shortest_distance = f64::INFINITY;

    for geometry in geometries {
        for parent_geometry in parent_geometries {
            let distance_result =
                get_shortest_distance_between_geometries(geometry, parent_geometry);

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

fn get_shortest_distance_between_geometries(
    geometry: &Geometry,
    parent_geometry: &Geometry,
) -> Result<f64, TesseraError> {
    let mut shortest_distance = f64::INFINITY;

    for primitive in &geometry.primitives {
        for parent_primitive in &parent_geometry.primitives {
            let distance_result =
                get_shortest_distance_between_primitives(&primitive, &parent_primitive);

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

fn get_shortest_distance_between_primitives(
    primitive: &Primitive,
    parent_primitive: &Primitive,
) -> Result<f64, TesseraError> {
    let primitive_comparison_type = (primitive, parent_primitive);

    match primitive_comparison_type {
        (Primitive::PointPrimitive(a), Primitive::PointPrimitive(b)) => {
            return get_shortest_distance_between_points(a, b);
        }
        // TODO: etc..
        // (PrimitiveType::Point, PrimitiveType::Line) => {
        //     return get_shortest_distance_between_point_and_line(primitive, parent_primitive);
        // }
        // (PrimitiveType::Line, PrimitiveType::Point) => {
        //     return get_shortest_distance_between_point_and_line(parent_primitive, primitive);
        // }
        (_, _) => {
            return Err(TesseraError::UnsupportedPrimitiveComparison(format!(
                "{:?}",
                primitive_comparison_type
            )));
        }
    }
}
