use std::path::{Path, PathBuf};

pub fn strip_query_and_fragment(uri: &str) -> &str {
    let without_query = uri.split('?').next().unwrap_or(uri);
    without_query.split('#').next().unwrap_or(without_query)
}

pub fn resolve_uri(base_dir: &Path, uri: &str) -> PathBuf {
    let trimmed = strip_query_and_fragment(uri);
    let p = Path::new(trimmed);

    if p.is_absolute() {
        return p.to_path_buf();
    }
    return base_dir.join(p);
}
