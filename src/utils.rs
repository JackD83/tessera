use std::path::Path;

pub fn strip_query_and_fragment(uri: &str) -> &str {
    let without_query = uri.split('?').next().unwrap_or(uri);
    without_query.split('#').next().unwrap_or(without_query)
}

pub fn is_gltf_like(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext.to_lowercase().as_str(), "gltf" | "glb")
    } else {
        false
    }
}
