use std::marker::PhantomData;

use bevy::ecs::entity::Entity;
use leptos::{children::TypedChildren, tachys::view::{Mountable, Render}};

use self::core::{elements::BevyElement, renderer::{with_nodes_mut, BevyRenderer}, view::IntoBevyView};

pub mod core;
pub mod plugin;
pub mod hooks;

/// Spawns an entity and parents it to the parent element.
pub fn entity() -> BevyElement<Entity, (), ()> {
    BevyElement {
        ty: PhantomData,
        properties: (),
        children: (),
    }
}

