use std::marker::PhantomData;

use bevy::ecs::{system::Resource, world::World};
use leptos::{reactive_graph::{owner::on_cleanup, traits::Set}, tachys::html::node_ref::node_ref};

use crate::leptos_bevy::core::{
    signals::{BevyReadSignal, BevyReadSignalTrigger},
    use_bevy_write_slice,
};

#[derive(PartialEq)]
pub enum UseResourceTrackersAction<R: Resource> {
    TrackResource {
        notifier: BevyReadSignalTrigger,
        pd: PhantomData<R>,
    },
    UntrackResource {
        notifier: BevyReadSignalTrigger,
        pd: PhantomData<R>,
    },
}

pub fn use_resource<R: Resource>() -> BevyReadSignal<R> {
    let signal = BevyReadSignal::new(|world| world.get_resource::<R>());
    let dispatch =
        use_bevy_write_slice(|leptos_bevy, value: UseResourceTrackersAction<R>| {
            match value {
                UseResourceTrackersAction::TrackResource {
                    notifier, ..
                } => {
                    leptos_bevy.track_resource::<R>(notifier);
                }
                UseResourceTrackersAction::UntrackResource {
                    notifier, ..
                } => {
                    leptos_bevy.untrack_resource::<R>(notifier);
                }
            };
            println!("{leptos_bevy:?}");
        });

    dispatch.set(UseResourceTrackersAction::TrackResource {
        notifier: signal.get_notifier(),
        pd: PhantomData::<R>,
    });

    let notifier = signal.get_notifier();
    on_cleanup(move || {
        dispatch.set(UseResourceTrackersAction::UntrackResource {
            notifier,
            pd: PhantomData::<R>,
        });
    });

    // create_lifecycle_entity(|world| {
    //     world.spawn(LeptosBevyTrackedResource::<R> {
    //         pd: PhantomData,
    //         notifier: signal.get_notifier(),
    //     }).id()
    // });
    signal
}
