use any_spawner::Executor;
use bevy::{
    app::{Plugin, PostUpdate}, ecs::{entity::Entity, schedule::IntoSystemConfigs, world::World}, log::warn, transform::TransformSystem, utils::HashMap
};

use super::{
    core::{executor::BevyLeptosExecutor, renderer::{set_bevy_world_ref, unset_bevy_world_ref}, BevyLeptosState},
    world_ext::BevyLeptosData,
};

pub struct LeptosPlugin;

impl Plugin for LeptosPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        _ = Executor::init_any(
            BevyLeptosExecutor::spawn,
            BevyLeptosExecutor::spawn_local,
        );

        app.insert_non_send_resource(LeptosResource::default());
        app.add_systems(
            PostUpdate,
            update_leptos.before(TransformSystem::TransformPropagate),
        );
    }
}

#[derive(Default)]
pub struct LeptosResource {
    pub(crate) roots: HashMap<Entity, BevyLeptosData>,
}

pub fn update_leptos(world: &mut World) {
    warn!("update_leptos");
    BevyLeptosState::sys_notify_tracked_resources(world);

    set_bevy_world_ref(world);
    BevyLeptosExecutor::flush();
    unset_bevy_world_ref()
}
