pub mod compare;

#[derive(Debug)]
pub enum PrimitiveType {
    Point,
    Line,
    Triangle,
}

#[derive(Debug)]

pub struct Geometry {
    // Name of the geometry
    pub name: String,

    pub primitives: Vec<Primitive>,
}

#[derive(Debug)]
pub struct Primitive {
    // Vertex data for the geometry
    pub vertices: Vec<[f32; 3]>,

    // Optional indices for the geometry
    pub indices: Option<Vec<u32>>,

    // Type of primitive for this geometry
    pub primitive_type: PrimitiveType,
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

impl Primitive {
    pub fn new(primitive_type: PrimitiveType) -> Primitive {
        Primitive {
            vertices: Vec::new(),
            indices: None,
            primitive_type,
        }
    }

    pub fn set_vertices(&mut self, other: Vec<[f32; 3]>) {
        self.vertices = other;
    }

    pub fn set_indices(&mut self, other: Vec<u32>) {
        self.indices = Some(other);
    }
}
