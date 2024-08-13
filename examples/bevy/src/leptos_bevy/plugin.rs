use any_spawner::Executor;
use bevy::{app::Plugin, ecs::entity::Entity, utils::HashMap};

use super::world_ext::BevyLeptosData;

pub struct LeptosPlugin;

impl Plugin for LeptosPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        _ = Executor::init_wasm_bindgen();

        app.insert_non_send_resource(LeptosResource::default());
        // app.add_systems(PostUpdate, leptos_update_system);
    }
}

#[derive(Default)]
pub struct LeptosResource {
    pub(crate) roots: HashMap<Entity, BevyLeptosData>,
}
