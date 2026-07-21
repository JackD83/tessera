use crate::maths::{bounding_box::BoundingBox, vec::Vec3};

#[derive(Debug, Clone, Copy)]
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

    pub fn min_distance_to(&self, other: &Sphere) -> f64 {
        let center_distance = (self.center - other.center).length();
        return (center_distance - self.radius - other.radius).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_distance_to_returns_gap_between_disjoint_spheres() {
        let a = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
        let b = Sphere::new(Vec3::new(5.0, 0.0, 0.0), 2.0);

        assert_eq!(a.min_distance_to(&b), 2.0);
    }

    #[test]
    fn min_distance_to_returns_zero_for_touching_spheres() {
        let a = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
        let b = Sphere::new(Vec3::new(3.0, 0.0, 0.0), 2.0);

        assert_eq!(a.min_distance_to(&b), 0.0);
    }

    #[test]
    fn min_distance_to_returns_zero_for_overlapping_spheres() {
        let a = Sphere::new(Vec3::new(0.0, 0.0, 0.0), 2.0);
        let b = Sphere::new(Vec3::new(1.0, 0.0, 0.0), 2.0);

        assert_eq!(a.min_distance_to(&b), 0.0);
    }
}
