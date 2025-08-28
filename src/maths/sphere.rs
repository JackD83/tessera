use crate::maths::{bounding_box::BoundingBox, vec::Vec3};

pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn from_points(points: &Vec<&[f32; 3]>) -> Self {
        let center = BoundingBox::from_points(points).get_center();
        let mut radius: f64 = 0.0;

        for point in points {
            let distance = (Vec3::from_array(point) - center).length();

            radius = radius.max(distance);
        }

        Self { center, radius }
    }

    pub fn expand_by_point(&mut self, point: Vec3) {
        let distance = (point - self.center).length();
        if distance > self.radius {
            self.radius = distance;
        }
    }

    pub fn union(&mut self, other: &Sphere) {
        let center = (self.center + other.center) * 0.5;

        let radius_from_self = (self.center - center).length() + self.radius;
        let radius_from_other = (other.center - center).length() + other.radius;

        let radius = radius_from_self.max(radius_from_other);

        self.center = center;
        self.radius = radius;
    }
}
