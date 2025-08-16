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

    return dx * dx + dy * dy + dz * dz;
}

#[cfg(test)]
mod tests {
    use crate::geometry::PrimitiveType;

    use super::*;

    #[test]
    fn test_get_shortest_distance_between_points() {
        let mut a = Primitive::new(PrimitiveType::Point);
        a.set_vertices(vec![[0.0, 0.0, 0.0]]);

        let mut b = Primitive::new(PrimitiveType::Point);
        b.set_vertices(vec![[1.0, 1.0, 1.0]]);

        let distance = get_shortest_distance_between_points(&a, &b);
        assert!(distance.is_ok());
        assert_eq!(distance.unwrap(), 3.0_f64.sqrt());
    }

    #[test]
    fn test_get_shortest_distance_between_same_points() {
        let mut a = Primitive::new(PrimitiveType::Point);
        a.set_vertices(vec![[1.0, 1.0, 1.0]]);

        let mut b = Primitive::new(PrimitiveType::Point);
        b.set_vertices(vec![[1.0, 1.0, 1.0]]);

        let distance = get_shortest_distance_between_points(&a, &b);
        assert!(distance.is_ok());
        assert_eq!(distance.unwrap(), 0.0_f64);
    }

    #[test]
    fn test_get_shortest_distance_between_points_with_multiple_points() {
        let mut a = Primitive::new(PrimitiveType::Point);
        a.set_vertices(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);

        let mut b = Primitive::new(PrimitiveType::Point);
        b.set_vertices(vec![[2.0, 2.0, 2.0], [3.0, 3.0, 3.0]]);

        let distance = get_shortest_distance_between_points(&a, &b);
        assert!(distance.is_ok());
        assert_eq!(distance.unwrap(), 3.0_f64.sqrt());
    }

    #[test]
    fn test_get_shortest_distance_between_points_with_multiple_points_that_share_a_point() {
        let mut a = Primitive::new(PrimitiveType::Point);
        a.set_vertices(vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);

        let mut b = Primitive::new(PrimitiveType::Point);
        b.set_vertices(vec![[1.0, 1.0, 1.0], [2.0, 2.0, 2.0]]);

        let distance = get_shortest_distance_between_points(&a, &b);
        assert!(distance.is_ok());
        assert_eq!(distance.unwrap(), 0.0_f64);
    }

    #[test]
    fn test_point_distance_squared() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 1.0, 1.0];
        let distance = point_distance_squared(&a, &b);
        assert_eq!(distance, 3.0);
    }
}
