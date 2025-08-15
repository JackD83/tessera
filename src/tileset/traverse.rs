use std::collections::HashMap;

use crate::tileset::{Tile, Tileset};

#[derive(Debug)]
pub(crate) struct TilesetNode<'a> {
    pub key: u32,

    pub tile: &'a Tile,

    pub parent_key: Option<u32>,

    pub child_keys: Vec<u32>,

    // current lower bound for geometric error
    pub geometric_error_lower_bound: Option<f64>,

    // current upper bound for geometric error
    pub geometric_error_upper_bound: Option<f64>,

    // actual geometric error
    pub geometric_error: Option<f64>,
}

impl<'a> TilesetNode<'a> {
    pub fn add_child(&mut self, child_key: u32) {
        self.child_keys.push(child_key);
    }

    pub fn is_leaf(&self) -> bool {
        self.child_keys.is_empty()
    }
}

pub(crate) fn parse_tileset_nodes<'a>(tileset: &'a Tileset) -> HashMap<u32, TilesetNode<'a>> {
    let mut node_map = HashMap::<u32, TilesetNode>::new();
    let mut current_key: u32 = 0;

    fn traverse<'a>(
        tile: &'a Tile,
        parent_key: Option<u32>,
        node_map: &mut HashMap<u32, TilesetNode<'a>>,
        current_key: &mut u32,
    ) -> u32 {
        let mut node = TilesetNode {
            key: *current_key,
            tile,
            parent_key,
            child_keys: Vec::<u32>::new(),
            geometric_error_lower_bound: None,
            geometric_error_upper_bound: None,
            geometric_error: None,
        };
        *current_key += 1;

        if !tile.children.is_empty() {
            for child in &tile.children {
                let new_node_key = traverse(child, Some(node.key), node_map, current_key);

                node.add_child(new_node_key);
            }
        }

        let key = node.key;
        node_map.insert(key, node);

        return key;
    }

    traverse(&tileset.root, None, &mut node_map, &mut current_key);

    return node_map;
}
