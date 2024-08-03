//!

use std::{cell::RefCell, ops::DerefMut};

use bevy::{
    ecs::{component::Component, entity::Entity, world::World},
    hierarchy::{BuildWorldChildren, Children, DespawnRecursiveExt, Parent},
    prelude::Deref,
    utils::hashbrown::HashMap,
};
use leptos::tachys::{
    renderer::{CastFrom, Renderer},
    view::{Mountable, Render},
};

use super::{properties::Property, BevyElementState};

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

/// Nodes in leptos are a simple usize ID, this map relates them to entities within
/// the bevy world.
pub type LeptosNodeMap = HashMap<LeptosNodeId, BevyNode>;

thread_local! {
    static BEVY_NODES: RefCell<LeptosNodeMap> = RefCell::new(LeptosNodeMap::new());
    static NODE_ID_COUNTER: RefCell<usize> = RefCell::new(0);
}
pub fn get_next_node_id() -> LeptosNodeId {
    NODE_ID_COUNTER.with(|id| {
        let mut node_id = id.borrow_mut();
        *node_id += 1;
        LeptosNodeId(*node_id)
    })
}

#[derive(Debug, Component, Deref, Copy, Clone, Eq, Hash, PartialEq)]
pub struct LeptosNodeId(usize);
impl<'a> LeptosNodeId {
    pub fn node(&'a self, node_map: &'a mut LeptosNodeMap) -> &'a mut BevyNode {
        node_map.get_mut(self).expect("LeptosNodeId::node() Tried to get self {self:?} but no node in map.")
    }
}

/// Represents a real entity within the world.
///
/// * `entity`:
/// * `parent`:
#[derive(Debug, Clone, Copy)]
pub struct BevyNode {
    node_id: LeptosNodeId,
    entity: Entity,
    parent: Option<LeptosNodeId>,
    parent_entity: Option<Entity>,
}
impl BevyNode {
    pub fn entity(&self) -> &Entity {
        &self.entity
    }
    pub fn node_id(&self) -> LeptosNodeId {
        self.node_id
    }

    pub fn spawn_new(world: &mut World, node_map: &mut LeptosNodeMap) -> Self {
        let node_id = get_next_node_id();
        let entity = world.spawn(node_id.clone()).id();
        let instance = Self {
            node_id,
            entity,
            parent: None,
            parent_entity: None,
        };
        node_map.insert(node_id, instance);
        instance
    }

    // This should get cleaned up immediately by new bevy hooks
    // (TODO: Implement bevy hook to cleanup LeptosNodeMap)
    pub fn despawn(&mut self, world: &mut World) {
        world.entity_mut(self.entity).despawn_descendants();
    }

    pub fn from_existing(world: &mut World, entity: Entity) -> Self {
        let entity = world.entity(entity);
        let node_id = *entity.get::<LeptosNodeId>().expect("BevyNode::from_existing().  Tried to get node id of {entity:?} but it doesn't exist.");
        let parent_entity = entity.get::<Parent>().map(|p| p.get());
        let parent = parent_entity.and_then(|parent_entity| {
            world.get::<LeptosNodeId>(parent_entity).copied()
        });
        Self {
            node_id,
            entity: entity.id(),
            parent,
            parent_entity,
        }
    }

    pub fn detatch_from_parent(&mut self, world: &mut World) {
        if self.parent.is_some() {
            world.entity_mut(self.entity).remove_parent();
            self.parent = None;
            self.parent_entity = None;
        }
    }

    pub fn attach_to_parent(
        &mut self,
        world: &mut World,
        parent: LeptosNodeId,
        parent_entity: Entity,
    ) {
        world.entity_mut(self.entity).set_parent(parent_entity);
        self.parent = Some(parent);
        self.parent_entity = Some(parent_entity)
    }

    pub fn attach_to_parent_before_marker(
        &mut self,
        world: &mut World,
        parent: Entity,
        marker: Entity,
    ) -> bool {
        let mut parent_mut = world.entity_mut(parent);
        let position = parent_mut
            .get::<Children>()
            .and_then(|c| c.iter().position(|e| *e == marker));
        let Some(position) = position else {
            return false;
        };
        parent_mut.insert_children(position, &[self.entity]);
        true
    }
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

impl AsRef<LeptosNodeId> for LeptosNodeId {
    fn as_ref(&self) -> &LeptosNodeId {
        self
    }
}

impl CastFrom<LeptosNodeId> for LeptosNodeId {
    fn cast_from(source: LeptosNodeId) -> Option<Self> {
        Some(source)
    }
}

#[derive(Debug)]
pub struct LeptosBevy;

impl Mountable<LeptosBevy> for LeptosNodeId {
    fn unmount(&mut self) {
        with_world_and_nodes(|world, node_map| {
            let node = node_map
                .get_mut(self)
                .expect("Mountable::unmount() Couldn't get node for {self}.");
            node.detatch_from_parent(world);
        })
    }

    fn mount(
        &mut self,
        parent: &<LeptosBevy as Renderer>::Element,
        marker: Option<&<LeptosBevy as Renderer>::Node>,
    ) {
        with_world_and_nodes(|world, node_map| match marker {
            Some(marker) => {
                let [parent, child, marker] =
                        node_map.get_many_mut([parent, self, marker]).expect(
                            "Mountable::insert_node() Couldn't get new_child, parent and/or marker.",
                        );
                child.attach_to_parent_before_marker(
                    world,
                    parent.entity,
                    marker.entity,
                );
            }
            None => {
                let [parent, child] =
                        node_map.get_many_mut([parent, self]).expect(
                            "Mountable::insert_node() Couldn't get new_child and/or parent.",
                        );
                child.attach_to_parent(world, parent.node_id(), parent.entity);
            }
        });
    }

    fn insert_before_this(
        &self,
        child: &mut dyn Mountable<LeptosBevy>,
    ) -> bool {
        let parent_id = with_nodes(|node_map| {
            node_map.get(self).expect("Mountable::insert_before_this(). Tried to get BevyNode for LeptosNodeId {self:?} but it doesn't exist.").parent
        });
        let Some(parent_id) = parent_id else {
            return false;
        };
        child.mount(&parent_id, Some(self));
        true
    }
}

impl<TElement, TProps, Children> Mountable<LeptosBevy>
    for BevyElementState<TElement, TProps, Children>
where
    TProps: Property,
    Children: Render<LeptosBevy>,
{
    fn unmount(&mut self) {
        self.children.unmount();
        self.node.unmount();
    }

    fn mount(
        &mut self,
        parent: &<LeptosBevy as Renderer>::Element,
        marker: Option<&<LeptosBevy as Renderer>::Node>,
    ) {
        println!("mounting {}", std::any::type_name::<TElement>());
        self.children.mount(&self.node, None);
        self.node.mount(parent, marker);
    }

    fn insert_before_this(
        &self,
        child: &mut dyn Mountable<LeptosBevy>,
    ) -> bool {
        self.node.insert_before_this(child)
    }
}

impl Renderer for LeptosBevy {
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
