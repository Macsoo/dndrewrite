use bevy::asset::{AssetPath, AssetServer, Handle, LoadedFolder, RecursiveDependencyLoadState};
use bevy::log::error;
use bevy::prelude::*;
use bevy::utils::HashMap;
use regex::Regex;
use crate::app::resources::AppLoaded;
use crate::model::texture_tree::TextureNode;
use crate::model::id::Id;
use super::*;

pub fn load(
    asset_server: Res<AssetServer>,
    mut folders_loading: ResMut<resources::FoldersLoading>,
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

pub fn register_load_handlers(
    mut commands: Commands
) {
    let tile_regex = Regex::new(r"(?<number>\d+)\.\w*$").unwrap();
    let regex_clone = tile_regex.clone();
    let mut load_handlers = Vec::new();
    load_handlers.push(load_handler::LoadHandler::new(
        r"^pointy\.(?<tile_type>\w+)_(?<tile_variant>[A-Za-z0-9]*)\.(?<color>\w*)$",
        move |captures, handles| {
            let tile_type = captures.get("tile_type").unwrap();
            let tile_variant = captures.get("tile_variant").unwrap();
            let color = captures.get("color").unwrap();
            handles.into_iter()
                .map(|(file, handle)| {
                    let captures = regex_clone.captures(&file).unwrap();
                    let number_str = captures.name("number").unwrap().as_str();
                    (id::id![tile_type, tile_variant, color, number_str], handle)
                })
                .collect()
        },
    ).unwrap());
    load_handlers.push(load_handler::LoadHandler::new(
        r"^overlay_(?<name>\w+)\.standard_full$",
        move |captures, handles| {
            let name = captures.get("name").unwrap();
            handles.into_iter()
                .map(|(file, handle)| {
                    let captures = tile_regex.captures(&file).unwrap();
                    let number_str = captures.name("number").unwrap().as_str();
                    (id::id!["overlay", name, number_str], handle)
                })
                .collect()
        },
    ).unwrap());
    commands.insert_resource(resources::LoadHandlers(load_handlers));
}

pub fn check_load_handlers(
    load_handlers: Res<resources::LoadHandlers>,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<resources::TexturesMap>,
    mut folders_loading: ResMut<resources::FoldersLoading>,
    mut loaded_folders: ResMut<Assets<LoadedFolder>>,
    mut commands: Commands,
) {
    for (name, handle) in std::mem::take(&mut folders_loading.0) {
        let load_state = asset_server.recursive_dependency_load_state(handle.id());
        if !matches!(load_state, RecursiveDependencyLoadState::Loaded) {
            folders_loading.0.push((name, handle));
            continue;
        }
        let loaded_folder = loaded_folders.remove(handle.id()).unwrap();
        for load_handler in &load_handlers.0 {
            let Some(captures) = load_handler.pattern().captures(&name) else { continue; };
            //Found our handler
            let mut capture_map = HashMap::new();
            for capture_name in load_handler.pattern().capture_names().skip(1) {
                let Some(name) = capture_name else { continue; };
                let Some(value) = captures.name(name) else { continue };
                capture_map.insert(name.to_owned(), value.as_str().to_owned());
            }
            let handles: HashMap<String, Handle<Image>> = loaded_folder.handles
                .into_iter()
                .map(UntypedHandle::typed::<Image>)
                .map(|handle| {
                    let Some(path) = handle.path().map(AssetPath::path) else { return Err(handle.clone()) };
                    let Some(file_name) = path.file_name() else { return Err(handle.clone()) };
                    let Some(file_name_str) = file_name.to_str() else { return Err(handle.clone()) };
                    Ok((file_name_str.to_owned(), handle))
                })
                .filter_map(Result::ok)
                .collect();
            //Handle loading
            for (id, handle) in load_handler.handle(capture_map, handles) {
                textures.0.insert(id, handle);
            }
            break;
        }
        info!("Loaded folder '{}'.", name);
    }
    if folders_loading.0.is_empty() {
        commands.remove_resource::<resources::FoldersLoading>();
        info!("Assets loaded!");
    }
}

pub fn transform_textures(
    mut commands: Commands,
    mut textures_map: ResMut<resources::TexturesMap>,
) {
    let map = std::mem::take(&mut textures_map.0);
    commands.remove_resource::<resources::TexturesMap>();
    let tree = TextureNode::new(map);
    commands.insert_resource(resources::TextureTreeResource(tree));
    commands.insert_resource(AppLoaded);
}