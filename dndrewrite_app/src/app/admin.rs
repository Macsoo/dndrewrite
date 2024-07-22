use crate::app::admin_button::AdminButton;
use std::sync::Arc;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::utils::HashMap;
use crate::app::resources::AdminButtonMarker;
use crate::model::id::{Id, id};
use crate::model::resources::TextureTreeResource;
use crate::model::texture_tree::TextureNode;
use crate::view::query::UIQuery;
use super::*;

impl FromWorld for resources::AdminMenus {
    fn from_world(world: &mut World) -> Self {
        fn recursive_helper(
            texture_node: TextureNode,
            admin_map: &mut HashMap<Id, Arc<admin_menu::AdminMenu>>,
            id: Id,
        ) -> Option<Handle<Image>>
        {
            let branch;
            match texture_node.0 {
                Ok(map) => branch = map,
                Err(handle) => return Some(handle)
            }
            let mut buttons = Vec::new();
            for (name, node) in branch {
                let is_leaf = node.is_err();
                if let Some(handle) = recursive_helper(node, admin_map, id.extend(name.clone())) {
                    buttons.push(Arc::new(AdminButton {
                        texture: handle,
                        name: name.clone(),
                        on_click: if !is_leaf {
                            Box::new(move |id: Id| id.extend(name.clone()))
                        } else {
                            Box::new(|id: Id| id)
                        },
                        on_hover: Box::new(|mut window| window.cursor.icon = CursorIcon::Pointer),
                    }));
                }
            }
            let first_texture = buttons.first()
                .map(|x| x.texture.clone());
            admin_map.insert(id, admin_menu::AdminMenu(buttons).into());
            first_texture
        }
        let texture_tree: &TextureTreeResource = world.get_resource().unwrap();
        let mut admin_map = HashMap::new();
        recursive_helper(texture_tree.0.clone(), &mut admin_map, Id::new(&[]));
        Self(admin_map)
    }
}

impl FromWorld for resources::CurrentAdminMenu {
    fn from_world(world: &mut World) -> Self {
        let menus: &resources::AdminMenus = world.get_resource().unwrap();
        let root = menus.get(&id!()).unwrap().clone();
        root.render(world);
        Self(root)
    }
}

pub fn handle_admin(
    interaction_query: Query<(
        Entity,
        Ref<Interaction>,
        Ref<AdminButtonMarker>,
        Ref<RelativeCursorPosition>,
    )>,
    mut admin_stack: ResMut<resources::AdminMenuStack>,
    mut ui: UIQuery,
    admin_menus: Res<resources::AdminMenus>,
    mut current_admin: ResMut<resources::CurrentAdminMenu>,
    mut commands: Commands,
) {
    let Some((_, mut window)) = ui.get_focused_window_mut() else { return; };
    let mut is_over_any = false;
    for (_, interaction, button, relative) in &interaction_query {
        if relative.mouse_over() {
            is_over_any = true;
        }
        if !interaction.is_changed() { continue; }
        match *interaction {
            Interaction::Pressed => {
                let id = (button.on_click)(admin_stack.0.clone());
                if admin_stack.0 != id {
                    let menu = admin_menus.get(&id).unwrap().clone();
                    admin_stack.0 = id;
                    let am = menu.clone();
                    commands.add(move |w: &mut World| menu.render(w));
                    **current_admin = am;
                }
            }
            Interaction::Hovered => (button.on_hover)(window.reborrow()),
            _ => {}
        }
    }
    if !is_over_any {
        window.cursor.icon = CursorIcon::Default;
    }
}