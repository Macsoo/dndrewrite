use std::sync::Arc;
use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::app::admin_button::AdminButton;
use super::*;
use crate::model::*;

#[derive(Resource, Deref, DerefMut)]
pub struct CurrentAdminMenu(pub(super) Arc<admin_menu::AdminMenu>);

#[derive(Resource, Deref, DerefMut)]
pub struct AdminMenus(pub(super) HashMap<id::Id, Arc<admin_menu::AdminMenu>>);

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct AdminMenuStack(pub(super) id::Id);

#[derive(Resource, Default)]
pub struct AppLoaded;

#[derive(Component, DerefMut, Deref)]
pub struct AdminButtonMarker(pub Arc<AdminButton>);

#[derive(Resource, Debug)]
pub struct UITracker {
    admin_bar: Entity,
    back_button: Entity,
    pub buttons: Vec<Entity>,
}

impl UITracker {
    pub fn new(admin_bar: Entity, back_button: Entity) -> Self {
        UITracker {
            admin_bar,
            back_button,
            buttons: Vec::new(),
        }
    }

    pub fn back_button(&self) -> Entity {
        self.back_button
    }

    pub fn admin_bar(&self) -> Entity {
        self.admin_bar
    }
}