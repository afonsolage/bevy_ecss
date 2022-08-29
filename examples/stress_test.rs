use bevy::{asset::AssetServerSettings, prelude::*};
use bevy_ecss::{Class, EcssPlugin, StyleSheet};

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        // Whenever a StyleSheet is loaded, it'll be automatically applied.
        // This also works for asset hot reloading
        .add_plugin(EcssPlugin::apply_on_load())
        .add_startup_system(load_stylesheet)
        .add_startup_system(setup)
        .run();
}

fn load_stylesheet(asset_server: Res<AssetServer>, mut commands: Commands) {
    let stylesheet: Handle<StyleSheet> = asset_server.load("sheets/stress.css");

    // The Handle needs to be stored somewhere to prevent AssetServer from unloading it
    commands.insert_resource(stylesheet)
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn_bundle(NodeBundle {
            ..Default::default()
        })
        .insert(Name::new("root"))
        .with_children(|builder| {
            for _ in 0..10 {
                builder
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Class::new("red"))
                    .with_children(|builder| {
                        for _ in 0..10 {
                            builder
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .insert(Class::new("green"))
                                .with_children(|builder| {
                                    for _ in 0..10 {
                                        builder
                                            .spawn_bundle(NodeBundle {
                                                style: Style {
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            })
                                            .insert(Class::new("blue"))
                                            .with_children(|builder| {
                                                for _ in 0..10 {
                                                    builder
                                                        .spawn_bundle(NodeBundle {
                                                            ..Default::default()
                                                        })
                                                        .insert(Class::new("purple"));
                                                }
                                            });
                                    }
                                });
                        }
                    });
            }
        });
}
