use bevy::prelude::*;
use hexx::Hex;
use crate::components::tile::{MapOverlayComponent, MapTileComponent};
use crate::map::tile::{MapTile, Overlays};
use crate::model::id::Id;
use crate::model::resources::TextureTreeResource;
use super::*;

impl resources::Map {
    pub fn place_tile(
        &mut self,
        commands: &mut Commands,
        texture_tree: &TextureTreeResource,
        hex: Hex,
        id: Id,
    ) {
        let tile = commands.spawn((SpriteBundle {
            texture: texture_tree.0[&id].leaf().unwrap(),
            ..default()
        }, MapTileComponent)).id();
        self.tiles.entry(hex).and_modify(|map_tile| {
            commands.entity(std::mem::replace(&mut map_tile.background, tile)).despawn();
        }).or_insert(MapTile {
            background: tile,
            overlay: Overlays::default(),
            text: None,
        });
    }

    pub fn place_overlay(
        &mut self,
        commands: &mut Commands,
        texture_tree: &TextureTreeResource,
        hex: Hex,
        id: Id,
    ) {
        let Some(overlays) = self.tiles.get_mut(&hex)
            .map(|x| &mut x.overlay) else { return };
        let entity = commands.spawn((SpriteBundle {
            texture: texture_tree.0[&id].leaf().unwrap(),
            ..default()
        }, MapOverlayComponent)).id();
        let overlay_field = match id.get(1).unwrap() {
            "marker" => &mut overlays.marker,
            "flair" => &mut overlays.flair,
            "location" => &mut overlays.location,
            _ => panic!()
        };
        *overlay_field = Some(entity);
    }

    pub fn place_text(
        &mut self,
        commands: &mut Commands,
        hex: Hex,
        text: String,
    ) {
        let Some(text_enity) = self.tiles.get_mut(&hex)
            .map(|x| &mut x.text) else { return };
        let entity = commands.spawn(Text2dBundle {
            text: Text::from_section(text, TextStyle::default()),
            ..default()
        }).id();
        *text_enity = Some(entity);
    }
}