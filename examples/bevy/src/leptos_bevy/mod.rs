use std::marker::PhantomData;

use bevy::ecs::entity::Entity;
use leptos::tachys::view::{Mountable, Render};

use self::core::{elements::BevyElement, renderer::{with_nodes, BevyRenderer}};

pub mod core;
pub mod plugin;

/// Spawns an entity and parents it to the parent element.
pub fn entity() -> BevyElement<Entity, (), ()> {
    BevyElement {
        ty: PhantomData,
        properties: (),
        children: (),
    }
}

pub fn leptos_root<TChildren>(
    children: TChildren,
) -> (Entity, impl Mountable<BevyRenderer>)
where
    TChildren: Render<BevyRenderer>,
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
