use thiserror::Error;

#[derive(Error, Debug)]
pub enum TesseraError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Processing error: {0}")]
    Processing(String),

    #[error("Tileset error: {0}")]
    Tileset(String),

    #[error("Unsupported tile type: {0}")]
    UnsupportedTileType(String),

    #[error("Invalid GLTF file: {0}")]
    InvalidGltfFile(String),

    #[error("Unsupported GLTF primitive type: {0}")]
    UnsuportedGltfPrimitiveType(String),

    #[error("Unsupported primitive comparison: {0}")]
    UnsupportedPrimitiveComparison(String),
}

pub type Result<T> = std::result::Result<T, TesseraError>;
