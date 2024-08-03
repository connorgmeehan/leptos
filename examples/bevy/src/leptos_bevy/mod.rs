use std::marker::PhantomData;

use self::{
    properties::Property,
    renderer::{
        with_nodes, with_world_and_nodes, BevyNode, LeptosBevy, LeptosNodeId,
    },
};

use bevy::ecs::entity::Entity;
use leptos::prelude::*;
use next_tuple::NextTuple;

pub mod plugin;
pub mod properties;
pub mod renderer;

/// Represents an element without data or state.  Usually for when an element isn't mounted yet.
///
/// * `widget`:
/// * `properties`:
/// * `children`:
pub struct BevyElement<TElement, TProps, TChildren> {
    ty: PhantomData<TElement>,
    properties: TProps,
    children: TChildren,
}

impl<TElement, TProps, TChildren> BevyElement<TElement, TProps, TChildren>
where
    TChildren: NextTuple,
{
    pub fn child<T>(
        self,
        child: T,
    ) -> BevyElement<TElement, TProps, TChildren::Output<T>> {
        let BevyElement {
            ty,
            properties,
            children,
        } = self;
        BevyElement {
            ty,
            properties,
            children: children.next_tuple(child),
        }
    }
}

impl<TProps, TChildren> Render<LeptosBevy>
    for BevyElement<Entity, TProps, TChildren>
where
    TProps: Property,
    TChildren: Render<LeptosBevy>,
{
    type State = BevyElementState<Entity, TProps, TChildren>;

    fn build(self) -> Self::State {
        let node = with_world_and_nodes(|world, node_map| {
            BevyNode::spawn_new(world, node_map).node_id()
        });
        let properties = self.properties.build(&node);
        let mut children = self.children.build();

        children.mount(&node, None);
        BevyElementState {
            ty: PhantomData,
            node,
            properties,
            children,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        self.properties.rebuild(&state.node, &mut state.properties);
        self.children.rebuild(&mut state.children);
    }
}

///
///
/// * `ty`:
/// * `widget`:
/// * `properties`:
/// * `children`:
pub struct BevyElementState<TElement, TProps, TChildren>
where
    TChildren: Render<LeptosBevy>,
    TProps: Property,
{
    ty: PhantomData<TElement>,
    node: LeptosNodeId,
    properties: TProps::State,
    children: TChildren::State,
}

/// Spawns an entity and parents it to the parent element.
pub fn entity() -> BevyElement<Entity, (), ()> {
    BevyElement {
        ty: PhantomData,
        properties: (),
        children: (),
    }
}

pub fn root<TChildren>(
    children: TChildren,
) -> (Entity, impl Mountable<LeptosBevy>)
where
    TChildren: Render<LeptosBevy>,
{
    let state = entity().child(children).build();
    let entity = with_nodes(|node_map| {
        *node_map
            .get(&state.node)
            .expect("root(). Could not get the node (that I just created).")
            .entity()
    });
    (entity, state)
}
