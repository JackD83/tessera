use std::slice;

use crate::maths::{matrix::Mat4, sphere::Sphere, vec::Vec3};

pub mod compare;
pub mod delta;
pub mod kdtree;
pub mod prepared;

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

    pub fn bounding_sphere(&self) -> &Sphere {
        match self {
            Primitive::PointPrimitive(p) => &p.bounding_sphere,
            Primitive::LinePrimitive(p) => &p.bounding_sphere,
            Primitive::TrianglePrimitive(p) => &p.bounding_sphere,
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

pub enum PointVertexIter<'a> {
    Indexed {
        indices: slice::Iter<'a, u32>,
        vertices: &'a [[f32; 3]],
    },
    Unindexed(slice::Iter<'a, [f32; 3]>),
}

impl<'a> Iterator for PointVertexIter<'a> {
    type Item = &'a [f32; 3];

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            PointVertexIter::Indexed { indices, vertices } => {
                indices.next().map(|index| &vertices[*index as usize])
            }
            PointVertexIter::Unindexed(iter) => iter.next(),
        }
    }
}

pub enum LineVertexIter<'a> {
    Indexed {
        chunks: slice::Chunks<'a, u32>,
        vertices: &'a [[f32; 3]],
    },
    Unindexed(slice::Chunks<'a, [f32; 3]>),
}

impl<'a> Iterator for LineVertexIter<'a> {
    type Item = (&'a [f32; 3], &'a [f32; 3]);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            LineVertexIter::Indexed { chunks, vertices } => chunks
                .next()
                .map(|chunk| (&vertices[chunk[0] as usize], &vertices[chunk[1] as usize])),
            LineVertexIter::Unindexed(chunks) => chunks.next().map(|chunk| (&chunk[0], &chunk[1])),
        }
    }
}

pub enum TriangleVertexIter<'a> {
    Indexed {
        chunks: slice::Chunks<'a, u32>,
        vertices: &'a [[f32; 3]],
    },
    Unindexed(slice::Chunks<'a, [f32; 3]>),
}

impl<'a> Iterator for TriangleVertexIter<'a> {
    type Item = (&'a [f32; 3], &'a [f32; 3], &'a [f32; 3]);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TriangleVertexIter::Indexed { chunks, vertices } => chunks.next().map(|chunk| {
                (
                    &vertices[chunk[0] as usize],
                    &vertices[chunk[1] as usize],
                    &vertices[chunk[2] as usize],
                )
            }),
            TriangleVertexIter::Unindexed(chunks) => {
                chunks.next().map(|chunk| (&chunk[0], &chunk[1], &chunk[2]))
            }
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

    pub fn iter_vertices(&self) -> PointVertexIter<'_> {
        match &self.indices {
            Some(index) => PointVertexIter::Indexed {
                indices: index.iter(),
                vertices: &self.vertices,
            },
            None => PointVertexIter::Unindexed(self.vertices.iter()),
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

    pub fn iter_vertices(&self) -> LineVertexIter<'_> {
        match &self.indices {
            Some(index) => {
                // ignore vertices that do not form a whole line
                let safe_index_length = index.len() - (index.len() % 2);

                LineVertexIter::Indexed {
                    chunks: index[..safe_index_length].chunks(2),
                    vertices: &self.vertices,
                }
            }
            None => {
                // ignore vertices that do not form a whole line
                let safe_vertex_length = self.vertices.len() - (self.vertices.len() % 2);

                LineVertexIter::Unindexed(self.vertices[..safe_vertex_length].chunks(2))
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

    pub fn iter_vertices(&self) -> TriangleVertexIter<'_> {
        match &self.indices {
            Some(index) => {
                // ignore vertices that do not form a whole triangle
                let safe_index_length = index.len() - (index.len() % 3);

                TriangleVertexIter::Indexed {
                    chunks: index[..safe_index_length].chunks(3),
                    vertices: &self.vertices,
                }
            }
            None => {
                // ignore vertices that do not form a whole triangle
                let safe_vertex_length = self.vertices.len() - (self.vertices.len() % 3);

                TriangleVertexIter::Unindexed(self.vertices[..safe_vertex_length].chunks(3))
            }
        }
    }
}

#[cfg(test)]
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
