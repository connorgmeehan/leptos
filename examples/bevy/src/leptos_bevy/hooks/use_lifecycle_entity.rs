use bevy::ecs::{entity::Entity, world::World};
use leptos::reactive_graph::owner::on_cleanup;

use crate::leptos_bevy::core::renderer::with_world_mut;

/// Spawns an entity when this component is mounted, despawns it when unmounted.
pub fn create_lifecycle_entity(
    on_mount: impl FnOnce(&mut World) -> Entity,
) -> Entity {
    let entity = with_world_mut(|world| (on_mount)(world));
    on_cleanup(move || {
        with_world_mut(|world| world.despawn(entity));
    });
    entity
}
