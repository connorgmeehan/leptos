use bevy::{ecs::system::RunSystemWithInput, prelude::Deref};
use leptos::{context::use_context, reactive_graph::signal::RwSignal};

pub mod elements;
pub mod node;
pub mod properties;
pub mod renderer;
pub mod view;
pub mod provider;
pub mod children;

#[derive(Clone)]
pub struct BevyLeptosState {
    pub value: String,
}

#[derive(Clone, Deref)]
pub struct BevyLeptosContext(RwSignal<BevyLeptosState>);

impl BevyLeptosContext {
    pub fn new() -> Self {
        Self(RwSignal::new(BevyLeptosState { value: "Value from context :)".to_string() }))
    }
}

pub fn use_bevy() -> RwSignal<BevyLeptosState> {
    *use_context::<BevyLeptosContext>().unwrap()
}
