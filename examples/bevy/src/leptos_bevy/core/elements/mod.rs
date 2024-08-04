use std::marker::PhantomData;

use bevy::ecs::entity::Entity;
use leptos::tachys::{renderer::Renderer, view::{Mountable, Render}};
use next_tuple::NextTuple;
use super::{node::{BevyNode, LeptosNodeId}, properties::Property, renderer::{with_world_and_nodes, BevyRenderer}};

/// Represents an element without data or state.  Usually for when an element isn't mounted yet.
///
/// * `widget`:
/// * `properties`:
/// * `children`:
pub struct BevyElement<TElement, TProps, TChildren> {
    pub(crate) ty: PhantomData<TElement>,
    pub(crate) properties: TProps,
    pub(crate) children: TChildren,
}

impl<TElement, TProps, TChildren> BevyElement<TElement, TProps, TChildren>
where
    TChildren: NextTuple,
{
    pub fn children<T>(
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

impl<TProps, TChildren> Render<BevyRenderer>
    for BevyElement<Entity, TProps, TChildren>
where
    TProps: Property,
    TChildren: Render<BevyRenderer>,
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
    TChildren: Render<BevyRenderer>,
    TProps: Property,
{
    pub(crate) ty: PhantomData<TElement>,
    pub(crate) node: LeptosNodeId,
    pub(crate) properties: TProps::State,
    pub(crate) children: TChildren::State,
}

impl<TElement, TProps, Children> Mountable<BevyRenderer>
    for BevyElementState<TElement, TProps, Children>
where
    TProps: Property,
    Children: Render<BevyRenderer>,
{
    fn unmount(&mut self) {
        self.children.unmount();
        self.node.unmount();
    }

    fn mount(
        &mut self,
        parent: &<BevyRenderer as Renderer>::Element,
        marker: Option<&<BevyRenderer as Renderer>::Node>,
    ) {
        println!("mounting {}", std::any::type_name::<TElement>());
        self.children.mount(&self.node, None);
        self.node.mount(parent, marker);
    }

    fn insert_before_this(
        &self,
        child: &mut dyn Mountable<BevyRenderer>,
    ) -> bool {
        self.node.insert_before_this(child)
    }
}
