use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::error::{Result, TesseraError};

pub const DEFAULT_GEOMETRIC_ERROR_TOLERANCE: f64 = 1e-9;

pub fn compare_tileset_json_files(
    expected_path: &Path,
    actual_path: &Path,
    geometric_error_tolerance: f64,
) -> Result<()> {
    let expected = read_json(expected_path)?;
    let actual = read_json(actual_path)?;

    compare_tileset_json_values(&expected, &actual, geometric_error_tolerance)
}

pub fn compare_tileset_json_values(
    expected: &Value,
    actual: &Value,
    geometric_error_tolerance: f64,
) -> Result<()> {
    if geometric_error_tolerance < 0.0 || geometric_error_tolerance.is_nan() {
        return Err(TesseraError::Tileset(format!(
            "Invalid geometric error tolerance {}; tolerance must be non-negative",
            geometric_error_tolerance
        )));
    }

    compare_json_except_geometric_error(expected, actual, "$")?;
    compare_geometric_errors(expected, actual, geometric_error_tolerance, "$")?;

    Ok(())
}

fn read_json(path: &Path) -> Result<Value> {
    let data = fs::read_to_string(path).map_err(TesseraError::Io)?;
    serde_json::from_str(&data).map_err(TesseraError::Json)
}

fn compare_json_except_geometric_error(expected: &Value, actual: &Value, path: &str) -> Result<()> {
    match (expected, actual) {
        (Value::Object(expected_object), Value::Object(actual_object)) => {
            for (key, expected_value) in expected_object {
                if key == "geometricError" {
                    continue;
                }

                let child_path = format_json_path(path, key);
                let actual_value = actual_object.get(key).ok_or_else(|| {
                    TesseraError::Tileset(format!(
                        "Missing key in actual tileset at {}",
                        child_path
                    ))
                })?;

                compare_json_except_geometric_error(expected_value, actual_value, &child_path)?;
            }

            for key in actual_object.keys() {
                if key == "geometricError" {
                    continue;
                }

                if !expected_object.contains_key(key) {
                    return Err(TesseraError::Tileset(format!(
                        "Unexpected key in actual tileset at {}",
                        format_json_path(path, key)
                    )));
                }
            }

            Ok(())
        }
        (Value::Array(expected_array), Value::Array(actual_array)) => {
            if expected_array.len() != actual_array.len() {
                return Err(TesseraError::Tileset(format!(
                    "Array length mismatch at {}: expected {}, got {}",
                    path,
                    expected_array.len(),
                    actual_array.len()
                )));
            }

            for (index, (expected_value, actual_value)) in
                expected_array.iter().zip(actual_array.iter()).enumerate()
            {
                compare_json_except_geometric_error(
                    expected_value,
                    actual_value,
                    &format!("{}[{}]", path, index),
                )?;
            }

            Ok(())
        }
        _ => {
            if expected == actual {
                Ok(())
            } else {
                Err(TesseraError::Tileset(format!(
                    "Value mismatch at {}: expected {}, got {}",
                    path, expected, actual
                )))
            }
        }
    }
}

fn compare_geometric_errors(
    expected: &Value,
    actual: &Value,
    tolerance: f64,
    path: &str,
) -> Result<()> {
    match (expected, actual) {
        (Value::Object(expected_object), Value::Object(actual_object)) => {
            let expected_error = expected_object.get("geometricError");
            let actual_error = actual_object.get("geometricError");

            match (expected_error, actual_error) {
                (Some(expected_error), Some(actual_error)) => {
                    compare_geometric_error_values(expected_error, actual_error, tolerance, path)?;
                }
                (None, None) => {}
                (Some(_), None) => {
                    return Err(TesseraError::Tileset(format!(
                        "Missing geometricError in actual tileset at {}",
                        path
                    )));
                }
                (None, Some(_)) => {
                    return Err(TesseraError::Tileset(format!(
                        "Unexpected geometricError in actual tileset at {}",
                        path
                    )));
                }
            }

            for (key, expected_value) in expected_object {
                if key == "geometricError" {
                    continue;
                }

                let Some(actual_value) = actual_object.get(key) else {
                    continue;
                };

                compare_geometric_errors(
                    expected_value,
                    actual_value,
                    tolerance,
                    &format_json_path(path, key),
                )?;
            }

            Ok(())
        }
        (Value::Array(expected_array), Value::Array(actual_array)) => {
            for (index, (expected_value, actual_value)) in
                expected_array.iter().zip(actual_array.iter()).enumerate()
            {
                compare_geometric_errors(
                    expected_value,
                    actual_value,
                    tolerance,
                    &format!("{}[{}]", path, index),
                )?;
            }

            Ok(())
        }
        _ => Ok(()),
    }
}

fn compare_geometric_error_values(
    expected: &Value,
    actual: &Value,
    tolerance: f64,
    path: &str,
) -> Result<()> {
    let expected = expected.as_f64().ok_or_else(|| {
        TesseraError::Tileset(format!(
            "Expected geometricError at {} is not a number: {}",
            path, expected
        ))
    })?;
    let actual = actual.as_f64().ok_or_else(|| {
        TesseraError::Tileset(format!(
            "Actual geometricError at {} is not a number: {}",
            path, actual
        ))
    })?;

    let difference = (expected - actual).abs();
    if difference <= tolerance {
        Ok(())
    } else {
        Err(TesseraError::Tileset(format!(
            "geometricError mismatch at {}: expected {}, got {}, absolute difference {} exceeds tolerance {}",
            path, expected, actual, difference, tolerance
        )))
    }
}

fn format_json_path(parent: &str, key: &str) -> String {
    format!("{}.{}", parent, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn compare_accepts_geometric_error_values_within_tolerance() {
        let expected = json!({
            "asset": { "version": "1.0" },
            "geometricError": 10.0,
            "root": {
                "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 1.0] },
                "geometricError": 5.0,
                "children": [
                    {
                        "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 1.0] },
                        "geometricError": 0.0
                    }
                ]
            }
        });
        let actual = json!({
            "asset": { "version": "1.0" },
            "geometricError": 10.0 + 5e-10,
            "root": {
                "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 1.0] },
                "geometricError": 5.0 - 5e-10,
                "children": [
                    {
                        "boundingVolume": { "sphere": [0.0, 0.0, 0.0, 1.0] },
                        "geometricError": 0.0
                    }
                ]
            }
        });

        compare_tileset_json_values(&expected, &actual, DEFAULT_GEOMETRIC_ERROR_TOLERANCE).unwrap();
    }

    #[test]
    fn compare_rejects_geometric_error_values_outside_tolerance() {
        let expected = json!({ "geometricError": 10.0, "root": { "geometricError": 5.0 } });
        let actual = json!({ "geometricError": 10.0, "root": { "geometricError": 5.00000001 } });

        let result =
            compare_tileset_json_values(&expected, &actual, DEFAULT_GEOMETRIC_ERROR_TOLERANCE);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("$.root"));
    }

    #[test]
    fn compare_rejects_structure_changes() {
        let expected = json!({
            "root": {
                "geometricError": 1.0,
                "children": [{ "geometricError": 0.0 }]
            }
        });
        let actual = json!({
            "root": {
                "geometricError": 1.0,
                "children": []
            }
        });

        let result =
            compare_tileset_json_values(&expected, &actual, DEFAULT_GEOMETRIC_ERROR_TOLERANCE);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("children"));
    }

    #[test]
    fn compare_rejects_non_geometric_error_value_changes() {
        let expected = json!({
            "asset": { "version": "1.0" },
            "root": { "geometricError": 1.0 }
        });
        let actual = json!({
            "asset": { "version": "1.1" },
            "root": { "geometricError": 1.0 }
        });

        let result =
            compare_tileset_json_values(&expected, &actual, DEFAULT_GEOMETRIC_ERROR_TOLERANCE);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("$.asset.version"));
    }
}
