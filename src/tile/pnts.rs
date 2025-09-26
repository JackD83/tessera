use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fs,
    io::{self, BufReader, Read, Seek},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{
    error::TesseraError,
    geometry::{Geometry, PointPrimitive, Primitive, Vertices},
    utils::resolve_uri,
};

/// The header section of a .pnts file.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Header {
    /// Must be `b"pnts"`.
    pub magic: [u8; 4],
    /// Must be `1`.
    pub version: u32,
    /// Must match the length of the .pnts file.
    pub length: u32,
    /// Must match the length of the feature table JSON.
    pub feature_table_json_byte_length: u32,
    /// Must match the length of the feature table binary.
    pub feature_table_binary_byte_length: u32,
    /// Must match the length of the batch table JSON. Zero if no batch table.
    pub batch_table_json_byte_length: u32,
    /// Must match the length of the batch table binary. Zero if json byte length is zero.
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

        if &magic != b"pnts" {
            return Err(TesseraError::InvalidPntsFile(
                "invalid PNTS file, magic did not match".to_string(),
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryBodyReference {
    #[serde(rename = "byteOffset")]
    pub byte_offset: u32,
    #[serde(rename = "componentType", skip_serializing_if = "Option::is_none")]
    pub component_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalPropertyInteger {
    pub value: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalPropertyCartesian3 {
    pub value: [f64; 3],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalPropertyCartesian4 {
    pub value: [f64; 4],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointCloudFeatureTable {
    // The position of the point in the local coordinate system.
    // Required if POSITION_QUANTIZED is not present.
    #[serde(rename = "POSITION", skip_serializing_if = "Option::is_none")]
    pub position: Option<BinaryBodyReference>,

    // The position of the point in the local coordinate system.
    // Required if POSITION is not present.
    #[serde(rename = "POSITION_QUANTIZED", skip_serializing_if = "Option::is_none")]
    pub position_quantized: Option<BinaryBodyReference>,

    #[serde(rename = "RGBA", skip_serializing_if = "Option::is_none")]
    pub rgba: Option<BinaryBodyReference>,

    #[serde(rename = "RGB", skip_serializing_if = "Option::is_none")]
    pub rgb: Option<BinaryBodyReference>,

    #[serde(rename = "RGB565", skip_serializing_if = "Option::is_none")]
    pub rgb565: Option<BinaryBodyReference>,

    #[serde(rename = "NORMAL", skip_serializing_if = "Option::is_none")]
    pub normal: Option<BinaryBodyReference>,

    #[serde(rename = "NORMAL_OCT16P", skip_serializing_if = "Option::is_none")]
    pub normal_oct16p: Option<BinaryBodyReference>,

    #[serde(rename = "BATCH_ID", skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<BinaryBodyReference>,

    #[serde(rename = "POINTS_LENGTH")]
    pub points_length: GlobalPropertyInteger,

    #[serde(rename = "RTC_CENTER", skip_serializing_if = "Option::is_none")]
    pub rtc_center: Option<GlobalPropertyCartesian3>,

    // A 3-component array of numbers defining the offset for the quantized volume
    // Required if POSITION_QUANTIZED is present.
    #[serde(
        rename = "QUANTIZED_VOLUME_OFFSET",
        skip_serializing_if = "Option::is_none"
    )]
    pub quantized_volume_offset: Option<GlobalPropertyCartesian3>,

    // A 3-component array of numbers defining the scale for the quantized volume
    // Required if POSITION_QUANTIZED is present.
    #[serde(
        rename = "QUANTIZED_VOLUME_SCALE",
        skip_serializing_if = "Option::is_none"
    )]
    pub quantized_volume_scale: Option<GlobalPropertyCartesian3>,

    #[serde(rename = "CONSTANT_RGBA", skip_serializing_if = "Option::is_none")]
    pub constant_rgba: Option<GlobalPropertyCartesian4>,

    #[serde(rename = "BATCH_LENGTH", skip_serializing_if = "Option::is_none")]
    pub batch_length: Option<GlobalPropertyInteger>,
}

impl Header {
    pub fn get_feature_table_length(self) -> u64 {
        return self.feature_table_json_byte_length as u64
            + self.feature_table_binary_byte_length as u64;
    }

    pub fn get_feature_table_start_offset(self) -> u64 {
        return self.header_size();
    }

    pub fn get_feature_table_binary_start_offset(self) -> u64 {
        return self.get_feature_table_start_offset() + self.feature_table_json_byte_length as u64;
    }
}

pub fn is_pnts_like(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("pnts")
}

pub fn load_tile_pnts(base_dir: &Path, uri: &String) -> Result<Geometry, TesseraError> {
    let path = resolve_uri(base_dir, uri);

    if !is_pnts_like(&path) {
        return Err(TesseraError::InvalidPntsFile(uri.to_string()));
    }

    let file = fs::File::open(path).map_err(TesseraError::Io)?;
    let file_metadata = file.metadata().map_err(TesseraError::Io)?;
    let file_name = uri.to_string();
    let mut reader = BufReader::new(file);

    let header = Header::from_reader(&mut reader)?;

    let file_length = file_metadata.len();
    let header_specified_length = header.length as u64;
    if header_specified_length != file_length {
        return Err(TesseraError::InvalidPntsFile(format!(
            r"{file_name} has different file length {file_length} than reported \
            in header {header_specified_length}! File is probably corrupt in some way."
        )));
    }

    let feature_table_length = header.get_feature_table_length();

    let feature_table_start_position = header.get_feature_table_start_offset();
    if feature_table_start_position >= header_specified_length {
        // Point data would start off the end of the file
        return Err(TesseraError::InvalidPntsFile(format!(
            "{file_name}: Point data would start off the end of the file"
        )));
    }
    if feature_table_start_position + feature_table_length > header_specified_length {
        return Err(TesseraError::InvalidPntsFile(format!(
            "{file_name}: Point data exceeds file length"
        )));
    }

    // push reader forward to start of Point data
    let seek_offset: Result<i64, std::num::TryFromIntError> =
        feature_table_start_position.try_into();
    if seek_offset.is_err() {
        return Err(TesseraError::InvalidPntsFile(format!(
            "could not seek to start of Point data in {file_name}, specified length was too large"
        )));
    }

    reader
        .seek_relative(seek_offset.unwrap())
        .map_err(TesseraError::Io)?;

    let feature_table: PointCloudFeatureTable =
        serde_json::from_reader(&mut reader).map_err(TesseraError::Json)?;

    // seek to start of feature table binary
    reader
        .seek(io::SeekFrom::Start(
            header.get_feature_table_binary_start_offset(),
        ))
        .map_err(TesseraError::Io)?;

    let mut feature_table_binary_data =
        Vec::with_capacity(header.feature_table_binary_byte_length as usize);

    // read feature table binary
    reader
        .read_exact(&mut feature_table_binary_data)
        .map_err(TesseraError::Io)?;

    let mut geometry = Geometry::new(file_name.clone());
    let mut points = PointPrimitive::new();

    match (&feature_table.position, &feature_table.position_quantized) {
        (Some(_), Some(_)) => {
            return Err(TesseraError::InvalidPntsFile(format!(
                "{file_name}: Both Position and Position_QUANTIZED are present"
            )));
        }
        (Some(position), None) => {
            points.set_vertices(extract_positions(
                &feature_table_binary_data,
                &position,
                feature_table.points_length.value as usize,
            ));
        }
        (None, Some(position_quantized)) => {
            points.set_vertices(extract_positions(
                &feature_table_binary_data,
                &position_quantized,
                feature_table.points_length.value as usize,
            ));
        }
        (None, None) => {
            return Err(TesseraError::InvalidPntsFile(format!(
                "{file_name}: POSITION or POSITION_QUANTIZED is required"
            )));
        }
    }

    geometry.add_primitive(Primitive::PointPrimitive(points));

    return Ok(geometry);
}

fn extract_positions(
    feature_table_binary_data: &Vec<u8>,
    position: &BinaryBodyReference,
    num_points: usize,
) -> Vec<[f32; 3]> {
    let mut position_data = Vec::<[f32; 3]>::with_capacity(num_points);
    let byte_start = position.byte_offset as usize;
    let byte_end = byte_start + num_points * 12; // 3 f32s per point, 4 bytes per f32
    let bytes = &feature_table_binary_data[byte_start..byte_end];

    // Convert bytes to f32 array safely
    for chunk in bytes.chunks_exact(12) {
        let x = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let y = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        let z = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
        position_data.push([x, y, z]);
    }

    return position_data;
}
