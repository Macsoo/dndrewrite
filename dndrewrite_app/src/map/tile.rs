use bevy::prelude::Entity;

pub enum TilePart {
    Background,
    Overlay,
    Text,
}

#[derive(Debug, Default)]
pub struct Overlays {
    pub(super) location: Option<Entity>,
    pub(super) flair: Option<Entity>,
    pub(super) marker: Option<Entity>,
}

#[derive(Debug)]
pub struct MapTile {
    pub(super) background: Entity,
    pub(super) overlay: Overlays,
    pub(super) text: Option<Entity>,
}