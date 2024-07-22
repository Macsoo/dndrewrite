use bevy::prelude::*;
use super::*;

pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<resources::FoldersLoading>()
            .init_resource::<resources::TexturesMap>()
            .add_systems(Startup, (
                loading::load,
                loading::register_load_handlers
            ))
            .add_systems(Update, (
                loading::check_load_handlers
                    .run_if(resource_exists::<resources::FoldersLoading>),
                loading::transform_textures
                    .run_if(resource_removed::<resources::FoldersLoading>()),
            ));
    }
}