use std::ops::Deref;
use bevy::asset::{LoadedFolder, LoadState};
use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::HashMap;
use hexx::{Hex, HexLayout, HexOrientation, PlaneMeshBuilder, shapes};

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum AppStates {
    #[default]
    Loading,
    Loaded
}

#[derive(Resource, Default)]
struct Scale(f32);

#[derive(Resource, Default)]
struct MouseLastPosition(Vec2);

#[derive(Resource, Default)]
struct TileHandles(Vec<Handle<Image>>);

#[derive(Resource, Default)]
struct FoldersLoading(Vec<(String, Handle<LoadedFolder>)>);

impl FoldersLoading {
    fn remove(&mut self, id: &AssetId<LoadedFolder>) -> (String, Handle<LoadedFolder>) {
        let index = self.0.iter()
            .position(|(_, h)| h.id() == *id)
            .unwrap();
        self.0.swap_remove(index)
    }
}

fn load(
    asset_server: Res<AssetServer>,
    mut folders_loading: ResMut<FoldersLoading>,
) {
    let Ok(folders) = std::fs::read_dir("assets") else { panic!("No 'assets' folder.") };
    for folder in folders {
        let Ok(folder) = folder else {
            error!("IO error `{}` while loading asset folder.", folder.unwrap_err().kind());
            continue;
        };
        let folder_name = folder.file_name().to_string_lossy().into_owned();
        let path = folder.file_name();
        let folder_handle = asset_server.load_folder(path.to_string_lossy().into_owned());
        folders_loading.0.push((folder_name, folder_handle));
    }
}

fn check_loading(
    mut asset_event: EventReader<AssetEvent<LoadedFolder>>,
    mut folders_loading: ResMut<FoldersLoading>,
    mut tiles: ResMut<TileHandles>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut next_state: ResMut<NextState<AppStates>>,
) {
    for ev in asset_event.read() {
        let AssetEvent::LoadedWithDependencies { id } = ev else { continue };
        let loaded_folder = loaded_folders.get(*id).unwrap();
        let (name, _) = folders_loading.remove(id);
        info!("Loaded folder '{}'.", name);
        for handle in &loaded_folder.handles {
            tiles.0.push(handle.clone().typed());
        }
        if folders_loading.0.is_empty() {
            next_state.set(AppStates::Loaded);
        }
    }
}

fn setup(
    mut commands: Commands,
    tiles: Res<TileHandles>,
) {
    let layout = HexLayout {
        hex_size: Vec2::splat(420. * 3f32.sqrt() / 2.),
        orientation: HexOrientation::Pointy,
        ..default()
    };
    info!("The len is: {}", tiles.0.len());
    let _ = Hex::default().spiral_range(0..100)
        .take(tiles.0.len())
        .enumerate()
        .map(|(i, hex)| {
            let pos = layout.hex_to_world_pos(hex);
            commands.spawn(SpriteBundle {
                texture: tiles.0.get(i).unwrap().clone(),
                transform: Transform::from_xyz(pos.x, pos.y, 0.),
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
    let vel = 10_000. * time.delta().as_secs_f32();
    if keys.pressed(KeyCode::W) {
        camera.translation.y += vel;
    }
    if keys.pressed(KeyCode::A) {
        camera.translation.x -= vel;
    }
    if keys.pressed(KeyCode::S) {
        camera.translation.y -= vel;
    }
    if keys.pressed(KeyCode::D) {
        camera.translation.x += vel;
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
                scale.0 = scale.0.clamp(-2., 4.);
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
    window: Query<&Window>,
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
        .init_resource::<Scale>()
        .init_resource::<MouseLastPosition>()
        .init_resource::<TileHandles>()
        .init_resource::<FoldersLoading>()
        .add_state::<AppStates>()
        .add_systems(OnEnter(AppStates::Loading), load)
        .add_systems(Update, check_loading.run_if(in_state(AppStates::Loading)))
        .add_systems(OnEnter(AppStates::Loaded), setup)
        .add_systems(FixedUpdate, (
            move_camera,
            zoom,
            (detect_press, map_drag).chain(),
        ).run_if(in_state(AppStates::Loaded)))
        .run();
}
