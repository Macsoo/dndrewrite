use bevy::asset::{Handle, LoadedFolder};
use bevy::prelude::{Deref, DerefMut, Image, Resource};
use bevy::utils::HashMap;

use super::*;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct FoldersLoading(pub(super) Vec<(String, Handle<LoadedFolder>)>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct LoadHandlers(pub(super) Vec<load_handler::LoadHandler>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct TexturesMap(pub(super) HashMap<id::Id, Handle<Image>>);

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct TextureTreeResource(pub(crate) texture_tree::TextureNode);