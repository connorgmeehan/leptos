use std::{
    fmt::{Debug, Formatter, Result},
    hash::Hash,
    panic::Location,
    sync::{Arc, RwLock},
};

use bevy::{ecs::world::World, time::Time};

use leptos::reactive_graph::{
    graph::{ReactiveNode, SubscriberSet},
    signal::subscriber_traits::AsSubscriberSet,
    traits::{DefinedAt, IsDisposed, WithUntracked},
};

use crate::leptos_bevy::core::renderer::with_world_ref;

pub struct BevyReadSignal<T, TGetter: for<'a> Fn(&'a World) -> Option<&'a T>> {
    #[cfg(debug_assertions)]
    pub defined_at: &'static Location<'static>,
    pub getter: Arc<TGetter>,
    pub inner: Arc<RwLock<SubscriberSet>>,
}

impl<T, TGetter: for<'a> Fn(&'a bevy::prelude::World) -> Option<&'a T>>
    BevyReadSignal<T, TGetter>
{
    pub fn new(getter: TGetter) -> Self {
        Self {
            defined_at: std::panic::Location::caller(),
            getter: Arc::new(getter),
            inner: Arc::new(RwLock::new(SubscriberSet::default())),
        }
    }
}

fn test() {
    let v = BevyReadSignal::new(|world| world.get_resource::<Time>());
}

impl<T, TGetter: Fn(&World) -> Option<&T>> Clone
    for BevyReadSignal<T, TGetter>
{
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

impl<T, TGetter: Fn(&World) -> Option<&T>> Debug
    for BevyReadSignal<T, TGetter>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("BevyReadSignal")
            .field("type", &std::any::type_name::<T>())
            .field("getter", &Arc::as_ptr(&self.getter))
            .finish()
    }
}

impl<T, TGetter: Fn(&World) -> Option<&T>> PartialEq
    for BevyReadSignal<T, TGetter>
{
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.getter, &other.getter)
    }
}

impl<T, TGetter: Fn(&World) -> Option<&T>> Eq for BevyReadSignal<T, TGetter> {}

impl<T, TGetter: Fn(&World) -> Option<&T>> Hash for BevyReadSignal<T, TGetter> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&Arc::as_ptr(&self.getter), state);
    }
}

impl<T, TGetter: Fn(&World) -> Option<&T>> DefinedAt
    for BevyReadSignal<T, TGetter>
{
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

impl<T, TGetter: Fn(&World) -> Option<&T>> IsDisposed
    for BevyReadSignal<T, TGetter>
{
    #[inline(always)]
    fn is_disposed(&self) -> bool {
        false
    }
}

impl<T, TGetter: Fn(&World) -> Option<&T>> AsSubscriberSet
    for BevyReadSignal<T, TGetter>
{
    type Output = Arc<RwLock<SubscriberSet>>;

    #[inline(always)]
    fn as_subscriber_set(&self) -> Option<Self::Output> {
        Some(Arc::clone(&self.inner))
    }
}

impl<T, TGetter: Fn(&World) -> Option<&T>> WithUntracked
    for BevyReadSignal<T, TGetter>
{
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
