use crate::{error::TesseraError, geometry::Primitive};

pub fn get_shortest_distance_between_points(
    a: &Primitive,
    b: &Primitive,
) -> Result<f64, TesseraError> {
    let mut shortest_distance = f64::INFINITY;

    for a_point in &a.vertices {
        for b_point in &b.vertices {
            let distance = point_distance_squared(&a_point, &b_point);

            if distance < shortest_distance {
                shortest_distance = distance;
            }
        }
    }

    return Ok(shortest_distance.sqrt());
}

fn point_distance_squared(a: &[f32; 3], b: &[f32; 3]) -> f64 {
    let dx: f64 = a[0] as f64 - b[0] as f64;
    let dy: f64 = a[1] as f64 - b[1] as f64;
    let dz: f64 = a[2] as f64 - b[2] as f64;

    return (dx * dx + dy * dy + dz * dz);
}
