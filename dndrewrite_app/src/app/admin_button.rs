use bevy::prelude::*;
use crate::model::id::Id;

pub struct AdminButton {
    pub texture: Handle<Image>,
    pub name: String,
    pub on_click: Box<dyn Fn(Id) -> Id + Send + Sync>,
    pub on_hover: Box<dyn Fn(Mut<Window>) -> () + Send + Sync>,
}