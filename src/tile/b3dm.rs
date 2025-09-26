use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fs,
    io::{self, BufReader},
    path::Path,
};

use gltf::{Gltf, import_buffers};

use crate::{
    error::TesseraError, geometry::Geometry, tile::gltf::gltf_to_geometry, utils::resolve_uri,
};

/// The header section of a .b3dm file.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Header {
    /// Must be `b"b3dm"`.
    pub magic: [u8; 4],
    /// Must be `1`.
    pub version: u32,
    /// Must match the length of the .b3dm file.
    pub length: u32,
    /// Must match the length of the feature table JSON.
    pub feature_table_json_byte_length: u32,
    /// Must match the length of the feature table binary.
    pub feature_table_binary_byte_length: u32,
    /// Must match the length of the batch table JSON.
    pub batch_table_json_byte_length: u32,
    /// Must match the length of the batch table binary.
    pub batch_table_binary_byte_length: u32,
}

impl Header {
    pub fn from_reader<R: io::Read>(reader: &mut R) -> Result<Header, TesseraError> {
        let mut magic = [0; 4];
        reader.read_exact(&mut magic).map_err(TesseraError::Io)?;

        let version = reader
            .read_u32::<LittleEndian>()
            .map_err(TesseraError::Io)?;
        let length = reader
            .read_u32::<LittleEndian>()
            .map_err(TesseraError::Io)?;
        let feature_table_json_byte_length = reader
            .read_u32::<LittleEndian>()
            .map_err(TesseraError::Io)?;
        let feature_table_binary_byte_length = reader
            .read_u32::<LittleEndian>()
            .map_err(TesseraError::Io)?;
        let batch_table_json_byte_length = reader
            .read_u32::<LittleEndian>()
            .map_err(TesseraError::Io)?;
        let batch_table_binary_byte_length = reader
            .read_u32::<LittleEndian>()
            .map_err(TesseraError::Io)?;

        if &magic != b"b3dm" {
            return Err(TesseraError::InvalidB3dmFile(
                "invalid B3DM file, magic did not match".to_string(),
            ));
        }

        Ok(Header {
            magic,
            version,
            length,
            feature_table_json_byte_length,
            feature_table_binary_byte_length,
            batch_table_json_byte_length,
            batch_table_binary_byte_length,
        })
    }

    pub fn header_size(self) -> u64 {
        return 28;
    }

    pub fn get_non_glb_data_length(self) -> u64 {
        return self.feature_table_json_byte_length as u64
            + self.feature_table_binary_byte_length as u64
            + self.batch_table_json_byte_length as u64
            + self.batch_table_binary_byte_length as u64;
    }

    pub fn get_glb_data_start_offset(self) -> u64 {
        return self.get_non_glb_data_length() + self.header_size();
    }
}

pub fn is_b3dm_like(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("b3dm")
}

pub fn load_tile_b3dm(base_dir: &Path, uri: &String) -> Result<Geometry, TesseraError> {
    let path = resolve_uri(base_dir, uri);

    if !is_b3dm_like(&path) {
        return Err(TesseraError::InvalidB3dmFile(uri.to_string()));
    }

    let file = fs::File::open(path).map_err(TesseraError::Io)?;
    let file_metadata = file.metadata().map_err(TesseraError::Io)?;
    let file_name = uri.to_string();
    let mut reader = BufReader::new(file);

    let header = Header::from_reader(&mut reader)?;

    let file_length = file_metadata.len();
    let header_specified_length = header.length as u64;
    if header_specified_length != file_length {
        return Err(TesseraError::InvalidB3dmFile(format!(
            r"{file_name} has different file length {file_length} than reported \
            in header {header_specified_length}! File is probably corrupt in some way."
        )));
    }

    let non_glb_data_length = header.get_non_glb_data_length();

    let glb_data_start_position = header.get_glb_data_start_offset();
    if glb_data_start_position >= header_specified_length {
        // GLB data would start off the end of the file
        return Err(TesseraError::InvalidB3dmFile(format!(
            "{file_name}: GLB data would start off the end of the file"
        )));
    }

    // push reader forward to start of GLB data
    let seek_offset: Result<i64, std::num::TryFromIntError> = non_glb_data_length.try_into();
    if seek_offset.is_err() {
        return Err(TesseraError::InvalidB3dmFile(format!(
            "could not seek to start of GLB data in {file_name}, specified length was too large"
        )));
    }

    reader
        .seek_relative(seek_offset.unwrap())
        .map_err(TesseraError::Io)?;

    // delegate to GLB loader from start of GLB data
    match Gltf::from_reader(reader) {
        Ok(gltf) => {
            let buffer_data = import_buffers(&gltf.document, None, gltf.blob).unwrap();
            return gltf_to_geometry(&uri, &gltf.document, &buffer_data);
        }
        Err(e) => {
            return Err(TesseraError::InvalidB3dmFile(format!(
                "could not parse GLB data in {file_name}: {e}"
            )));
        }
    }
}
