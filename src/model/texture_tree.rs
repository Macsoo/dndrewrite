use std::collections::BTreeMap;
use std::ops::{Index, IndexMut};
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use super::*;

#[derive(Debug, Deref, DerefMut, Clone)]
pub struct TextureNode(pub Result<BTreeMap<String, TextureNode>, Handle<Image>>);

impl TextureNode {
    pub fn new(map: HashMap<id::Id, Handle<Image>>) -> Self {
        let mut root = Self(Ok(BTreeMap::new()));
        for (id, handle) in map {
            root[&id] = Self(Err(handle));
        }
        root
    }

    pub fn insert_branch(&mut self, name: String, branch: TextureNode) -> &mut TextureNode {
        let Ok(map) = &mut self.0 else { panic!("the node is a leaf") };
        map.entry(name).or_insert(branch)
    }

    pub fn get_branch(&self, name: &str) -> Option<&TextureNode> {
        let Ok(map) = &self.0 else { return None; };
        map.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        if let Ok(map) = &self.0 {
            map.contains_key(name)
        } else {
            false
        }
    }

    pub fn get_branch_mut(&mut self, name: &str) -> Option<&mut TextureNode> {
        let Ok(map) = &mut self.0 else { return None; };
        map.get_mut(name)
    }

    pub fn leaf(&self) -> Option<Handle<Image>> {
        self.0.as_ref().err().cloned()
    }

    pub fn branch(&self) -> Option<&BTreeMap<String, TextureNode>> {
        self.0.as_ref().ok()
    }

    pub fn result(&self) -> Result<&BTreeMap<String, TextureNode>, &Handle<Image>> {
        self.0.as_ref()
    }
}

impl Index<&id::Id> for TextureNode {
    type Output = TextureNode;

    fn index(&self, index: &id::Id) -> &Self::Output {
        let mut current = self;
        for name in &index.0 {
            current = current.get_branch(name).unwrap();
        }
        current
    }
}

impl IndexMut<&id::Id> for TextureNode {
    fn index_mut(&mut self, index: &id::Id) -> &mut Self::Output {
        let mut current = self;
        for name in &index.0 {
            let chosen = 'b: {
                if current.contains(name) {
                    match current.get_branch_mut(name) {
                        Some(branch) => break 'b branch,
                        None => unreachable!()
                    };
                }
                current.insert_branch(name.clone(), Self(Ok(BTreeMap::new())))
            };
            current = chosen;
        }
        current
    }
}