use crate::maths::{matrix::Mat4, sphere::Sphere, vec::Vec3};

pub mod compare;
pub mod delta;

#[derive(Debug)]
pub enum Primitive {
    PointPrimitive(PointPrimitive),
    LinePrimitive(LinePrimitive),
    TrianglePrimitive(TrianglePrimitive),
}

impl Primitive {
    pub fn get_vertices(&self) -> &Vec<[f32; 3]> {
        match self {
            Primitive::PointPrimitive(p) => p.get_vertices(),
            Primitive::LinePrimitive(p) => p.get_vertices(),
            Primitive::TrianglePrimitive(p) => p.get_vertices(),
        }
    }

    pub fn set_indices(&mut self, other: Vec<u32>) {
        match self {
            Primitive::PointPrimitive(p) => p.set_indices(other),
            Primitive::LinePrimitive(p) => p.set_indices(other),
            Primitive::TrianglePrimitive(p) => p.set_indices(other),
        }
    }

    pub fn set_vertices(&mut self, other: Vec<[f32; 3]>) {
        match self {
            Primitive::PointPrimitive(p) => p.set_vertices(other),
            Primitive::LinePrimitive(p) => p.set_vertices(other),
            Primitive::TrianglePrimitive(p) => p.set_vertices(other),
        }
    }
}

pub trait Vertices {
    fn get_vertices(&self) -> &Vec<[f32; 3]>;

    fn set_vertices(&mut self, other: Vec<[f32; 3]>);

    fn set_indices(&mut self, other: Vec<u32>);
}

#[derive(Debug)]

pub struct Geometry {
    // Name of the geometry
    pub name: String,

    pub primitives: Vec<Primitive>,
}

#[derive(Debug)]
pub struct PointPrimitive {
    // Vertex data for the geometry
    pub vertices: Vec<[f32; 3]>,

    // Optional indices for the geometry
    pub indices: Option<Vec<u32>>,

    // bounding sphere for the geometry, based on vertex data
    pub bounding_sphere: Sphere,
}

impl Vertices for PointPrimitive {
    fn get_vertices(&self) -> &Vec<[f32; 3]> {
        return &self.vertices;
    }

    fn set_vertices(&mut self, other: Vec<[f32; 3]>) {
        self.vertices = other;
        self.bounding_sphere = Sphere::from_points(&self.vertices.iter().collect());
    }

    fn set_indices(&mut self, other: Vec<u32>) {
        self.indices = Some(other);
    }
}

#[derive(Debug)]
pub struct LinePrimitive {
    // Vertex data for the geometry
    pub vertices: Vec<[f32; 3]>,

    // Optional indices for the geometry
    pub indices: Option<Vec<u32>>,

    // bounding sphere for the geometry, based on vertex data
    pub bounding_sphere: Sphere,
}

impl Vertices for LinePrimitive {
    fn get_vertices(&self) -> &Vec<[f32; 3]> {
        return &self.vertices;
    }

    fn set_vertices(&mut self, other: Vec<[f32; 3]>) {
        self.vertices = other;
        self.bounding_sphere = Sphere::from_points(&self.vertices.iter().collect());
    }

    fn set_indices(&mut self, other: Vec<u32>) {
        self.indices = Some(other);
    }
}

#[derive(Debug)]
pub struct TrianglePrimitive {
    // Vertex data for the geometry
    pub vertices: Vec<[f32; 3]>,

    // Optional indices for the geometry
    pub indices: Option<Vec<u32>>,

    // bounding sphere for the geometry, based on vertex data
    pub bounding_sphere: Sphere,
}

impl Vertices for TrianglePrimitive {
    fn get_vertices(&self) -> &Vec<[f32; 3]> {
        return &self.vertices;
    }

    fn set_vertices(&mut self, other: Vec<[f32; 3]>) {
        self.vertices = other;
        self.bounding_sphere = Sphere::from_points(&self.vertices.iter().collect());
    }

    fn set_indices(&mut self, other: Vec<u32>) {
        self.indices = Some(other);
    }
}

impl Geometry {
    pub fn new(name: String) -> Geometry {
        Geometry {
            name,
            primitives: Vec::new(),
        }
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.primitives.push(primitive);
    }

    pub fn apply_transform(&mut self, transform: &Mat4) {
        for primitive in &mut self.primitives {
            let transformed_vertices = primitive
                .get_vertices()
                .iter()
                .map(|v| (*transform * Vec3::from_array(v)).to_array())
                .collect();
            primitive.set_vertices(transformed_vertices);
        }
    }
}

impl PointPrimitive {
    pub fn new() -> PointPrimitive {
        PointPrimitive {
            vertices: Vec::new(),
            indices: None,
            bounding_sphere: Sphere::new(Vec3::new(0.0, 0.0, 0.0), 0.0),
        }
    }

    pub fn iter_vertices(&self) -> Box<dyn Iterator<Item = &[f32; 3]> + '_> {
        match &self.indices {
            Some(index) => {
                return Box::new(index.iter().map(|i| &self.vertices[*i as usize]));
            }
            None => {
                return Box::new(self.vertices.iter());
            }
        }
    }
}

impl LinePrimitive {
    pub fn new() -> LinePrimitive {
        LinePrimitive {
            vertices: Vec::new(),
            indices: None,
            bounding_sphere: Sphere::new(Vec3::new(0.0, 0.0, 0.0), 0.0),
        }
    }

    pub fn iter_vertices(&self) -> Box<dyn Iterator<Item = (&[f32; 3], &[f32; 3])> + '_> {
        match &self.indices {
            Some(index) => {
                if index.len() == 0 {
                    return Box::new(std::iter::empty());
                }

                // ignore vertices that do not form a whole line
                let safe_index_length = if index.len() % 2 == 0 {
                    index.len()
                } else {
                    index.len() - 1
                };

                return Box::new(index[..safe_index_length].chunks(2).map(|chunk| {
                    (
                        &self.vertices[chunk[0] as usize],
                        &self.vertices[chunk[1] as usize],
                    )
                }));
            }
            None => {
                if self.vertices.len() == 0 {
                    return Box::new(std::iter::empty());
                }

                // ignore vertices that do not form a whole line
                let safe_vertex_length = if self.vertices.len() % 2 == 0 {
                    self.vertices.len()
                } else {
                    self.vertices.len() - 1
                };

                return Box::new(
                    self.vertices[..safe_vertex_length]
                        .chunks(2)
                        .map(|chunk| (&chunk[0], &chunk[1])),
                );
            }
        }
    }
}

impl TrianglePrimitive {
    pub fn new() -> TrianglePrimitive {
        TrianglePrimitive {
            vertices: Vec::new(),
            indices: None,
            bounding_sphere: Sphere::new(Vec3::new(0.0, 0.0, 0.0), 0.0),
        }
    }

    pub fn iter_vertices(
        &self,
    ) -> Box<dyn Iterator<Item = (&[f32; 3], &[f32; 3], &[f32; 3])> + '_> {
        match &self.indices {
            Some(index) => {
                if index.len() == 0 {
                    return Box::new(std::iter::empty());
                }

                // ignore vertices that do not form a whole triangle
                let excess_indices = index.len() % 3;
                let safe_index_length = if excess_indices == 0 {
                    index.len()
                } else {
                    index.len() - excess_indices
                };

                return Box::new(index[..safe_index_length].chunks(3).map(|chunk| {
                    (
                        &self.vertices[chunk[0] as usize],
                        &self.vertices[chunk[1] as usize],
                        &self.vertices[chunk[2] as usize],
                    )
                }));
            }
            None => {
                if self.vertices.len() == 0 {
                    return Box::new(std::iter::empty());
                }

                // ignore vertices that do not form a whole triangle
                let excess_vertices = self.vertices.len() % 3;
                let safe_vertex_length = if excess_vertices == 0 {
                    self.vertices.len()
                } else {
                    self.vertices.len() - excess_vertices
                };

                return Box::new(
                    self.vertices[..safe_vertex_length]
                        .chunks(3)
                        .map(|chunk| (&chunk[0], &chunk[1], &chunk[2])),
                );
            }
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_iter_point_vertices() {
        let mut primitive = PointPrimitive::new();
        primitive.set_vertices(vec![[0.0, 1.0, 2.0], [1.0, 2.0, 3.0], [2.0, 3.0, 4.0]]);

        let mut iter = primitive.iter_vertices();

        assert_eq!(iter.next(), Some(&[0.0, 1.0, 2.0]));
        assert_eq!(iter.next(), Some(&[1.0, 2.0, 3.0]));
        assert_eq!(iter.next(), Some(&[2.0, 3.0, 4.0]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_point_vertices_with_indices() {
        let mut primitive = PointPrimitive::new();
        primitive.set_vertices(vec![[0.0, 1.0, 2.0], [1.0, 2.0, 3.0], [2.0, 3.0, 4.0]]);
        primitive.set_indices(vec![0, 2, 2, 1]);

        let mut iter = primitive.iter_vertices();

        assert_eq!(iter.next(), Some(&[0.0, 1.0, 2.0]));
        assert_eq!(iter.next(), Some(&[2.0, 3.0, 4.0]));
        // repeated index should be returned as expected
        assert_eq!(iter.next(), Some(&[2.0, 3.0, 4.0]));
        assert_eq!(iter.next(), Some(&[1.0, 2.0, 3.0]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_line_vertices() {
        let mut primitive = LinePrimitive::new();
        primitive.set_vertices(vec![
            [1.0, 1.0, 2.0],
            [1.0, 2.0, 3.0],
            [2.0, 3.0, 4.0],
            [3.0, 4.0, 5.0],
            [4.0, 5.0, 6.0],
        ]);

        let mut iter = primitive.iter_vertices();
        assert_eq!(iter.next(), Some((&[1.0, 1.0, 2.0], &[1.0, 2.0, 3.0])));
        assert_eq!(iter.next(), Some((&[2.0, 3.0, 4.0], &[3.0, 4.0, 5.0])));
        // last vertex should be dropped as it does not have a valid pair
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_line_vertices_with_indices() {
        let mut primitive = LinePrimitive::new();
        primitive.set_vertices(vec![
            [0.0, 1.0, 2.0],
            [1.0, 2.0, 3.0],
            [2.0, 3.0, 4.0],
            [3.0, 4.0, 5.0],
            [4.0, 5.0, 6.0],
        ]);
        // note: order swap here!
        primitive.set_indices(vec![0, 2, 1, 3, 2, 1, 4]);

        let mut iter = primitive.iter_vertices();
        assert_eq!(iter.next(), Some((&[0.0, 1.0, 2.0], &[2.0, 3.0, 4.0])));
        assert_eq!(iter.next(), Some((&[1.0, 2.0, 3.0], &[3.0, 4.0, 5.0])));
        // repeated index should be returned as expected
        assert_eq!(iter.next(), Some((&[2.0, 3.0, 4.0], &[1.0, 2.0, 3.0])));
        // last vertex should be dropped as no valid index pair
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_triangle_vertices() {
        let mut primitive = TrianglePrimitive::new();
        primitive.set_vertices(vec![
            [0.0, 1.0, 2.0], // tri 1 a
            [1.0, 2.0, 3.0], // tri 1 b
            [2.0, 3.0, 4.0], // tri 1 c
            [3.0, 4.0, 5.0], // tri 2 a
            [4.0, 5.0, 6.0], // tri 2 b
            [5.0, 6.0, 7.0], // tri 2 c
            [6.0, 7.0, 8.0], // discarded, not a whole triangle
            [7.0, 8.0, 9.0], // discarded, not a whole triangle
        ]);

        let mut iter = primitive.iter_vertices();
        assert_eq!(
            iter.next(),
            Some((&[0.0, 1.0, 2.0], &[1.0, 2.0, 3.0], &[2.0, 3.0, 4.0]))
        );
        assert_eq!(
            iter.next(),
            Some((&[3.0, 4.0, 5.0], &[4.0, 5.0, 6.0], &[5.0, 6.0, 7.0]))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_triangle_vertices_with_indices() {
        let mut primitive = TrianglePrimitive::new();
        primitive.set_vertices(vec![
            [0.0, 1.0, 2.0],
            [1.0, 2.0, 3.0],
            [2.0, 3.0, 4.0],
            [3.0, 4.0, 5.0],
            [4.0, 5.0, 6.0],
        ]);
        // note: triangles ABC, DEF, then 2 discarded entries
        primitive.set_indices(vec![0, 2, 1, 3, 2, 1, 4, 5]);

        let mut iter = primitive.iter_vertices();
        assert_eq!(
            iter.next(),
            Some((&[0.0, 1.0, 2.0], &[2.0, 3.0, 4.0], &[1.0, 2.0, 3.0]))
        );
        assert_eq!(
            iter.next(),
            Some((&[3.0, 4.0, 5.0], &[2.0, 3.0, 4.0], &[1.0, 2.0, 3.0]))
        );
        assert_eq!(iter.next(), None);
    }
}
