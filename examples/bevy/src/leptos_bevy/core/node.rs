use std::cell::RefCell;

use bevy::{
    ecs::{component::Component, entity::Entity, world::World},
    hierarchy::{BuildWorldChildren, Children, DespawnRecursiveExt, Parent},
    prelude::Deref,
    utils::HashMap,
};
use leptos::tachys::{
    renderer::{CastFrom, Renderer},
    view::Mountable,
};

use super::renderer::{with_nodes_mut, with_world_and_nodes, BevyRenderer};

/// Nodes in leptos are a simple usize ID, this map relates them to entities within
/// the bevy world.
pub type LeptosNodeMap = HashMap<LeptosNodeId, BevyNode>;

thread_local! {
    pub static BEVY_NODES: RefCell<LeptosNodeMap> = RefCell::new(LeptosNodeMap::new());
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
    pub(crate) node_id: LeptosNodeId,
    pub(crate) entity: Entity,
    pub(crate) parent: Option<LeptosNodeId>,
    pub(crate) parent_entity: Option<Entity>,
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

impl Mountable<BevyRenderer> for LeptosNodeId {
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
        parent: &<BevyRenderer as Renderer>::Element,
        marker: Option<&<BevyRenderer as Renderer>::Node>,
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
        child: &mut dyn Mountable<BevyRenderer>,
    ) -> bool {
        let parent_id = with_nodes_mut(|node_map| {
            node_map.get(self).expect("Mountable::insert_before_this(). Tried to get BevyNode for LeptosNodeId {self:?} but it doesn't exist.").parent
        });
        let Some(parent_id) = parent_id else {
            return false;
        };
        child.mount(&parent_id, Some(self));
        true
    }
}
