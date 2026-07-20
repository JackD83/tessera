use crate::error::{Result, TesseraError};
use crate::tileset::{Content, Tile, Tileset};
use crate::utils::strip_query_and_fragment;
use pathdiff::diff_paths;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;

pub fn load_tileset(path: &Path) -> Result<Tileset> {
    let data = fs::read_to_string(path).map_err(TesseraError::Io)?;
    let mut tileset: Tileset = serde_json::from_str(&data)
        .map_err(|e| TesseraError::Tileset(format!("Failed to parse tileset.json: {}", e)))?;

    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    inline_external_tilesets(&mut tileset.root, base_dir, base_dir)?;
    assign_tile_ids_warn_and_reject_implicit_tiling(&mut tileset.root)?;

    return Ok(tileset);
}

fn assign_tile_ids_warn_and_reject_implicit_tiling(root: &mut Tile) -> Result<()> {
    let mut current_id = 0;

    fn traverse(tile: &mut Tile, current_id: &mut usize) -> Result<()> {
        tile.id = *current_id;

        let content = tile
            .content
            .as_ref()
            .map(|content| content.uri.as_str())
            .or_else(|| {
                tile.contents
                    .as_ref()
                    .and_then(|contents| contents.first())
                    .map(|content| content.uri.as_str())
            })
            .unwrap_or("<no content>");

        if tile.implicit_tiling.is_some() {
            return Err(TesseraError::Tileset(format!(
                "Implicit tiling is not implemented; encountered on tile {} with content {}",
                tile.id, content
            )));
        }

        if tile.transform.is_some() {
            warn!(
                tile_id = tile.id,
                content,
                "Tile transform encountered but tileset transformations are not implemented; geometric error may be inaccurate"
            );
        }

        *current_id += 1;
        for child in &mut tile.children {
            traverse(child, current_id)?;
        }

        return Ok(());
    }

    return traverse(root, &mut current_id);
}

fn inline_external_tilesets(
    tile: &mut Tile,
    current_base_dir: &Path,
    root_base_dir: &Path,
) -> Result<()> {
    // Existing child tiles are relative to the current tileset file.
    for child in &mut tile.children {
        inline_external_tilesets(child, current_base_dir, root_base_dir)?;
    }

    // Multi-content external tilesets cannot be merged into this tile directly,
    // so inline them as additional children. Normal contents just get path-rebased.
    let mut external_content_roots = Vec::<Tile>::new();
    if let Some(contents) = tile.contents.take() {
        let mut kept_contents = Vec::<Content>::new();

        for mut content in contents {
            if is_tileset_uri(&content.uri) {
                external_content_roots.push(load_external_tileset_root(
                    &content.uri,
                    current_base_dir,
                    root_base_dir,
                )?);
            } else {
                content.uri = rebase_uri(&content.uri, current_base_dir, root_base_dir);
                kept_contents.push(content);
            }
        }

        if !kept_contents.is_empty() {
            tile.contents = Some(kept_contents);
        }
    }

    tile.children.extend(external_content_roots);

    if let Some(content) = &mut tile.content {
        if is_tileset_uri(&content.uri) {
            let external_root =
                load_external_tileset_root(&content.uri, current_base_dir, root_base_dir)?;
            merge_external_root_into_tile(tile, external_root);
        } else {
            content.uri = rebase_uri(&content.uri, current_base_dir, root_base_dir);
        }
    }

    return Ok(());
}

fn load_external_tileset_root(
    uri: &str,
    current_base_dir: &Path,
    root_base_dir: &Path,
) -> Result<Tile> {
    let external_tileset_path = current_base_dir.join(strip_query_and_fragment(uri));
    let external_base_dir = external_tileset_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let data = fs::read_to_string(&external_tileset_path).map_err(TesseraError::Io)?;
    let mut external_tileset: Tileset = serde_json::from_str(&data).map_err(|e| {
        TesseraError::Tileset(format!(
            "Failed to parse external tileset {:?}: {}",
            external_tileset_path, e
        ))
    })?;

    inline_external_tilesets(&mut external_tileset.root, external_base_dir, root_base_dir)?;

    return Ok(external_tileset.root);
}

fn merge_external_root_into_tile(tile: &mut Tile, mut external_root: Tile) {
    let existing_children = std::mem::take(&mut tile.children);

    tile.content = external_root.content.take();
    tile.contents = external_root.contents.take();
    tile.children = external_root.children;
    tile.children.extend(existing_children);

    // Keep the parent tile's bounding volume and transform: they are what located
    // the external tileset in the parent. Use the external root's refinement if it
    // has one, because that describes the inlined subtree.
    if external_root.refine.is_some() {
        tile.refine = external_root.refine;
    }
}

fn rebase_uri(uri: &str, from_base_dir: &Path, to_base_dir: &Path) -> String {
    let trimmed = strip_query_and_fragment(uri);
    let suffix = &uri[trimmed.len()..];
    let path = Path::new(trimmed);

    if path.is_absolute() {
        return uri.to_string();
    }

    let absolute_path = from_base_dir.join(path);
    let relative_path = diff_paths(&absolute_path, to_base_dir).unwrap_or(absolute_path);
    let relative_uri = path_to_uri(relative_path);

    return format!("{}{}", relative_uri, suffix);
}

fn path_to_uri(path: PathBuf) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn is_tileset_uri(uri: &str) -> bool {
    strip_query_and_fragment(uri)
        .to_ascii_lowercase()
        .ends_with(".json")
}
