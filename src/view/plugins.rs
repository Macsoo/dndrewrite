use bevy::prelude::*;
use hexx::{HexLayout, HexOrientation};
use crate::app::admin;
use crate::app::resources::AppLoaded;
use super::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<resources::Scale>()
            .init_resource::<resources::MouseLastPosition>()
            .insert_resource(resources::HexLayoutResource(HexLayout {
                hex_size: Vec2::splat(105. * 3f32.sqrt()),
                orientation: HexOrientation::Pointy,
                ..default()
            }))
            .add_systems(First, ui::setup_ui
                .run_if(resource_added::<AppLoaded>))
            .add_systems(FixedUpdate, (
                ui::move_camera,
                ui::zoom,
                (ui::detect_press, ui::map_drag).chain(),
            )
                .run_if(resource_exists::<AppLoaded>))
            .add_systems(Update, (
                ui::exit_on_esc,
                scrolling_list::mouse_scroll,
                admin::handle_admin,
            ).run_if(resource_exists::<AppLoaded>));
    }
}