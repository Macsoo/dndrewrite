use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::ExitCondition;

mod view;
mod model;
mod app;
mod map;
mod components;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    close_when_requested: false,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .set(LogPlugin {
                    level: Level::INFO,
                    ..default()
                }),
            model::plugins::LoaderPlugin,
            view::plugins::UIPlugin))
        .init_resource::<map::resources::Map>()
        .run();
}
