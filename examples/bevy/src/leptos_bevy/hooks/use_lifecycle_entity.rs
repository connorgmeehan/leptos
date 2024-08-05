use bevy::ecs::{entity::Entity, system::Resource, world::World};
use leptos::reactive_graph::{effect::create_effect, owner::on_cleanup, signal::{create_signal, ReadSignal}, traits::Get};

use crate::leptos_bevy::core::{renderer::with_world_mut, signals::BevyReadSignal};

pub struct ResourceTracker<T: Resource> {
    pd: T,
    signal: BevyReadSignal<T>,
}

/// Spawns an entity when this component is mounted, despawns it when unmounted.
pub fn create_lifecycle_entity(on_mount: impl FnOnce(&mut World) -> Entity) -> Entity {
    let entity = with_world_mut(|world| (on_mount)(world));
    on_cleanup(move|| {
        with_world_mut(|world| world.despawn(entity));
    });
    entity
}

pub fn use_resource<R: Resource>() {
    let (get, set) = RwSign::new(1);
    get.get()
    create_lifecycle_entity(|world| world.spawn(TrackedResource<R> { }));
}
