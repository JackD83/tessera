use crate::{
    geometry::{
        Geometry, LinePrimitive, PointPrimitive, Primitive, TrianglePrimitive, kdtree::PointKdTree,
    },
    maths::sphere::Sphere,
};

#[derive(Debug)]
pub struct PreparedTileGeometry {
    pub geometries: Vec<PreparedGeometry>,
}

#[derive(Debug)]
pub struct PreparedGeometry {
    pub name: String,
    pub primitives: Vec<PreparedPrimitive>,
}

#[derive(Debug)]
pub enum PreparedPrimitive {
    Points(PreparedPointPrimitive),
    Lines(PreparedLinePrimitive),
    Triangles(PreparedTrianglePrimitive),
}

#[derive(Debug)]
pub struct PreparedPointPrimitive {
    pub points: Vec<[f32; 3]>,
    pub bounding_sphere: Sphere,
    pub kdtree: PointKdTree,
}

#[derive(Debug)]
pub struct PreparedLinePrimitive {
    pub lines: Vec<([f32; 3], [f32; 3])>,
    pub line_aabbs: Vec<PreparedAabb>,
    pub bounding_sphere: Sphere,
}

#[derive(Debug)]
pub struct PreparedTrianglePrimitive {
    pub triangles: Vec<([f32; 3], [f32; 3], [f32; 3])>,
    pub triangle_aabbs: Vec<PreparedAabb>,
    pub bounding_sphere: Sphere,
}

#[derive(Debug, Clone, Copy)]
pub struct PreparedAabb {
    min: [f32; 3],
    max: [f32; 3],
}

impl PreparedAabb {
    fn from_points(points: &[[f32; 3]]) -> Self {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];

        for point in points {
            for axis in 0..3 {
                min[axis] = min[axis].min(point[axis]);
                max[axis] = max[axis].max(point[axis]);
            }
        }

        Self { min, max }
    }

    pub fn min_distance_to_point_squared(&self, point: &[f32; 3]) -> f64 {
        let mut distance = 0.0;

        for axis in 0..3 {
            let point = point[axis] as f64;
            let min = self.min[axis] as f64;
            let max = self.max[axis] as f64;

            if point < min {
                let delta = min - point;
                distance += delta * delta;
            } else if point > max {
                let delta = point - max;
                distance += delta * delta;
            }
        }

        distance
    }

    pub fn min_distance_to_aabb_squared(&self, other: &PreparedAabb) -> f64 {
        let mut distance = 0.0;

        for axis in 0..3 {
            let self_min = self.min[axis] as f64;
            let self_max = self.max[axis] as f64;
            let other_min = other.min[axis] as f64;
            let other_max = other.max[axis] as f64;

            if self_max < other_min {
                let delta = other_min - self_max;
                distance += delta * delta;
            } else if other_max < self_min {
                let delta = self_min - other_max;
                distance += delta * delta;
            }
        }

        distance
    }
}

impl PreparedTileGeometry {
    pub fn from_geometries(geometries: &[Geometry]) -> Self {
        Self {
            geometries: geometries
                .iter()
                .map(PreparedGeometry::from_geometry)
                .collect(),
        }
    }
}

impl PreparedGeometry {
    pub fn from_geometry(geometry: &Geometry) -> Self {
        Self {
            name: geometry.name.clone(),
            primitives: geometry
                .primitives
                .iter()
                .map(PreparedPrimitive::from_primitive)
                .collect(),
        }
    }
}

impl PreparedPrimitive {
    pub fn from_primitive(primitive: &Primitive) -> Self {
        match primitive {
            Primitive::PointPrimitive(primitive) => {
                PreparedPrimitive::Points(PreparedPointPrimitive::from_primitive(primitive))
            }
            Primitive::LinePrimitive(primitive) => {
                PreparedPrimitive::Lines(PreparedLinePrimitive::from_primitive(primitive))
            }
            Primitive::TrianglePrimitive(primitive) => {
                PreparedPrimitive::Triangles(PreparedTrianglePrimitive::from_primitive(primitive))
            }
        }
    }

    pub fn bounding_sphere(&self) -> &Sphere {
        match self {
            PreparedPrimitive::Points(primitive) => &primitive.bounding_sphere,
            PreparedPrimitive::Lines(primitive) => &primitive.bounding_sphere,
            PreparedPrimitive::Triangles(primitive) => &primitive.bounding_sphere,
        }
    }
}

impl PreparedPointPrimitive {
    pub(crate) fn from_primitive(primitive: &PointPrimitive) -> Self {
        let points = primitive.iter_vertices().copied().collect::<Vec<_>>();
        let kdtree = PointKdTree::from_points(&points);

        Self {
            points,
            bounding_sphere: primitive.bounding_sphere,
            kdtree,
        }
    }
}

impl PreparedLinePrimitive {
    pub(crate) fn from_primitive(primitive: &LinePrimitive) -> Self {
        let lines = primitive
            .iter_vertices()
            .map(|(start, end)| (*start, *end))
            .collect::<Vec<_>>();
        let line_aabbs = lines
            .iter()
            .map(|(start, end)| PreparedAabb::from_points(&[*start, *end]))
            .collect();

        Self {
            lines,
            line_aabbs,
            bounding_sphere: primitive.bounding_sphere,
        }
    }
}

impl PreparedTrianglePrimitive {
    pub(crate) fn from_primitive(primitive: &TrianglePrimitive) -> Self {
        let triangles = primitive
            .iter_vertices()
            .map(|(a, b, c)| (*a, *b, *c))
            .collect::<Vec<_>>();
        let triangle_aabbs = triangles
            .iter()
            .map(|(a, b, c)| PreparedAabb::from_points(&[*a, *b, *c]))
            .collect();

        Self {
            triangles,
            triangle_aabbs,
            bounding_sphere: primitive.bounding_sphere,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Vertices;

    #[test]
    fn prepared_points_preserve_index_order_and_repeated_indices() {
        let mut primitive = PointPrimitive::new();
        primitive.set_vertices(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        primitive.set_indices(vec![2, 0, 2, 1]);

        let prepared = PreparedPointPrimitive::from_primitive(&primitive);

        assert_eq!(
            prepared.points,
            vec![
                [2.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 0.0, 0.0]
            ]
        );
    }

    #[test]
    fn prepared_lines_drop_incomplete_index_chunk() {
        let mut primitive = LinePrimitive::new();
        primitive.set_vertices(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        primitive.set_indices(vec![0, 1, 2]);

        let prepared = PreparedLinePrimitive::from_primitive(&primitive);

        assert_eq!(prepared.lines, vec![([0.0, 0.0, 0.0], [1.0, 0.0, 0.0])]);
    }

    #[test]
    fn prepared_triangles_drop_incomplete_vertex_chunk() {
        let mut primitive = TrianglePrimitive::new();
        primitive.set_vertices(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [9.0, 9.0, 9.0],
        ]);

        let prepared = PreparedTrianglePrimitive::from_primitive(&primitive);

        assert_eq!(
            prepared.triangles,
            vec![([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])]
        );
    }
}
