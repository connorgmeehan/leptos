mod leptos_bevy;

use bevy::{asset::AssetMetaCheck, prelude::*};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use leptos::reactive_graph::traits::Get;
use leptos_bevy::{
    core::{elements::entity, use_bevy, view::IntoBevyView}, hooks::use_resource::use_resource, plugin::LeptosPlugin, world_ext::LeptosWorldExt
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
        .insert_resource(MyResource::default())
        .add_systems(Startup, setup_leptos)
        .add_systems(Update, update_my_resource)
        .run();
}

#[derive(Default, Resource, Clone)]
pub struct MyResource(pub usize);

pub fn update_my_resource(mut resource: ResMut<MyResource>) {
    resource.0 += 1;
}

fn setup_leptos(world: &mut World) {
    world.spawn_leptos(app);
}

fn app() -> impl IntoBevyView {
    let v = use_resource::<MyResource>();

    entity()
        .component(Name::from(format!("App {}", v.get().0)))
        .component(Visibility::default())
}
