use any_spawner::Executor;
use bevy::{
    app::Plugin,
    core::Name,
    ecs::{component::Component, entity::Entity, world::World},
    utils::HashMap,
};
use leptos::{
    context::provide_context, reactive_graph::owner::Owner, tachys::view::{Mountable, Render}
};

use super::{
    core::{
        renderer::{
            set_bevy_world_ref, unset_bevy_world_ref, with_nodes_mut, BevyRenderer,
        },
        view::IntoBevyView, BevyLeptosContext,
    },
    entity,
};

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
    roots: HashMap<Entity, BevyLeptosData>,
}

// TODO: Unmounting leptos roots.
#[allow(dead_code)]
struct BevyLeptosData {
    owner: Owner,
    // mountable: Box<dyn Mountable<LeptosBevy> +'static>,
    entity: Entity,
}

pub fn leptos_root(
    app_fn: impl IntoBevyView,
) -> (Entity, impl Mountable<BevyRenderer>) {
    let ctx = BevyLeptosContext::new();
    provide_context(ctx);

    let view = app_fn.into_view();
    let state = entity().children(view).build();

    let entity = with_nodes_mut(|node_map| {
        *node_map
            .get(&state.node)
            .expect("root(). Could not get the node (that I just created).")
            .entity()
    });
    (entity, state)
}

pub trait LeptosWorldExt {
    fn spawn_leptos(&mut self, app_fn: impl IntoBevyView) -> Entity;
}

impl LeptosWorldExt for World {
    fn spawn_leptos(&mut self, app_fn: impl IntoBevyView) -> Entity {
        set_bevy_world_ref(self);
        let owner = Owner::new();
        let (entity, _mountable) = owner.with(|| leptos_root(app_fn));
        let mut res = self.non_send_resource_mut::<LeptosResource>();
        res.roots.insert(
            entity,
            BevyLeptosData {
                owner,
                // mountable: Box::new(mountable),
                entity,
            },
        );
        let mut entity_mut = self.entity_mut(entity);
        entity_mut.insert((HasLeptosRoot, Name::from("LeptosRoot")));
        unset_bevy_world_ref();
        entity_mut.id()
    }
}

#[derive(Component)]
pub struct HasLeptosRoot;

// impl Component for HasLeptosRoot {
//     const STORAGE_TYPE: StorageType = StorageType::Table;
//
//     fn register_component_hooks(hooks: &mut ComponentHooks) {
//         hooks.on_remove(|mut world, entity, _component_id| {
//             let v: &mut World = &mut world;
//             set_bevy_world_ref(v);
//             let mut res = world.non_send_resource_mut::<LeptosResource>();
//         });
//     }
// }
