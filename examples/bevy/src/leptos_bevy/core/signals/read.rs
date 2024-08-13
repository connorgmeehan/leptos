use std::{
    fmt::{Debug, Formatter, Result},
    hash::Hash,
    panic::Location,
    sync::{Arc, RwLock},
};

use bevy::{ecs::world::World, time::Time};

use leptos::reactive_graph::{
    graph::SubscriberSet,
    signal::subscriber_traits::AsSubscriberSet,
    traits::{DefinedAt, IsDisposed, WithUntracked},
};

use crate::leptos_bevy::core::renderer::with_world_ref;

use super::BevyReadSignalTrigger;

pub struct BevyReadSignal<T> {
    #[cfg(debug_assertions)]
    pub defined_at: &'static Location<'static>,
    pub getter: Arc<dyn for<'a> Fn(&'a World) -> Option<&'a T>>,
    pub inner: Arc<RwLock<SubscriberSet>>,
}

impl<T> BevyReadSignal<T> {
    pub fn new(
        getter: impl for<'a> Fn(&'a World) -> Option<&'a T> + 'static,
    ) -> Self {
        Self {
            defined_at: std::panic::Location::caller(),
            getter: Arc::new(getter),
            inner: Arc::new(RwLock::new(SubscriberSet::default())),
        }
    }

    pub fn get_notifier(&self) -> BevyReadSignalTrigger {
        BevyReadSignalTrigger {
            defined_at: self.defined_at,
            inner: self.inner.clone(),
        }
    }
}

fn test() {
    let v = BevyReadSignal::new(|world| world.get_resource::<Time>());
}

impl<T> Clone for BevyReadSignal<T> {
    #[track_caller]
    fn clone(&self) -> Self {
        Self {
            #[cfg(debug_assertions)]
            defined_at: self.defined_at,
            getter: Arc::clone(&self.getter),
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Debug for BevyReadSignal<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("BevyReadSignal")
            .field("type", &std::any::type_name::<T>())
            .field("getter", &Arc::as_ptr(&self.getter))
            .finish()
    }
}

impl<T> PartialEq for BevyReadSignal<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.getter, &other.getter)
    }
}

impl<T> Eq for BevyReadSignal<T> {}

impl<T> Hash for BevyReadSignal<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&Arc::as_ptr(&self.getter), state);
    }
}

impl<T> DefinedAt for BevyReadSignal<T> {
    #[inline(always)]
    fn defined_at(&self) -> Option<&'static Location<'static>> {
        #[cfg(debug_assertions)]
        {
            Some(self.defined_at)
        }
        #[cfg(not(debug_assertions))]
        {
            None
        }
    }
}

impl<T> IsDisposed for BevyReadSignal<T> {
    #[inline(always)]
    fn is_disposed(&self) -> bool {
        false
    }
}

impl<T> AsSubscriberSet for BevyReadSignal<T> {
    type Output = Arc<RwLock<SubscriberSet>>;

    #[inline(always)]
    fn as_subscriber_set(&self) -> Option<Self::Output> {
        Some(Arc::clone(&self.inner))
    }
}

impl<T> WithUntracked for BevyReadSignal<T> {
    type Value = T;

    fn try_with_untracked<U>(
        &self,
        fun: impl FnOnce(&Self::Value) -> U,
    ) -> Option<U> {
        with_world_ref(|w| {
            let value = (self.getter)(w);
            value.map(|value| fun(&value))
        })
    }
}
