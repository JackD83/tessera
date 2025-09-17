use std::collections::HashMap;

use crate::tileset::{Tile, Tileset};

#[derive(Debug)]
pub(crate) struct TilesetNode {
    pub id: usize,

    pub content: Vec<String>,

    pub parent_id: Option<usize>,

    pub child_ids: Vec<usize>,

    // actual geometric error
    pub geometric_error: Option<f64>,
}

impl TilesetNode {
    pub fn add_content(&mut self, content: String) {
        self.content.push(content);
    }

    pub fn add_child(&mut self, child_id: usize) {
        self.child_ids.push(child_id);
    }

    pub fn is_leaf(&self) -> bool {
        self.child_ids.is_empty()
    }
}

pub(crate) fn parse_tileset_nodes(
    tileset: &Tileset,
) -> (HashMap<usize, TilesetNode>, usize, Vec<usize>) {
    let mut node_map = HashMap::<usize, TilesetNode>::new();
    let mut leaf_ids = Vec::<usize>::new();

    fn traverse(
        tile: &Tile,
        parent_id: Option<usize>,
        node_map: &mut HashMap<usize, TilesetNode>,
        leaf_ids: &mut Vec<usize>,
    ) -> usize {
        let mut node = TilesetNode {
            id: tile.id,
            content: Vec::<String>::new(),
            parent_id,
            child_ids: Vec::<usize>::new(),
            geometric_error: None,
        };

        // handle content, if exists
        // some tiles may have no content because they are a placeholder or
        // because they should be unconditionally refined.
        if let Some(content) = &tile.content {
            node.add_content(content.uri.clone());
        } else if let Some(contents) = &tile.contents {
            contents
                .iter()
                .for_each(|c| node.add_content(c.uri.clone()));
        };

        // handle children/leaves
        if !tile.children.is_empty() {
            for child in &tile.children {
                let child_id = traverse(child, Some(node.id), node_map, leaf_ids);

                node.add_child(child_id);
            }
        }

        let key = node.id;
        if node.is_leaf() {
            leaf_ids.push(key);
        }
        node_map.insert(key, node);

        return key;
    }

    traverse(&tileset.root, None, &mut node_map, &mut leaf_ids);

    return (node_map, tileset.root.id, leaf_ids);
}
