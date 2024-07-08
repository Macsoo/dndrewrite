use bevy::ecs::system::SystemParam;
use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::WindowRef;
use super::*;

#[derive(SystemParam)]
pub struct UIQuery<'w, 's> {
    pub(super) cameras: Query<'w, 's, (Entity, Mut<'static, Transform>, Mut<'static, OrthographicProjection>, &'static Camera, &'static GlobalTransform)>,
    pub(super) window_query: Query<'w, 's, (Entity, Mut<'static, Window>), Without<Camera>>,
    pub(super) layout: Res<'w, resources::HexLayoutResource>,
    pub(super) mouse_button: Res<'w, ButtonInput<MouseButton>>,
    pub(super) windows: Option<Res<'w, resources::Windows>>,
    pub(super) last_pos: ResMut<'w, resources::MouseLastPosition>,
    pub(super) scale: ResMut<'w, resources::Scale>,
}

impl<'w, 's> UIQuery<'w, 's> {
    pub fn get_focused_window(&self) -> Option<(Entity, Ref<Window>)> {
        self.window_query.iter().find(|w| w.1.focused)
    }

    pub fn get_focused_window_mut(&mut self) -> Option<(Entity, Mut<Window>)> {
        self.window_query.iter_mut().find(|w| w.1.focused)
    }
    pub fn get_focused_camera(&mut self) -> Option<(Entity, Mut<Transform>, Mut<OrthographicProjection>, &Camera, &GlobalTransform)> {
        let focused = self.get_focused_window()?.0;
        self.cameras.iter_mut().find(|c| match c.3.target {
            RenderTarget::Window(WindowRef::Entity(e)) => e == focused,
            _ => false
        })
    }
}