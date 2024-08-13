use bevy::{
    core::Name,
    ecs::{component::Component, entity::Entity, world::World},
};
use leptos::{
    context::provide_context,
    reactive_graph::owner::Owner,
    tachys::view::{Mountable, Render},
};

use super::{core::{
    elements::entity,
    renderer::{
        set_bevy_world_ref, unset_bevy_world_ref, with_nodes_mut, BevyRenderer,
    },
    view::IntoBevyView,
    BevyLeptosContext,
}, plugin::LeptosResource};

// TODO: Unmounting leptos roots.
#[allow(dead_code)]
pub struct BevyLeptosData {
    pub(crate) owner: Owner,
    // mountable: Box<dyn Mountable<LeptosBevy> +'static>,
    pub(crate) entity: Entity,

    pub(crate) context: BevyLeptosContext,
}

pub fn leptos_root(
    app_fn: impl IntoBevyView,
) -> (Entity, impl Mountable<BevyRenderer>, BevyLeptosContext) {
    let ctx = BevyLeptosContext::default();
    provide_context(ctx.clone());

    let view = app_fn.into_view();
    let state = entity().children(view).build();

    let entity = with_nodes_mut(|node_map| {
        *node_map
            .get(&state.node)
            .expect("root(). Could not get the node (that I just created).")
            .entity()
    });
    (entity, state, ctx)
}

pub trait LeptosWorldExt {
    fn spawn_leptos(&mut self, app_fn: impl IntoBevyView) -> Entity;
}

impl LeptosWorldExt for World {
    fn spawn_leptos(&mut self, app_fn: impl IntoBevyView) -> Entity {
        set_bevy_world_ref(self);
        let owner = Owner::new();
        let (entity, _mountable, context) = owner.with(|| leptos_root(app_fn));
        let mut res = self.non_send_resource_mut::<LeptosResource>();
        res.roots.insert(
            entity,
            BevyLeptosData {
                owner,
                // mountable: Box::new(mountable),
                entity,
                context,
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
