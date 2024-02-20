use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::winit::WinitSettings;
use hexx::{Hex, HexLayout, HexOrientation, PlaneMeshBuilder, shapes};

#[derive(Resource)]
struct Scale(f32);

#[derive(Resource)]
struct MouseLastPosition(Vec2);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let layout = HexLayout {
        hex_size: Vec2::splat(32.),
        orientation: HexOrientation::Pointy,
        ..default()
    };
    let mesh_info = PlaneMeshBuilder::new(&layout)
        .facing(Vec3::Z)
        .with_scale(Vec3::splat(0.95))
        .center_aligned()
        .build();
    let mesh = Mesh::new(PrimitiveTopology::TriangleList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs)
        .with_indices(Some(Indices::U16(mesh_info.indices)));
    let mesh_handle = meshes.add(mesh);
    let white_handle = materials.add(Color::WHITE.into());
    let _ = shapes::hexagon(Hex::new(0, 0), 4)
        .map(|hex| {
            let pos = layout.hex_to_world_pos(hex);
            commands.spawn(MaterialMesh2dBundle {
                mesh: mesh_handle.clone().into(),
                transform: Transform::from_xyz(pos.x, pos.y, 0.),
                material: white_handle.clone(),
                ..default()
            });
        })
        .collect::<Vec<_>>();
    commands.spawn(Camera2dBundle::default());
}

fn move_camera(
    mut camera: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
) {
    let mut camera = camera.single_mut();
    let vel = 100. * time.delta().as_secs_f32();
    if keys.pressed(KeyCode::W) {
        camera.translation.y -= vel;
    }
    if keys.pressed(KeyCode::A) {
        camera.translation.x += vel;
    }
    if keys.pressed(KeyCode::S) {
        camera.translation.y += vel;
    }
    if keys.pressed(KeyCode::D) {
        camera.translation.x -= vel;
    }
}

fn zoom(
    mut camera: Query<&mut OrthographicProjection, With<Camera>>,
    time: Res<Time>,
    mut scale: ResMut<Scale>,
    mut scroll: EventReader<MouseWheel>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    let mut camera = camera.single_mut();
    for ev in scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                scale.0 -= time.delta().as_secs_f32() * ev.y;
                scale.0 = scale.0.clamp(-2., 1.);
                camera.scale = scale.0.exp();
            }
            MouseScrollUnit::Pixel => {}
        }
    }
}

fn map_drag(
    mut mouse_movement: EventReader<CursorMoved>,
    mouse_button: Res<Input<MouseButton>>,
    mut camera: Query<&mut Transform, With<Camera>>,
    mut last_pos: ResMut<MouseLastPosition>,
    scale: Res<Scale>,
) {
    let mut camera = camera.single_mut();
    for ev in mouse_movement.read() {
        if mouse_button.pressed(MouseButton::Left) {
            let d = ev.position - last_pos.0;
            let d = d * scale.0.exp();
            camera.translation += Vec3::new(-d.x, d.y, 0.);
            last_pos.0 = ev.position;
        }
    }
}

fn detect_press(
    mouse_button: Res<Input<MouseButton>>,
    mut last_pos: ResMut<MouseLastPosition>,
    window: Query<&Window>
) {
    let window = window.single();
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(pos) = window.cursor_position() {
            last_pos.0 = pos;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scale(0.))
        .insert_resource(MouseLastPosition(Vec2::default()))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (move_camera, zoom, (detect_press, map_drag).chain()))
        .run();
}