use bevy::{prelude::*, ui::FocusPolicy};
use bevy_ecss::prelude::{EcssPlugin, StyleSheet};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#bevy".to_string()),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EcssPlugin::with_hot_reload())
    .add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // root node
    commands
        .spawn(NodeBundle {
            focus_policy: FocusPolicy::Pass,
            ..default()
        })
        .insert(Name::new("ui-root"))
        .insert(StyleSheet::new(asset_server.load("sheets/interactive.css")))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                })
                .insert(Name::new("list"))
                .with_children(|parent| {
                    // Moving panel
                    parent
                        .spawn(NodeBundle {
                            focus_policy: FocusPolicy::Pass,
                            ..default()
                        })
                        .insert(Name::new("panel"))
                        .with_children(|parent| {
                            // List items
                            for i in 0..30 {
                                parent
                                    .spawn(TextBundle::from_section(
                                        format!("Item {i}"),
                                        TextStyle::default(),
                                    ))
                                    .insert(Interaction::default());
                            }
                        });
                });
        });
}
