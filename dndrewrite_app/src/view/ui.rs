use bevy::app::AppExit;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseWheel;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::texture;
use bevy::ui::RelativeCursorPosition;
use bevy::window::{PrimaryWindow, WindowRef};
use hexx::Hex;
use crate::app::admin_button::AdminButton;
use crate::app::resources::{AdminButtonMarker, AdminMenus, AdminMenuStack, CurrentAdminMenu, UITracker};
use crate::model::id::Id;
use crate::view::query::UIQuery;
use super::*;

pub fn zoom(
    mut ui: UIQuery,
    time: Res<Time>,
    mut scroll: EventReader<MouseWheel>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    for ev in scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                **ui.scale -= time.delta().as_secs_f32() * ev.y;
                **ui.scale = ui.scale.clamp(-2., 4.);
                let ui_sce = ui.scale.exp();
                let mut entity = ui.get_focused_camera().unwrap();
                let mut proj = entity.2;
                proj.scale = ui_sce;
            }
            MouseScrollUnit::Pixel => {}
        }
    }
}

pub fn map_drag(
    mut ui: UIQuery,
    mut mouse_movement: EventReader<CursorMoved>,
) {
    if !ui.mouse_button.pressed(MouseButton::Left) { return; }
    for ev in mouse_movement.read() {
        let d = ev.position - **ui.last_pos;
        let d = d * ui.scale.exp();
        let mut entity = ui.get_focused_camera().unwrap();
        let mut transform = entity.1;
        transform.translation += Vec3::new(-d.x, d.y, 0.);
        **ui.last_pos = ev.position;
    }
}

pub fn detect_press(
    mut ui: UIQuery,
) {
    if !ui.mouse_button.just_pressed(MouseButton::Left) { return; }
    let Some(w) = ui.get_focused_window() else { return; };
    let Some(pos) = w.1.cursor_position() else { return; };
    **ui.last_pos = pos;
}

pub fn exit_on_esc(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit_event: EventWriter<AppExit>,
) {
    if keys.all_pressed([KeyCode::ControlLeft, KeyCode::KeyQ]) {
        exit_event.send(AppExit::Success);
    }
}

pub fn move_camera(
    mut ui: UIQuery,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Some(mut entity) = ui.get_focused_camera() else { return };
    let mut transform = entity.1;
    let vel = 10_000. * time.delta().as_secs_f32();
    if keys.pressed(KeyCode::KeyW) {
        transform.translation.y += vel;
    }
    if keys.pressed(KeyCode::KeyA) {
        transform.translation.x -= vel;
    }
    if keys.pressed(KeyCode::KeyS) {
        transform.translation.y -= vel;
    }
    if keys.pressed(KeyCode::KeyD) {
        transform.translation.x += vel;
    }
}

pub fn get_clicked_hex(
    mut ui: UIQuery,
    admin: bool,
) -> Option<Hex> {
    if !ui.mouse_button.just_pressed(MouseButton::Left) { return None; }
    let windows = ui.windows?;
    let admin_window = windows.admin_window;
    ui.windows = Some(windows);
    let (focused, window) = ui.get_focused_window()?;
    let is_admin_window = focused != admin_window;
    if is_admin_window != admin { return None; };
    let cursor = window.cursor_position()?;
    let camera_entity = ui.get_focused_camera()?;
    let camera = camera_entity.3;
    let transform = camera_entity.4;
    let pos = camera.viewport_to_world_2d(transform, cursor)?;
    Some(ui.layout.world_pos_to_hex(pos))
}

pub fn setup_ui(
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    primary: Query<Entity, With<PrimaryWindow>>,
) {
    commands.entity(primary.single()).despawn_recursive();
    let admin_window = commands.spawn(Window {
        title: String::from("Faerûn - Admin"),
        ..default()
    }).id();
    info!("Admin Window: {}", admin_window);
    let admin_camera = commands.spawn(Camera2dBundle {
        camera: Camera {
            target: RenderTarget::Window(WindowRef::Entity(admin_window)),
            ..default()
        },
        ..default()
    }).id();
    info!("Admin Camera: {}", admin_camera);
    let user_window = commands.spawn(Window {
        title: String::from("Faerûn"),
        ..default()
    }).id();
    info!("User Window: {}", user_window);
    let user_camera = commands.spawn(Camera2dBundle {
        camera: Camera {
            target: RenderTarget::Window(WindowRef::Entity(user_window)),
            ..default()
        },
        ..default()
    }).id();
    info!("User Camera: {}", user_camera);
    let scroll_bar = commands.spawn((NodeBundle {
        style: Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            height: Val::Vh(10.),
            ..default()
        },
        ..default()
    }, scrolling_list::ScrollingList::default())).id();
    let back = commands.spawn((
        TextBundle::from_section(
            "<=",
            TextStyle {
                font_size: 48.,
                ..default()
            }
        ),
        Interaction::default(),
        RelativeCursorPosition::default(),
        AdminButtonMarker(AdminButton {
            texture: images.add(Image::transparent()),
            name: "BACK".to_string(),
            on_click: Box::new(|id| id.init()),
            on_hover: Box::new(|mut window| window.cursor.icon = CursorIcon::Pointer)
        }.into())
    )).id();
    commands.entity(scroll_bar).push_children(&[back]);
    commands.insert_resource(UITracker::new(scroll_bar, back));
    let bar = commands.spawn((NodeBundle {
        style: Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.),
            height: Val::Vh(10.),
            overflow: Overflow::clip_x(),
            ..default()
        },
        background_color: BackgroundColor(Color::BLACK),
        ..default()
    }, Interaction::default())).push_children(&[scroll_bar]).id();
    commands.spawn((NodeBundle {
        style: Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        ..default()
    }, TargetCamera(admin_camera))).push_children(&[bar]);
    commands.insert_resource(resources::Windows {
        admin_window,
        user_window,
    });
    commands.init_resource::<AdminMenuStack>();
    commands.init_resource::<AdminMenus>();
    commands.init_resource::<CurrentAdminMenu>();
}