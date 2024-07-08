use bevy::math::Vec2;
use bevy::prelude::*;
use hexx::HexLayout;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Scale(pub(super) f32);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct MouseLastPosition(pub(super) Vec2);

#[derive(Resource, Deref, DerefMut)]
pub struct HexLayoutResource(pub(super) HexLayout);

#[derive(Resource)]
pub struct Windows {
    pub admin_window: Entity,
    pub user_window: Entity,
}