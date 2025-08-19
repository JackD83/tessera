use itertools::Itertools;

pub mod compare;

#[derive(Debug)]
pub enum Primitive {
    PointPrimitive(PointPrimitive),
    LinePrimitive(LinePrimitive),
    TrianglePrimitive(TrianglePrimitive),
}

impl Primitive {
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
}

impl Vertices for PointPrimitive {
    fn set_vertices(&mut self, other: Vec<[f32; 3]>) {
        self.vertices = other;
    }

    fn set_indices(&mut self, other: Vec<u32>) {
        self.indices = Some(other);
    }
}

#[derive(Debug)]
pub struct LinePrimitive {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Option<Vec<u32>>,
}

impl Vertices for LinePrimitive {
    fn set_vertices(&mut self, other: Vec<[f32; 3]>) {
        self.vertices = other;
    }

    fn set_indices(&mut self, other: Vec<u32>) {
        self.indices = Some(other);
    }
}

#[derive(Debug)]
pub struct TrianglePrimitive {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Option<Vec<u32>>,
}

impl Vertices for TrianglePrimitive {
    fn set_vertices(&mut self, other: Vec<[f32; 3]>) {
        self.vertices = other;
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
}

impl PointPrimitive {
    pub fn new() -> PointPrimitive {
        PointPrimitive {
            vertices: Vec::new(),
            indices: None,
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
        }
    }

    pub fn iter_vertices(&self) -> Box<dyn Iterator<Item = (&[f32; 3], &[f32; 3])> + '_> {
        match &self.indices {
            Some(index) => {
                return Box::new(
                    index
                        .iter()
                        .tuple_windows()
                        .map(|(a, b)| (&self.vertices[*a as usize], &self.vertices[*b as usize])),
                );
            }
            None => {
                return Box::new(self.vertices.iter().tuple_windows());
            }
        }
    }
}

impl TrianglePrimitive {
    pub fn new() -> TrianglePrimitive {
        TrianglePrimitive {
            vertices: Vec::new(),
            indices: None,
        }
    }
}
