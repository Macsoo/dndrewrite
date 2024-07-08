use bevy::prelude::{Deref, DerefMut, Entity, Resource};
use bevy::utils::HashMap;
use hexx::Hex;
use super::*;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Party(pub(super) Vec<Entity>);

#[derive(Resource, Default, Debug)]
pub struct Map {
    pub(super) tiles: HashMap<Hex, tile::MapTile>,
    pub(super) combatants: Vec<Entity>,
}