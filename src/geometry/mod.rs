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

    // Vertex data for the geometry
    pub vertices: Vec<f32>,

    // Optional indices for the geometry
    pub indices: Option<Vec<u32>>,

    // Type of primitive for this geometry
    pub primitive_type: PrimitiveType,
}
