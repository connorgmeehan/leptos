mod leptos_bevy;

use bevy::{asset::AssetMetaCheck, prelude::*};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use leptos::{
    reactive_graph::{effect::Effect, signal::RwSignal, traits::Get},
    tachys::view::Render,
};
use leptos_bevy::{
    entity,
    plugin::{LeptosPlugin, LeptosWorldExt},
};

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::linear_rgb(0.4, 0.4, 0.4)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy game".to_string(), // ToDo
                        // Bind to canvas included in `index.html`
                        canvas: Some("#bevy".to_owned()),
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5 and Ctrl+R
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(LeptosPlugin)
        .add_systems(Startup, setup_leptos)
        .run();
}

const X_EXTENT: f32 = 900.;

fn setup_leptos(world: &mut World) {
    world.spawn_leptos(app);
}

fn app() -> impl Render<leptos_bevy::core::renderer::BevyRenderer> {
    let value = RwSignal::new(0);
    Effect::new(move |_| {
        let v = value.get();
        info!("value = {}", v);
    });

    entity()
        .component(Name::from("App"))
        .component(Visibility::default())
}
