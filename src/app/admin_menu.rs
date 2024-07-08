use std::sync::Arc;

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use crate::app::admin_button::AdminButton;

use crate::app::resources::{AdminButtonMarker, UITracker};

pub struct AdminMenu(pub Vec<Arc<AdminButton>>);

impl AdminMenu {
    pub fn render(&self, world: &mut World) {
        let mut ui_tracker = world.get_resource_mut::<UITracker>().unwrap();
        let buttons = ui_tracker.buttons.clone();
        let admin_bar = ui_tracker.admin_bar().clone();
        drop(ui_tracker);
        for button in buttons {
            world.entity_mut(button).despawn_recursive();
        }
        let mut buttons = Vec::new();
        for admin_button in &self.0 {
            let id = world.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Vh(10.),
                        height: Val::Vh(10.),
                        ..default()
                    },
                    image: UiImage::new(admin_button.texture.clone()),
                    ..default()
                },
                Interaction::default(),
                RelativeCursorPosition::default(),
                AdminButtonMarker(admin_button.clone())))
                .id();
            buttons.push(id);
        }
        world.entity_mut(admin_bar).push_children(&buttons);
        let mut ui_tracker = world.get_resource_mut::<UITracker>().unwrap();
        ui_tracker.buttons = buttons;
    }
}