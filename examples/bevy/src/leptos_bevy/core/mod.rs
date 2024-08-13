use bevy::{
    ecs::{component::ComponentId, system::Resource, world::World},
    prelude::Deref,
    utils::{HashMap, HashSet},
};
use leptos::{
    context::use_context,
    reactive_graph::{
        computed::{create_read_slice, create_slice, create_write_slice},
        signal::RwSignal,
        traits::{ReadUntracked, Trigger},
        wrappers::{read::Signal, write::SignalSetter},
    },
};

use self::{renderer::with_world_ref, signals::BevyReadSignalTrigger};

use super::plugin::LeptosResource;

pub mod children;
pub mod elements;
pub mod node;
pub mod properties;
pub mod provider;
pub mod renderer;
pub mod signals;
pub mod view;

#[derive(Clone, Default)]
pub struct BevyLeptosState {
    pub resource_trackers:
        HashMap<ComponentId, HashSet<BevyReadSignalTrigger>>,
}

#[allow(dead_code)]
impl BevyLeptosState {
    pub fn sys_notify_tracked_resources(world: &mut World) {
        let tick = world.change_tick();
        let last_tick = world.last_change_tick();
        let v = world.non_send_resource::<LeptosResource>();
        for root in v.roots.values() {
            let state = root.context.0;

            let untracked_state = state.read_untracked();
            let changed_resources = untracked_state
                .resource_trackers
                .iter()
                .filter(|(resource_id, _)| {
                    world
                        .get_resource_change_ticks_by_id(**resource_id)
                        .map(|ticks| ticks.is_changed(last_tick, tick))
                        .unwrap_or(false)
                });

            for (_, listeners) in changed_resources {
                for listener in listeners {
                    listener.trigger();
                }
            }
        }
    }

    pub fn track_resource<R: Resource>(
        &mut self,
        notifier: BevyReadSignalTrigger,
    ) {
        let resource_id = with_world_ref(|world| world.components().resource_id::<R>()).expect("BevyLeptosState::track_resource() Could not track resource as it hasn't been initialised within the world yet.");
        let entry = self.resource_trackers.entry(resource_id);
        let notifiers = entry.or_default();
        notifiers.insert(notifier);
    }
    pub fn untrack_resource<R: Resource>(
        &mut self,
        notifier: BevyReadSignalTrigger,
    ) -> bool {
        let resource_id = with_world_ref(|world| world.components().resource_id::<R>()).expect("BevyLeptosState::track_resource() Could not track resource as it hasn't been initialised within the world yet.");
        if let Some(notifiers) = self.resource_trackers.get_mut(&resource_id) {
            notifiers.remove(&notifier)
        } else {
            false
        }
    }
}

#[derive(Default, Clone, Deref)]
pub struct BevyLeptosContext(RwSignal<BevyLeptosState>);

pub fn use_bevy() -> RwSignal<BevyLeptosState> {
    *use_context::<BevyLeptosContext>().expect("use_bevy() No `BevyLeptosContext` struct in the current reactive owner.  Is this being called from within a leptos root? ")
}

#[allow(dead_code)]
pub fn use_bevy_read_slice<TValue: PartialEq + Send + Sync + 'static>(
    getter: impl Fn(&BevyLeptosState) -> TValue + Send + Sync + Copy + 'static,
) -> Signal<TValue> {
    let ctx = use_bevy();
    create_read_slice(ctx, getter)
}

pub fn use_bevy_write_slice<TValueSet>(
    setter: impl Fn(&mut BevyLeptosState, TValueSet)
        + Copy
        + Send
        + Sync
        + Copy
        + 'static,
) -> SignalSetter<TValueSet> {
    let ctx = use_bevy();
    create_write_slice(ctx, setter)
}

#[allow(dead_code)]
pub fn use_bevy_slice<TValue: PartialEq + Send + Sync + 'static, TValueSet>(
    getter: impl Fn(&BevyLeptosState) -> TValue + Send + Sync + Copy + 'static,
    setter: impl Fn(&mut BevyLeptosState, TValueSet)
        + Copy
        + Send
        + Sync
        + Copy
        + 'static,
) -> (Signal<TValue>, SignalSetter<TValueSet>) {
    let ctx = use_bevy();
    create_slice(ctx, getter, setter)
}
