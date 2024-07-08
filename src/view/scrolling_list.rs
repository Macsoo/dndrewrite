use bevy::hierarchy::Parent;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct ScrollingList {
    position: f32,
}

//Modified copy from https://bevyengine.org/examples/UI%20(User%20Interface)/ui/
pub fn mouse_scroll(
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
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };
            scrolling_list.position += dx;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, max_scroll);
            style.left = Val::Px(scrolling_list.position);
        }
    }
}