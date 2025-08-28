use crate::maths::vec::Vec3;

pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_points(points: &Vec<&[f32; 3]>) -> Self {
        let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for point in points {
            min.x = min.x.min(point[0] as f64);
            min.y = min.y.min(point[1] as f64);
            min.z = min.z.min(point[2] as f64);

            max.x = max.x.max(point[0] as f64);
            max.y = max.y.max(point[1] as f64);
            max.z = max.z.max(point[2] as f64);
        }

        Self { min, max }
    }

    pub fn get_center(&self) -> Vec3 {
        return self.min + (self.max - self.min) * 0.5;
    }
}
