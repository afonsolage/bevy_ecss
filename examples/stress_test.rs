use bevy::prelude::*;
use bevy_ecss::prelude::{Class, EcssPlugin, StyleSheet};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                canvas: Some("#bevy".to_string()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EcssPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            ..Default::default()
        })
        .insert(Name::new("root"))
        .insert(StyleSheet::new(asset_server.load("sheets/stress.css")))
        .with_children(|builder| {
            for _ in 0..10 {
                builder
                    .spawn(NodeBundle {
                        style: Style {
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(Class::new("red"))
                    .with_children(|builder| {
                        for _ in 0..10 {
                            builder
                                .spawn(NodeBundle {
                                    ..Default::default()
                                })
                                .insert(Class::new("green"))
                                .with_children(|builder| {
                                    for _ in 0..10 {
                                        builder
                                            .spawn(NodeBundle {
                                                style: Style {
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            })
                                            .insert(Class::new("blue"))
                                            .with_children(|builder| {
                                                for _ in 0..10 {
                                                    builder
                                                        .spawn(NodeBundle {
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
