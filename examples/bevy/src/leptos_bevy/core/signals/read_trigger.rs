use std::{
    hash::Hash, panic::Location, sync::{Arc, RwLock}
};

use leptos::reactive_graph::{
    graph::{ReactiveNode, SubscriberSet},
    signal::subscriber_traits::AsSubscriberSet,
    traits::{DefinedAt, Trigger},
};

#[derive(Clone)]
pub struct BevyReadSignalTrigger {
    #[cfg(debug_assertions)]
    pub defined_at: &'static Location<'static>,
    pub inner: Arc<RwLock<SubscriberSet>>,
}

impl PartialEq for BevyReadSignalTrigger {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for BevyReadSignalTrigger {}

impl Hash for BevyReadSignalTrigger {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&Arc::as_ptr(&self.inner), state);
    }
}

impl DefinedAt for BevyReadSignalTrigger {
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

impl AsSubscriberSet for BevyReadSignalTrigger {
    type Output = Arc<RwLock<SubscriberSet>>;

    #[inline(always)]
    fn as_subscriber_set(&self) -> Option<Self::Output> {
        Some(Arc::clone(&self.inner))
    }
}

impl Trigger for BevyReadSignalTrigger {
    fn trigger(&self) {
        self.mark_dirty();
    }
}
