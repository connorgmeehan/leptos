//!

use std::{cell::RefCell, ops::DerefMut};

use bevy::ecs::world::World;
use leptos::tachys::renderer::Renderer;

use super::node::{BevyNode, LeptosNodeId, LeptosNodeMap, BEVY_NODES};

thread_local! {
    static BEVY_WORLD: RefCell<Option<&'static mut World>> = RefCell::new(None);
}

pub fn extend_lifetime<'a, T>(r: &'a mut T) -> &'static mut T {
    unsafe {
        // Cast the reference to a 'static reference
        &mut *(r as *mut T)
    }
}
pub(crate) fn set_bevy_world_ref(world: &mut World) {
    let static_world = extend_lifetime(world);
    BEVY_WORLD.with(|v: &RefCell<Option<&mut World>>| {
        let mut v = v.borrow_mut();
        *v = Some(static_world);
    })
}
pub(crate) fn unset_bevy_world_ref() {
    BEVY_WORLD.with(|v: &RefCell<Option<&mut World>>| {
        let mut v = v.borrow_mut();
        *v = None;
    })
}

pub(crate) fn with_world<U, F>(with_fn: F) -> U
where
    F: FnOnce(&mut World) -> U,
{
    BEVY_WORLD.with(|v| {
        let mut v = v.borrow_mut();
        let v = v.as_deref_mut();
        let result = v.map(with_fn);
        return result.unwrap();
    })
}

pub(crate) fn with_nodes<U, F>(with_fn: F) -> U
where
    F: FnOnce(&mut LeptosNodeMap) -> U,
{
    BEVY_NODES.with(|v| {
        let mut v = v.borrow_mut();
        let node_map = v.deref_mut();
        let result = (with_fn)(node_map);
        return result;
    })
}

pub(crate) fn with_world_and_nodes<U, F>(with_fn: F) -> U
where
    F: FnOnce(&mut World, &mut LeptosNodeMap) -> U,
{
    with_world(|world| with_nodes(|nodes| (with_fn)(world, nodes)))
}

#[derive(Debug)]
pub struct BevyRenderer;

impl Renderer for BevyRenderer {
    type Node = LeptosNodeId;
    type Element = LeptosNodeId;
    type Text = LeptosNodeId;
    type Placeholder = LeptosNodeId;

    fn intern(text: &str) -> &str {
        text
    }

    fn create_text_node(_text: &str) -> Self::Text {
        todo!()
    }

    fn create_placeholder() -> Self::Placeholder {
        with_world_and_nodes(|world, node_map| {
            BevyNode::spawn_new(world, node_map).node_id()
        })
    }

    fn set_text(_node: &Self::Text, _text: &str) {
        todo!()
    }

    fn set_attribute(_node: &Self::Element, _name: &str, _value: &str) {
        // node.0.set_property(name, value);
        todo!()
    }

    fn remove_attribute(_node: &Self::Element, _name: &str) {
        // node.0.set_property(name, None::<&str>);
        todo!()
    }

    fn insert_node(
        parent: &Self::Element,
        new_child: &Self::Node,
        marker: Option<&Self::Node>,
    ) {
        with_world_and_nodes(|world, node_map| match marker {
            Some(marker) => {
                let [parent, child, marker] =
                        node_map.get_many_mut([parent, new_child, marker]).expect(
                            "Renderer::insert_node() Couldn't get new_child, parent and/or marker.",
                        );
                child.attach_to_parent_before_marker(
                    world,
                    parent.entity,
                    marker.entity,
                );
            }
            None => {
                let [parent, child] =
                        node_map.get_many_mut([parent, new_child]).expect(
                            "Renderer::insert_node() Couldn't get new_child and/or parent.",
                        );
                child.attach_to_parent(world, parent.node_id(), parent.entity);
            }
        });
    }

    fn remove_node(
        _parent: &Self::Element,
        child: &Self::Node,
    ) -> Option<Self::Node> {
        with_world_and_nodes(|world, node_map| {
            let child =
                node_map.get_mut(child).expect("Renderer::remove_node()");
            child.detatch_from_parent(world);
            Some(child.node_id())
        })
    }

    fn remove(_node: &Self::Node) {
        todo!()
    }

    fn get_parent(_node: &Self::Node) -> Option<Self::Node> {
        todo!()
        // with_world(|world| node.bevy_get_parent(world).map(::new))
    }

    fn first_child(_node: &Self::Node) -> Option<Self::Node> {
        todo!()
        // with_world(|world| node.bevy_first_child(world)).map(BElement::new)
    }

    fn next_sibling(_node: &Self::Node) -> Option<Self::Node> {
        todo!()
    }

    fn log_node(_node: &Self::Node) {
        todo!()
    }

    fn clear_children(_parent: &Self::Node) {
        todo!()
        // with_world(|world| {
        //     world.entity_mut(_parent.entity).despawn_descendants();
        // });
    }
}
