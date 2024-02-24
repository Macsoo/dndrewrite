use std::borrow::Cow;
use std::ops::Deref;
use bevy::app::AppExit;
use bevy::asset::{LoadedFolder, LoadState};
use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::render::camera::RenderTarget;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::HashMap;
use bevy::window::{ExitCondition, WindowFocused, WindowRef};
use hexx::{Hex, HexLayout, HexOrientation, PlaneMeshBuilder, shapes};

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum AppStates {
    #[default]
    Loading,
    Loaded,
}

#[derive(Resource, Default)]
struct Scale(f32);

#[derive(Resource, Default)]
struct MouseLastPosition(Vec2);

type TileColors = HashMap<String, Vec<Handle<Image>>>;

type TileVariants = HashMap<String, TileColors>;

#[derive(Resource, Default)]
struct Textures {
    location_textures: Vec<Handle<Image>>,
    figure_textures: Vec<Handle<Image>>,
    tile_textures: HashMap<String, TileVariants>,
}

impl Textures {
    fn push_tile(
        &mut self,
        tile_type: Cow<str>,
        tile_variant: Cow<str>,
        tile_color: Cow<str>,
        handles: Vec<Handle<Image>>,
    ) {
        let tile_variants = if let Some(tv) = self.tile_textures.get_mut(&*tile_type) {
            tv
        } else {
            self.tile_textures.entry(tile_type.into_owned())
                .insert(HashMap::new())
                .into_mut()
        };
        let tile_colors = if let Some(tc) = tile_variants.get_mut(&*tile_variant) {
            tc
        } else {
            tile_variants.entry(tile_variant.into_owned())
                .insert(HashMap::new())
                .into_mut()
        };
        tile_colors.insert(tile_color.into_owned(), handles);
    }

    fn get_tile_types(&self) -> impl Iterator<Item=&str> {
        self.tile_textures.keys().map(String::as_str)
    }

    fn get_tile_variants(&self, tile_type: &str) -> Option<impl Iterator<Item=&str>> {
        self.tile_textures.get(tile_type).map(HashMap::keys).map(|x| x.map(String::as_str))
    }

    fn get_tile_colors(
        &self,
        tile_type: &str,
        tile_variants: &str,
    ) -> Option<impl Iterator<Item=&str>> {
        self.tile_textures.get(tile_type)
            .and_then(|hm| hm.get(tile_variants))
            .map(HashMap::keys)
            .map(|x| x.map(String::as_str))
    }

    fn get_tile_textures(
        &self,
        tile_type: &str,
        tile_variants: &str,
        tile_color: &str,
    ) -> Option<&[Handle<Image>]> {
        self.tile_textures.get(tile_type)
            .and_then(|hm| hm.get(tile_variants))
            .and_then(|hm| hm.get(tile_color))
            .map(Vec::as_slice)
    }

    fn get_location(&self, index: usize) -> Option<&Handle<Image>> {
        self.location_textures.get(index)
    }

    fn get_figure(&self, index: usize) -> Option<&Handle<Image>> {
        self.figure_textures.get(index)
    }
}

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
    mut textures: ResMut<Textures>,
    mut loaded_folders: ResMut<Assets<LoadedFolder>>,
    mut next_state: ResMut<NextState<AppStates>>,
) {
    for ev in asset_event.read() {
        let AssetEvent::LoadedWithDependencies { id } = ev else { continue; };
        let loaded_folder = loaded_folders.remove(*id).unwrap();
        let (name, _) = folders_loading.remove(id);
        info!("Loaded folder '{}'.", name);
        if let Some(name) = name.strip_prefix("pointy.") {
            let Some((tile_name, color)) = name.split_once('.') else { continue; };
            let Some((tile_type, tile_variant)) = tile_name.rsplit_once('_') else { continue; };
            let handles: Vec<Handle<Image>> = loaded_folder.handles
                .into_iter()
                .map(UntypedHandle::typed)
                .collect();
            textures.push_tile(tile_type.into(), tile_variant.into(), color.into(), handles);
        } else {
            let Some(name) = name.strip_prefix("overlay_")
                .and_then(|s| s.strip_suffix(".standard_full")) else { continue; };
            let mut handles: Vec<Handle<Image>> = loaded_folder.handles.into_iter()
                .map(|h| h.typed())
                .collect();
            handles.sort_by_cached_key(|h| {
                let Some(asset_path) = h.path() else { return u32::MAX; };
                let Some(file_name_os) = asset_path.path().file_name() else { return u32::MAX; };
                let Some(file_name) = file_name_os.to_str() else { return u32::MAX; };
                let Some(without_extension) = file_name.strip_suffix(".png") else { return u32::MAX; };
                let Some(split) = without_extension.rsplit_once('_') else { return u32::MAX; };
                let Ok(index) = split.1.parse::<u32>() else { return u32::MAX; };
                index
            });
            if name == "figures" {
                textures.figure_textures.extend(handles.into_iter());
            } else if name == "locations" {
                textures.location_textures.extend(handles.into_iter());
            }
        };
        if folders_loading.0.is_empty() {
            next_state.set(AppStates::Loaded);
        }
    }
}

fn setup(
    mut commands: Commands,
    textures: Res<Textures>,
) {
    let admin_window = commands.spawn(Window {
        title: String::from("Faerûn - Admin"),
        ..default()
    }).id();
    let admin_camera = commands.spawn(Camera2dBundle {
        camera: Camera {
            target: RenderTarget::Window(WindowRef::Entity(admin_window)),
            ..default()
        },
        ..default()
    }).id();
    let user_window = commands.spawn(Window {
        title: String::from("Faerûn"),
        ..default()
    }).id();
    let user_camera = commands.spawn(Camera2dBundle {
        camera: Camera {
            target: RenderTarget::Window(WindowRef::Entity(user_window)),
            ..default()
        },
        ..default()
    }).id();
    let layout = HexLayout {
        hex_size: Vec2::splat(105. * 3f32.sqrt()),
        orientation: HexOrientation::Pointy,
        ..default()
    };
    let handles = &textures.location_textures[..];
    let _ = Hex::default().spiral_range(0..10)
        .take(handles.len())
        .enumerate()
        .map(|(i, hex)| {
            let pos = layout.hex_to_world_pos(hex);
            commands.spawn(SpriteBundle {
                texture: handles[i].clone(),
                transform: Transform::from_xyz(pos.x, pos.y, 0.),
                ..default()
            });
        })
        .collect::<Vec<_>>();
}

#[derive(Resource, Default)]
struct Focus(Option<Entity>);

fn focus_checker(
    mut event_reader: EventReader<WindowFocused>,
    mut focus: ResMut<Focus>,
) {
    for ev in event_reader.read() {
        if !ev.focused { continue }
        focus.0 = Some(ev.window);
    }
}

fn move_camera(
    mut cameras: Query<(&mut Transform, &Camera)>,
    time: Res<Time>,
    focus: Res<Focus>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (mut transform, camera) in cameras.iter_mut() {
        let RenderTarget::Window(window) = camera.target else { continue; };
        let WindowRef::Entity(entity) = window else { continue; };
        if focus.0 != Some(entity) { continue; }
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
}

fn zoom(
    mut cameras: Query<(&mut OrthographicProjection, &Camera)>,
    time: Res<Time>,
    focus: Res<Focus>,
    mut scale: ResMut<Scale>,
    mut scroll: EventReader<MouseWheel>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    for ev in scroll.read() {
        for (mut proj, camera) in cameras.iter_mut() {
            let RenderTarget::Window(window) = camera.target else { continue };
            let WindowRef::Entity(entity) = window else { continue };
            if focus.0 != Some(entity) { continue }
            match ev.unit {
                MouseScrollUnit::Line => {
                    scale.0 -= time.delta().as_secs_f32() * ev.y;
                    scale.0 = scale.0.clamp(-2., 4.);
                    proj.scale = scale.0.exp();
                }
                MouseScrollUnit::Pixel => {}
            }
        }
    }
}

fn map_drag(
    mut mouse_movement: EventReader<CursorMoved>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cameras: Query<(&mut Transform, &Camera)>,
    mut last_pos: ResMut<MouseLastPosition>,
    focus: Res<Focus>,
    scale: Res<Scale>,
) {
    for ev in mouse_movement.read() {
        for (mut transform, camera) in cameras.iter_mut() {
            let RenderTarget::Window(window) = camera.target else { continue };
            let WindowRef::Entity(entity) = window else { continue };
            if focus.0 != Some(entity) { continue }
            if mouse_button.pressed(MouseButton::Left) {
                let d = ev.position - last_pos.0;
                let d = d * scale.0.exp();
                transform.translation += Vec3::new(-d.x, d.y, 0.);
                last_pos.0 = ev.position;
            }
        }
    }
}

fn detect_press(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut last_pos: ResMut<MouseLastPosition>,
    windows: Query<&Window>,
) {
    for window in windows.iter() {
        if mouse_button.just_pressed(MouseButton::Left) {
            if let Some(pos) = window.cursor_position() {
                last_pos.0 = pos;
            }
        }
    }
}

fn exit_on_esc(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit_event: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit_event.send(AppExit);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: None,
            close_when_requested: true,
            exit_condition: ExitCondition::DontExit,
        }))
        .init_resource::<Scale>()
        .init_resource::<MouseLastPosition>()
        .init_resource::<Textures>()
        .init_resource::<FoldersLoading>()
        .init_resource::<Focus>()
        .init_state::<AppStates>()
        .add_systems(OnEnter(AppStates::Loading), load)
        .add_systems(Update, check_loading.run_if(in_state(AppStates::Loading)))
        .add_systems(OnEnter(AppStates::Loaded), setup)
        .add_systems(FixedUpdate, (
            move_camera,
            zoom,
            (detect_press, map_drag).chain(),
        ).run_if(in_state(AppStates::Loaded)))
        .add_systems(Update, (
            exit_on_esc,
            focus_checker,
        ).run_if(in_state(AppStates::Loaded)))
        .run();
}
