use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ops::Deref;
use bevy::app::AppExit;
use bevy::asset::{AssetPath, LoadedFolder, LoadState};
use bevy::prelude::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::render::camera::RenderTarget;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
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

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum AdminEditor {
    #[default]
    NotLoaded,
    Choose,
    ChooseTileType,
    ChooseTileVariant,
    ChooseTileColor,
    PlaceTile,
    ChooseOverlayType,
    ChooseOverlay,
    PlaceOverlay,
    ChooseTextPosition,
    WriteText,
}

#[derive(Resource, Default)]
struct Scale(f32);

#[derive(Resource, Default)]
struct MouseLastPosition(Vec2);

type TileColors = BTreeMap<String, Vec<Handle<Image>>>;

type TileVariants = BTreeMap<String, TileColors>;

#[derive(Resource, Default)]
struct Textures {
    overlay_textures: BTreeMap<String, Vec<Handle<Image>>>,
    figure_textures: Vec<Handle<Image>>,
    tile_textures: BTreeMap<String, TileVariants>,
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
            self.tile_textures.insert(tile_type.clone().into_owned(), BTreeMap::new());
            self.tile_textures.get_mut(&*tile_type).unwrap()
        };
        let tile_colors = if let Some(tc) = tile_variants.get_mut(&*tile_variant) {
            tc
        } else {
            tile_variants.insert(tile_variant.clone().into_owned(), BTreeMap::new());
            tile_variants.get_mut(&*tile_variant).unwrap()
        };
        tile_colors.insert(tile_color.into_owned(), handles);
    }

    fn get_tile_types(&self) -> impl Iterator<Item=&str> {
        self.tile_textures.keys().map(String::as_str)
    }

    fn get_tile_variants(&self, tile_type: &str) -> Option<impl Iterator<Item=&str>> {
        self.tile_textures.get(tile_type).map(BTreeMap::keys).map(|x| x.map(String::as_str))
    }

    fn get_tile_colors(
        &self,
        tile_type: &str,
        tile_variants: &str,
    ) -> Option<impl Iterator<Item=&str>> {
        self.tile_textures.get(tile_type)
            .and_then(|hm| hm.get(tile_variants))
            .map(BTreeMap::keys)
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
            let mut handles: Vec<Handle<Image>> = loaded_folder.handles
                .into_iter()
                .map(UntypedHandle::typed)
                .collect();
            handles.sort_by_cached_key(|h| {
                let Some(path) = h.path().map(AssetPath::path) else { return Err(h.clone()) };
                let Some(file_name) = path.file_name() else { return Err(h.clone()) };
                let Some(file_name_str) = file_name.to_str() else { return Err(h.clone()) };
                let Some((without_extension, _)) = file_name_str.rsplit_once('.') else { return Err(h.clone()) };
                let Some((_, number)) = without_extension.rsplit_once('_') else { return Err(h.clone()) };
                let Ok(parsed) = number.parse::<u32>() else { return Err(h.clone()) };
                Ok(parsed)
            });
            if tile_type == "overlay" {
                textures.overlay_textures.insert(tile_variant.to_owned(), handles);
            } else {
                textures.push_tile(tile_type.into(), tile_variant.into(), color.into(), handles);
            }
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
                textures.overlay_textures.insert("locations".to_owned(), handles);
            }
        };
        if folders_loading.0.is_empty() {
            next_state.set(AppStates::Loaded);
        }
    }
}

#[derive(Resource, Default, Debug)]
struct UITracker {
    admin_bar: Option<Entity>,
    back_button: Option<Entity>,
    buttons: Option<Vec<Entity>>,
    chosen_tile_type: Option<String>,
    chosen_tile_variant: Option<String>,
    chosen_tile_color: Option<String>,
    chosen_overlay_type: Option<String>,
    chosen_overlay: Option<usize>,
    chosen_text_position: Option<(Entity, Hex)>,
}

impl UITracker {
    fn back_button(&mut self, commands: &mut Commands) {
        let Some(ui) = self.admin_bar else { return };
        let back = commands.spawn((
            TextBundle::from_section(
                "<=",
                TextStyle {
                    font_size: 48.,
                    ..default()
                }
            ), Interaction::default()
        )).id();
        commands.entity(ui).push_children(&[back]);
        self.back_button = Some(back);
    }
}

fn button_listener(
    interaction_query: Query<(
        Entity,
        &Interaction,
    ), (Changed<Interaction>, With<Node>)>,
    textures: Res<Textures>,
    mut ui_tracker: ResMut<UITracker>,
    current_state: Res<State<AdminEditor>>,
    mut next_state: ResMut<NextState<AdminEditor>>,
) {
    for (entity, interaction) in &interaction_query {
        if *interaction != Interaction::Pressed { continue }
        if ui_tracker.back_button == Some(entity) {
            let next = match current_state.get() {
                AdminEditor::NotLoaded => None,
                AdminEditor::Choose => None,
                AdminEditor::ChooseTileType => Some(AdminEditor::Choose),
                AdminEditor::ChooseTileVariant => Some(AdminEditor::ChooseTileType),
                AdminEditor::ChooseTileColor => Some(AdminEditor::ChooseTileVariant),
                AdminEditor::PlaceTile => Some(AdminEditor::ChooseTileColor),
                AdminEditor::ChooseOverlayType => Some(AdminEditor::Choose),
                AdminEditor::ChooseOverlay => Some(AdminEditor::ChooseOverlayType),
                AdminEditor::PlaceOverlay => Some(AdminEditor::ChooseOverlay),
                AdminEditor::ChooseTextPosition => Some(AdminEditor::Choose),
                AdminEditor::WriteText => Some(AdminEditor::ChooseTextPosition),
            };
            if let Some(next) = next {
                next_state.set(next);
            }
        } else if let Some(n) = ui_tracker.buttons.as_ref()
            .and_then(|b| b.iter().position(|b| *b == entity)) {
            match current_state.get() {
                AdminEditor::NotLoaded => {}
                AdminEditor::Choose if n == 0 => {
                    next_state.set(AdminEditor::ChooseTileType);
                }
                AdminEditor::Choose if n == 1 => {
                    next_state.set(AdminEditor::ChooseOverlayType);
                }
                AdminEditor::Choose => {
                    next_state.set(AdminEditor::ChooseTextPosition);
                }
                AdminEditor::ChooseTileType => {
                    let Some(tile_type) = textures.tile_textures.keys().nth(n) else { continue };
                    ui_tracker.chosen_tile_type = Some(tile_type.clone());
                    next_state.set(AdminEditor::ChooseTileVariant);
                }
                AdminEditor::ChooseTileVariant => {
                    let Some(tile_type) = &ui_tracker.chosen_tile_type else { continue };
                    let Some(tile_variant) = textures.tile_textures.get(tile_type).unwrap().keys().nth(n) else { continue };
                    ui_tracker.chosen_tile_variant = Some(tile_variant.clone());
                    next_state.set(AdminEditor::ChooseTileColor);
                }
                AdminEditor::ChooseTileColor => {
                    let Some(tile_type) = &ui_tracker.chosen_tile_type else { continue };
                    let Some(tile_variant) = &ui_tracker.chosen_tile_variant else { continue };
                    let Some(tile_color) = textures.tile_textures.get(tile_type).unwrap().get(tile_variant).unwrap().keys().nth(n) else { continue };
                    ui_tracker.chosen_tile_color = Some(tile_color.clone());
                    next_state.set(AdminEditor::PlaceTile);
                }
                AdminEditor::PlaceTile => {}
                AdminEditor::ChooseOverlayType => {
                    let Some(overlay_type) = textures.overlay_textures.keys().nth(n) else { continue };
                    ui_tracker.chosen_overlay_type = Some(overlay_type.clone());
                    next_state.set(AdminEditor::ChooseOverlay);
                }
                AdminEditor::ChooseOverlay => {
                    ui_tracker.chosen_overlay = Some(n);
                    next_state.set(AdminEditor::PlaceOverlay);
                }
                AdminEditor::PlaceOverlay => {}
                AdminEditor::ChooseTextPosition => {}
                AdminEditor::WriteText => {}
            }
        }
    }
}

enum Size {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

struct MonsterBase {
    name: String,
    size: Size,
    hp: u16,
}

enum CombatantKind {
    Player,
    Summon,
    Enemy,
    Ally,
}

struct Combatant {
    kind: CombatantKind,
    texture: usize,
    summons: Vec<Entity>,
    summoner: Option<Entity>,
    base: Option<MonsterBase>,
}

#[derive(Resource, Default)]
struct Party {
    combatants: Vec<(Hex, Combatant)>,
}

#[derive(Resource, Default, Debug)]
struct Map {
    tiles: HashMap<Hex, (Entity, String)>,
    overlay: HashMap<Hex, HashMap<String, (Entity, usize)>>,
    text: HashMap<Hex, (Entity, String)>,
    combatants: HashMap<Hex, (Entity, Combatant)>,
}

fn place_tile(
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    layout: Res<HexLayoutResource>,
    focus: Res<Focus>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    ui_tracker: Res<UITracker>,
    textures: Res<Textures>,
    mut map: ResMut<Map>,
    mut commands: Commands,
    windows_res: Option<Res<Windows>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) { return }
    let Some(windows_res) = windows_res else { return };
    let Some(tile_type) = &ui_tracker.chosen_tile_type else { return };
    let Some(tile_variant) = &ui_tracker.chosen_tile_variant else { return };
    let Some(tile_color) = &ui_tracker.chosen_tile_color else { return };
    let Some(handles) = textures.get_tile_textures(tile_type, tile_variant, tile_color) else { return };
    let Ok(time) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else { unreachable!() };
    let Some(handle) = handles.get(time.as_secs() as usize % handles.len()) else { unreachable!() };
    for (camera, transform) in &cameras {
        let RenderTarget::Window(window) = camera.target else { continue };
        let WindowRef::Entity(entity) = window else { continue };
        if focus.0 != Some(entity) { continue }
        if entity != windows_res.admin_window { return }
        let Ok(window) = windows.get(entity) else { return };
        let Some(cursor) = window.cursor_position() else { return };
        let Some(pos) = camera.viewport_to_world_2d(transform, cursor) else { return };
        let hex = layout.0.world_pos_to_hex(pos);
        let pos = layout.0.hex_to_world_pos(hex);
        let id = commands.spawn(SpriteBundle {
            transform: Transform::from_xyz(pos.x, pos.y, 0.),
            texture: handle.clone(),
            ..default()
        }).id();
        let tile = format!("{}_{}.{}", tile_type, tile_variant, tile_color);
        if let Some((id, _)) = map.tiles.insert(hex, (id, tile)) {
            commands.entity(id).despawn_recursive();
        }
    }
}

fn place_overlay(
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    layout: Res<HexLayoutResource>,
    focus: Res<Focus>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    ui_tracker: Res<UITracker>,
    textures: Res<Textures>,
    mut map: ResMut<Map>,
    mut commands: Commands,
    windows_res: Option<Res<Windows>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) { return }
    let Some(windows_res) = windows_res else { return };
    let Some(overlay_type) = &ui_tracker.chosen_overlay_type else { return };
    let Some(overlay) = ui_tracker.chosen_overlay else { return };
    let Some(overlays) = textures.overlay_textures.get(overlay_type) else { return };
    let Some(handle) = overlays.get(overlay) else { return };
    for (camera, transform) in &cameras {
        let RenderTarget::Window(window) = camera.target else { continue };
        let WindowRef::Entity(entity) = window else { continue };
        if focus.0 != Some(entity) { continue }
        if entity != windows_res.admin_window { return }
        let Ok(window) = windows.get(entity) else { return };
        let Some(cursor) = window.cursor_position() else { return };
        let Some(pos) = camera.viewport_to_world_2d(transform, cursor) else { return };
        let hex = layout.0.world_pos_to_hex(pos);
        let pos = layout.0.hex_to_world_pos(hex);
        let id = commands.spawn(SpriteBundle {
            transform: Transform::from_xyz(pos.x, pos.y, 1.),
            texture: handle.clone(),
            ..default()
        }).id();
        map.overlay.entry(hex).and_modify(|map|  {
            if let Some((id, _)) = map.insert(overlay_type.clone(), (id, overlay)) {
                commands.entity(id).despawn_recursive();
            }
        }).or_insert_with(|| HashMap::from([(overlay_type.clone(), (id, overlay))]));
    }
}

fn admin_choose_text_position(
    cameras: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    layout: Res<HexLayoutResource>,
    focus: Res<Focus>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut ui_tracker: ResMut<UITracker>,
    windows_res: Option<Res<Windows>>,
    mut next_state: ResMut<NextState<AdminEditor>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) { return }
    let Some(windows_res) = windows_res else { return };
    for (camera, transform) in &cameras {
        let RenderTarget::Window(window) = camera.target else { continue };
        let WindowRef::Entity(entity) = window else { continue };
        if focus.0 != Some(entity) { continue }
        if entity != windows_res.admin_window { return }
        let Ok(window) = windows.get(entity) else { return };
        let Some(cursor) = window.cursor_position() else { return };
        let Some(pos) = camera.viewport_to_world_2d(transform, cursor) else { return };
        let hex = layout.0.world_pos_to_hex(pos);
        let pos = layout.0.hex_to_world_pos(hex);
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, layout.0.hex_corners(Hex::default()).into_iter().map(|v| [v.x, v.y, 0.]).collect::<Vec<_>>());
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.2578125, 0.52734375, 0.95703125, 0.15]; 6]);
        mesh.insert_indices(Indices::U16(vec![0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5]));
        let id = commands.spawn(MaterialMesh2dBundle {
            mesh: meshes.add(mesh).into(),
            material: materials.add(ColorMaterial::default()),
            transform: Transform::from_xyz(pos.x, pos.y, 0.),
            ..default()
        }).id();
        ui_tracker.chosen_text_position = Some((id, hex));
        next_state.set(AdminEditor::WriteText);
    }
}

fn place_text(
    mut ev_char: EventReader<ReceivedCharacter>,
    kbd: Res<ButtonInput<KeyCode>>,
    mut ui_tracker: ResMut<UITracker>,
    mut ui_text: Query<&mut Text, With<Node>>,
    windows: Option<Res<Windows>>,
    mut commands: Commands,
    layout: Res<HexLayoutResource>,
    mut map: ResMut<Map>,
    mut next: ResMut<NextState<AdminEditor>>,
) {
    let Some(ui_buttons) = &ui_tracker.buttons else { return };
    let Some(ui) = ui_buttons.first() else { return };
    let Some((chosen, hex)) = ui_tracker.chosen_text_position else { return };
    let Some(windows) = windows else { return };
    let Ok(mut text) = ui_text.get_mut(*ui) else { return };
    let Some(string) = text.sections.get_mut(0).map(|x| &mut x.value) else { return };
    if kbd.just_pressed(KeyCode::Enter) {
        let pos = layout.0.hex_to_world_pos(hex);
        let val = string.clone();
        let id = commands.spawn(Text2dBundle {
            transform: Transform::from_xyz(pos.x, pos.y, 1.),
            text: text.clone(),
            ..default()
        }).id();
        if let Some((id, _)) = map.text.insert(hex, (id, val)) {
            commands.entity(id).despawn_recursive();
        };
        commands.entity(chosen).despawn_recursive();
        ui_tracker.chosen_text_position = None;
        next.set(AdminEditor::Choose);
        return;
    }
    if kbd.just_pressed(KeyCode::Backspace) {
        string.pop();
    }
    for ev in ev_char.read() {
        if ev.window != windows.admin_window { continue };
        let Some(char) = ev.char.chars().next() else { continue };
        if char.is_control() { return };
        string.push(char);
    }
}

struct AdminUI;

impl Plugin for AdminUI {
    fn build(&self, app: &mut App) {
        app
            .init_state::<AdminEditor>()
            .add_systems(Update, button_listener.run_if(in_state(AppStates::Loaded)))
            .add_systems(OnEnter(AdminEditor::Choose), admin_enter_choose)
            .add_systems(OnExit(AdminEditor::Choose), admin_change_menu)
            .add_systems(OnEnter(AdminEditor::ChooseTileType), admin_enter_choose_tile)
            .add_systems(OnExit(AdminEditor::ChooseTileType), admin_change_menu)
            .add_systems(OnEnter(AdminEditor::ChooseTileVariant), admin_enter_choose_variant)
            .add_systems(OnExit(AdminEditor::ChooseTileVariant), admin_change_menu)
            .add_systems(OnEnter(AdminEditor::ChooseTileColor), admin_enter_choose_color)
            .add_systems(OnExit(AdminEditor::ChooseTileColor), admin_change_menu)
            .add_systems(OnEnter(AdminEditor::PlaceTile), admin_enter_place_tile)
            .add_systems(OnExit(AdminEditor::PlaceTile), admin_change_menu)
            .add_systems(OnEnter(AdminEditor::ChooseOverlayType), admin_enter_choose_overlay_type)
            .add_systems(OnExit(AdminEditor::ChooseOverlayType), admin_change_menu)
            .add_systems(OnEnter(AdminEditor::ChooseOverlay), admin_enter_choose_overlay)
            .add_systems(OnExit(AdminEditor::ChooseOverlay), admin_change_menu)
            .add_systems(OnEnter(AdminEditor::PlaceOverlay), admin_enter_place_overlay)
            .add_systems(OnExit(AdminEditor::PlaceOverlay), admin_change_menu)
            .add_systems(OnEnter(AdminEditor::ChooseTextPosition), admin_enter_choose_text_position)
            .add_systems(OnExit(AdminEditor::ChooseTextPosition), admin_change_menu)
            .add_systems(OnEnter(AdminEditor::WriteText), admin_enter_write_text)
            .add_systems(OnExit(AdminEditor::WriteText), admin_change_menu)
            .add_systems(Update, place_tile.run_if(in_state(AdminEditor::PlaceTile).and_then(mouse_not_over_admin_bar)))
            .add_systems(Update, place_overlay.run_if(in_state(AdminEditor::PlaceOverlay).and_then(mouse_not_over_admin_bar)))
            .add_systems(Update, admin_choose_text_position.run_if(in_state(AdminEditor::ChooseTextPosition).and_then(mouse_not_over_admin_bar)))
            .add_systems(Update, place_text.run_if(in_state(AdminEditor::WriteText).and_then(mouse_not_over_admin_bar)))
        ;
    }
}

fn mouse_not_over_admin_bar(
    ui_tracker: Res<UITracker>,
    focus: Res<Focus>,
    windows: Option<Res<Windows>>,
    scroll_bar_query: Query<&Parent, With<ScrollingList>>,
    nodes: Query<(&Interaction), With<Node>>,
) -> bool {
    let Some(ui) = ui_tracker.admin_bar else { return false };
    let Ok(parent) = scroll_bar_query.get(ui).map(Parent::get) else { return false };
    let Some(windows) = windows else { return false };
    let Some(entity) = focus.0 else { return false };
    if entity != windows.admin_window { return false }
    let Ok(interaction) = nodes.get(parent) else { return false };
    if *interaction == Interaction::None { true }
    else { false }
}

fn spawn_image_button(commands: &mut Commands, children: &mut Vec<Entity>, texture: Handle<Image>) {
    let button = commands.spawn((ImageBundle {
        style: Style {
            width: Val::Vh(10.),
            height: Val::Vh(10.),
            ..default()
        },
        image: UiImage::new(texture),
        ..default()
    }, Interaction::default())).id();
    children.push(button);
}

fn admin_enter_choose(
    mut commands: Commands,
    textures: Res<Textures>,
    mut ui_tracker: ResMut<UITracker>,
) {
    let Some(ui) = ui_tracker.admin_bar else { return };
    let Some(first_tile_type) = textures.tile_textures.values().next() else { return; };
    let Some(first_variant) = first_tile_type.values().next() else { return; };
    let Some(first_color) = first_variant.values().next() else { return; };
    let Some(first_tile_texture) = first_color.first() else { return; };
    let Some(first_overlay_type) = textures.overlay_textures.values().next() else { return };
    let Some(first_overlay_texture) = first_overlay_type.first() else { return };
    let mut choose_buttons = Vec::new();
    spawn_image_button(&mut commands, &mut choose_buttons, first_tile_texture.clone());
    spawn_image_button(&mut commands, &mut choose_buttons, first_overlay_texture.clone());
    let text = commands.spawn((TextBundle {
        text: Text::from_section("T", TextStyle {
            font_size: 48.,
            color: Color::WHITE,
            ..default()
        }),
        ..default()
    }, Interaction::default())).id();
    choose_buttons.push(text);
    commands.entity(ui).push_children(&choose_buttons[..]);
    ui_tracker.buttons = Some(choose_buttons);
}

fn admin_enter_choose_tile(
    mut commands: Commands,
    textures: Res<Textures>,
    mut ui_tracker: ResMut<UITracker>,
) {
    let Some(ui) = ui_tracker.admin_bar else { return };
    ui_tracker.back_button(&mut commands);
    let mut tile_type_buttons = Vec::new();
    for variants in textures.tile_textures.values() {
        let Some(first_variant) = variants.values().next() else { return; };
        let Some(first_color) = first_variant.values().next() else { return; };
        let Some(first_texture) = first_color.first() else { return; };
        spawn_image_button(&mut commands, &mut tile_type_buttons, first_texture.clone());
    }
    commands.entity(ui).push_children(&tile_type_buttons[..]);
    ui_tracker.buttons = Some(tile_type_buttons);
}

fn admin_change_menu(
    mut commands: Commands,
    mut ui_tracker: ResMut<UITracker>,
    mut scrolling_lists: Query<(&mut ScrollingList, &mut Style)>,
) {
    if let Some(back_button) = ui_tracker.back_button.take() {
        commands.entity(back_button).despawn_recursive();
    }
    let Some(buttons) = ui_tracker.buttons.take() else { return };
    for button in buttons {
        commands.entity(button).despawn_recursive();
    }
    for (mut scrolling_list, mut style) in &mut scrolling_lists {
        scrolling_list.position = 0.;
        style.left = Val::Px(0.);
    }
}

fn admin_enter_choose_variant(
    mut commands: Commands,
    textures: Res<Textures>,
    mut ui_tracker: ResMut<UITracker>,
) {
    let Some(ui) = ui_tracker.admin_bar else { return };
    ui_tracker.back_button(&mut commands);
    let Some(tile_type) = &ui_tracker.chosen_tile_type else { return };
    let mut tile_variant_buttons = Vec::new();
    for colors in textures.tile_textures.get(tile_type).unwrap().values() {
        let Some(first_color) = colors.values().next() else { return; };
        let Some(first_texture) = first_color.first() else { return; };
        spawn_image_button(&mut commands, &mut tile_variant_buttons, first_texture.clone());
    }
    commands.entity(ui).push_children(&tile_variant_buttons[..]);
    ui_tracker.buttons = Some(tile_variant_buttons);
}

fn admin_enter_choose_color(
    mut commands: Commands,
    textures: Res<Textures>,
    mut ui_tracker: ResMut<UITracker>,
) {
    let Some(ui) = ui_tracker.admin_bar else { return };
    ui_tracker.back_button(&mut commands);
    let Some(tile_type) = &ui_tracker.chosen_tile_type else { return };
    let Some(tile_variant) = &ui_tracker.chosen_tile_variant else { return };
    let mut tile_color_buttons = Vec::new();
    for colors in textures.tile_textures.get(tile_type).unwrap().get(tile_variant).unwrap().values() {
        let Some(first_texture) = colors.first() else { return; };
        spawn_image_button(&mut commands, &mut tile_color_buttons, first_texture.clone());
    }
    commands.entity(ui).push_children(&tile_color_buttons[..]);
    ui_tracker.buttons = Some(tile_color_buttons);
}

fn admin_enter_place_tile(
    mut commands: Commands,
    textures: Res<Textures>,
    mut ui_tracker: ResMut<UITracker>,
) {
    let Some(ui) = ui_tracker.admin_bar else { return };
    ui_tracker.back_button(&mut commands);
    let Some(tile_type) = &ui_tracker.chosen_tile_type else { return };
    let Some(tile_variant) = &ui_tracker.chosen_tile_variant else { return };
    let Some(tile_color) = &ui_tracker.chosen_tile_color else { return };
    let mut tile_textures = Vec::new();
    for texture in textures.get_tile_textures(tile_type, tile_variant, tile_color).unwrap() {
        spawn_image_button(&mut commands, &mut tile_textures, texture.clone());
    }
    commands.entity(ui).push_children(&tile_textures[..]);
    ui_tracker.buttons = Some(tile_textures);
}

fn admin_enter_choose_overlay_type(
    mut commands: Commands,
    textures: Res<Textures>,
    mut ui_tracker: ResMut<UITracker>,
) {
    let Some(ui) = ui_tracker.admin_bar else { return };
    ui_tracker.back_button(&mut commands);
    let mut overlay_textures = Vec::new();
    for overlay_types in textures.overlay_textures.values() {
        let Some(first_texture) = overlay_types.first() else { continue };
        spawn_image_button(&mut commands, &mut overlay_textures, first_texture.clone());
    }
    commands.entity(ui).push_children(&overlay_textures[..]);
    ui_tracker.buttons = Some(overlay_textures);
}

fn admin_enter_choose_overlay(
    mut commands: Commands,
    textures: Res<Textures>,
    mut ui_tracker: ResMut<UITracker>,
) {
    let Some(ui) = ui_tracker.admin_bar else { return };
    ui_tracker.back_button(&mut commands);
    let Some(overlay_type) = &ui_tracker.chosen_overlay_type else { return };
    let mut overlay_textures = Vec::new();
    for overlay_texture in textures.overlay_textures.get(overlay_type).unwrap() {
        spawn_image_button(&mut commands, &mut overlay_textures, overlay_texture.clone());
    }
    commands.entity(ui).push_children(&overlay_textures[..]);
    ui_tracker.buttons = Some(overlay_textures);
}

fn admin_enter_place_overlay(
    mut commands: Commands,
    textures: Res<Textures>,
    mut ui_tracker: ResMut<UITracker>,
) {
    let Some(ui) = ui_tracker.admin_bar else { return };
    ui_tracker.back_button(&mut commands);
    let Some(overlay_type) = &ui_tracker.chosen_overlay_type else { return };
    let Some(overlay) = ui_tracker.chosen_overlay else { return };
    let mut overlay_textures = Vec::new();
    let overlay_texture = textures.overlay_textures.get(overlay_type).unwrap().get(overlay).unwrap();
    spawn_image_button(&mut commands, &mut overlay_textures, overlay_texture.clone());
    commands.entity(ui).push_children(&overlay_textures[..]);
    ui_tracker.buttons = Some(overlay_textures);
}

fn admin_enter_choose_text_position(
    mut commands: Commands,
    mut ui_tracker: ResMut<UITracker>,
) {
    let Some(ui) = ui_tracker.admin_bar else { return };
    ui_tracker.back_button(&mut commands);
    let mut text = Vec::new();
    let help_text = commands.spawn(TextBundle {
        text: Text::from_section("CHOOSE A HEX FOR THE TEXT", TextStyle {
            font_size: 48.,
            color: Color::WHITE,
            ..default()
        }),
        ..default()
    }).id();
    text.push(help_text);
    commands.entity(ui).push_children(&text[..]);
    ui_tracker.buttons = Some(text);
}

fn admin_enter_write_text(
    mut commands: Commands,
    mut ui_tracker: ResMut<UITracker>,
) {
    let Some(ui) = ui_tracker.admin_bar else { return };
    ui_tracker.back_button(&mut commands);
    let mut overlay_textures = Vec::new();
    let button = commands.spawn((TextBundle {
        style: Style {
            height: Val::Vh(10.),
            ..default()
        },
        text: Text::from_section("", TextStyle {
            font_size: 48.,
            ..default()
        }),
        ..default()
    })).id();
    overlay_textures.push(button);
    commands.entity(ui).push_children(&overlay_textures[..]);
    ui_tracker.buttons = Some(overlay_textures);
}

#[derive(Resource)]
struct Windows {
    admin_window: Entity,
    user_window: Entity,
}

#[derive(Component, Default)]
struct ScrollingList {
    position: f32,
}

//Modified copy from https://bevyengine.org/examples/UI%20(User%20Interface)/ui/
fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
            let items_width = list_node.size().x;
            let container_width = query_node.get(parent.get()).unwrap().size().x;
            let max_scroll = (items_width - container_width).max(0.) / 2.;
            let dx = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.x * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.x,
            };
            scrolling_list.position += dx;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, max_scroll);
            style.left = Val::Px(scrolling_list.position);
        }
    }
}

fn setup_ui(
    mut commands: Commands,
    mut ui_tracker: ResMut<UITracker>,
    mut next_state: ResMut<NextState<AdminEditor>>,
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
    }, ScrollingList::default())).id();
    ui_tracker.admin_bar = Some(scroll_bar);
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
    commands.insert_resource(Windows {
        admin_window,
        user_window,
    });
    next_state.set(AdminEditor::Choose);
}

#[derive(Resource)]
struct HexLayoutResource(HexLayout);

#[derive(Resource, Default)]
struct Focus(Option<Entity>);

fn focus_checker(
    mut event_reader: EventReader<WindowFocused>,
    mut focus: ResMut<Focus>,
) {
    for ev in event_reader.read() {
        if !ev.focused { continue; }
        focus.0 = Some(ev.window);
    }
}

fn move_camera(
    mut cameras: Query<(&mut Transform, &Camera)>,
    time: Res<Time>,
    focus: Res<Focus>,
    keys: Res<ButtonInput<KeyCode>>,
    ui_tracker: Res<UITracker>,
) {
    if ui_tracker.chosen_text_position.is_some() { return }
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
            let RenderTarget::Window(window) = camera.target else { continue; };
            let WindowRef::Entity(entity) = window else { continue; };
            if focus.0 != Some(entity) { continue; }
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
            let RenderTarget::Window(window) = camera.target else { continue; };
            let WindowRef::Entity(entity) = window else { continue; };
            if focus.0 != Some(entity) { continue; }
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
            close_when_requested: false,
            exit_condition: ExitCondition::DontExit,
        }))
        .add_plugins(AdminUI)
        .insert_resource(HexLayoutResource(HexLayout {
            hex_size: Vec2::splat(105. * 3f32.sqrt()),
            orientation: HexOrientation::Pointy,
            ..default()
        }))
        .init_resource::<Scale>()
        .init_resource::<MouseLastPosition>()
        .init_resource::<Textures>()
        .init_resource::<FoldersLoading>()
        .init_resource::<Focus>()
        .init_resource::<UITracker>()
        .init_resource::<Map>()
        .init_state::<AppStates>()
        .add_systems(OnEnter(AppStates::Loading), load)
        .add_systems(Update, check_loading.run_if(in_state(AppStates::Loading)))
        .add_systems(OnEnter(AppStates::Loaded), (
            setup_ui,
        ))
        .add_systems(FixedUpdate, (
            move_camera,
            zoom,
            (detect_press, map_drag).chain(),
        ).run_if(in_state(AppStates::Loaded)))
        .add_systems(Update, (
            exit_on_esc,
            focus_checker,
            mouse_scroll,
        ).run_if(in_state(AppStates::Loaded)))
        .run();
}
