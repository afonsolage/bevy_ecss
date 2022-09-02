use bevy::{asset::AssetServerSettings, prelude::*, ui::FocusPolicy};
use bevy_ecss::prelude::{Class, CssRules, EcssPlugin, RegisterComponentSelector, StyleSheet};
use bevy_inspector_egui::WorldInspectorPlugin;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
struct Title;

fn main() {
    App::new()
        // Whenever an StyleSheet is loaded, it'll be applied automatically
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EcssPlugin)
        .add_plugin(WorldInspectorPlugin::default())
        .add_startup_system(setup)
        .add_system(change_theme)
        .register_component_selector::<Title>("title")
        .run();
}

struct Themes {
    pub root: Entity,
    pub dark: Handle<CssRules>,
    pub light: Handle<CssRules>,
}

fn change_theme(
    themes: Res<Themes>,
    mut styles_query: Query<&mut StyleSheet>,
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in &interaction_query {
        match *interaction {
            Interaction::Clicked => {
                if let Ok(mut sheet) = styles_query.get_mut(themes.root) {
                    if sheet.handle() == &themes.dark {
                        sheet.set(themes.light.clone());
                    } else {
                        sheet.set(themes.dark.clone());
                    }
                }
            }
            _ => (),
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let dark = asset_server.load("sheets/dark_theme.css");
    let light = asset_server.load("sheets/light_theme.css");

    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // root node
    let root = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            focus_policy: FocusPolicy::Pass,
            color: Color::NONE.into(),
            ..default()
        })
        .insert(Name::new("ui-root"))
        .insert(StyleSheet::new(dark.clone()))
        .with_children(|parent| {
            // left vertical fill (border)
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    color: Color::rgb(0.65, 0.65, 0.65).into(),
                    ..default()
                })
                .insert(Name::new("left-border"))
                .with_children(|parent| {
                    // left vertical fill (content)
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                align_items: AlignItems::FlexEnd,
                                ..default()
                            },
                            color: Color::rgb(0.15, 0.15, 0.15).into(),
                            ..default()
                        })
                        .insert(Name::new("left-bg"))
                        .with_children(|parent| {
                            // text
                            parent
                                .spawn_bundle(
                                    TextBundle::from_section(
                                        "Text Example",
                                        TextStyle {
                                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                            font_size: 30.0,
                                            color: Color::WHITE,
                                        },
                                    )
                                    .with_style(Style {
                                        margin: UiRect::all(Val::Px(5.0)),
                                        ..default()
                                    }),
                                )
                                .insert(Name::new("left-text"));
                        });
                });
            // right vertical fill
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::Center,
                        size: Size::new(Val::Px(200.0), Val::Percent(100.0)),
                        ..default()
                    },
                    color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                })
                .insert(Name::new("right-border"))
                .with_children(|parent| {
                    // Title
                    parent
                        .spawn_bundle(
                            TextBundle::from_section(
                                "Scrolling list",
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 25.,
                                    color: Color::WHITE,
                                },
                            )
                            .with_style(Style {
                                size: Size::new(Val::Undefined, Val::Px(25.)),
                                margin: UiRect {
                                    left: Val::Auto,
                                    right: Val::Auto,
                                    ..default()
                                },
                                ..default()
                            }),
                        )
                        .insert(Title)
                        .insert(Name::new("right-bg"));
                    // List with hidden overflow
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::ColumnReverse,
                                align_self: AlignSelf::Center,
                                size: Size::new(Val::Percent(100.0), Val::Percent(50.0)),
                                overflow: Overflow::Hidden,
                                ..default()
                            },
                            color: Color::rgb(0.10, 0.10, 0.10).into(),
                            ..default()
                        })
                        .insert(Name::new("right-list"))
                        .with_children(|parent| {
                            // Moving panel
                            parent
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::ColumnReverse,
                                        flex_grow: 1.0,
                                        max_size: Size::new(Val::Undefined, Val::Undefined),
                                        ..default()
                                    },
                                    color: Color::NONE.into(),
                                    ..default()
                                })
                                .insert(Name::new("right-moving-panel"))
                                .with_children(|parent| {
                                    // List items
                                    for i in 0..30 {
                                        parent
                                            .spawn_bundle(
                                                TextBundle::from_section(
                                                    format!("Item {i}"),
                                                    TextStyle {
                                                        font: asset_server
                                                            .load("fonts/FiraSans-Bold.ttf"),
                                                        font_size: 20.,
                                                        color: Color::WHITE,
                                                    },
                                                )
                                                .with_style(Style {
                                                    flex_shrink: 0.,
                                                    size: Size::new(Val::Undefined, Val::Px(20.)),
                                                    margin: UiRect {
                                                        left: Val::Auto,
                                                        right: Val::Auto,
                                                        ..default()
                                                    },
                                                    ..default()
                                                }),
                                            )
                                            .insert(Class::new("big-text"))
                                            .insert(Name::new(format!("right-item-{}", i)));
                                    }
                                });
                        });
                });

            // render order test: reddest in the back, whitest in the front (flex center)
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    color: Color::NONE.into(),
                    ..default()
                })
                .insert(Name::new("mid-red-last"))
                .insert(Class::new("blue-bg container"))
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                ..default()
                            },
                            color: Color::rgb(1.0, 0.0, 0.0).into(),
                            ..default()
                        })
                        .insert(Name::new("mid-red-last-but-one"))
                        .with_children(|parent| {
                            parent
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                        position_type: PositionType::Absolute,
                                        position: UiRect {
                                            left: Val::Px(20.0),
                                            bottom: Val::Px(20.0),
                                            ..default()
                                        },
                                        ..default()
                                    },
                                    color: Color::rgb(1.0, 0.3, 0.3).into(),
                                    ..default()
                                })
                                .insert(Name::new("mid-red-center"));
                            parent
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                        position_type: PositionType::Absolute,
                                        position: UiRect {
                                            left: Val::Px(40.0),
                                            bottom: Val::Px(40.0),
                                            ..default()
                                        },
                                        ..default()
                                    },
                                    color: Color::rgb(1.0, 0.5, 0.5).into(),
                                    ..default()
                                })
                                .insert(Class::new("blue-bg"))
                                .insert(Name::new("mid-red-top-but-one"));
                            parent
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                        position_type: PositionType::Absolute,
                                        position: UiRect {
                                            left: Val::Px(60.0),
                                            bottom: Val::Px(60.0),
                                            ..default()
                                        },
                                        ..default()
                                    },
                                    color: Color::rgb(1.0, 0.7, 0.7).into(),
                                    ..default()
                                })
                                .insert(Name::new("mid-red-top"));
                            // alpha test
                            parent
                                .spawn_bundle(NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(100.0), Val::Px(100.0)),
                                        position_type: PositionType::Absolute,
                                        position: UiRect {
                                            left: Val::Px(80.0),
                                            bottom: Val::Px(80.0),
                                            ..default()
                                        },
                                        ..default()
                                    },
                                    color: Color::rgba(1.0, 0.9, 0.9, 0.4).into(),
                                    ..default()
                                })
                                .insert(Class::new("blue-bg"))
                                .insert(Name::new("mid-red-alpha"));
                        });
                });
            // bevy logo (flex center)
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        position_type: PositionType::Absolute,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        ..default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    color: Color::NONE.into(),
                    ..default()
                })
                .insert(Name::new("mid-bevy-logo-bg"))
                .with_children(|parent| {
                    // bevy logo (image)
                    parent
                        .spawn_bundle(ImageBundle {
                            style: Style {
                                size: Size::new(Val::Px(500.0), Val::Auto),
                                ..default()
                            },
                            image: asset_server.load("branding/bevy_logo_dark_big.png").into(),
                            ..default()
                        })
                        .insert(Name::new("mid-bevy-logo-image"));
                });
            // absolute positioning
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Px(200.0)),
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Px(210.0),
                            bottom: Val::Px(10.0),
                            ..default()
                        },
                        border: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    color: Color::rgb(0.4, 0.4, 1.0).into(),
                    ..default()
                })
                .insert(Name::new("mid-blue-border"))
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..default()
                            },
                            focus_policy: FocusPolicy::Pass,
                            color: Color::rgb(0.8, 0.8, 1.0).into(),
                            ..default()
                        })
                        .insert(Name::new("mid-navy-blue-content"))
                        .with_children(|parent| {
                            parent
                                .spawn_bundle(ButtonBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn_bundle(TextBundle::from_section(
                                        "Change Theme",
                                        TextStyle {
                                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                            font_size: 40.0,
                                            color: Color::rgb(0.9, 0.9, 0.9),
                                        },
                                    ));
                                });
                        });
                });
        })
        .id();

    commands.insert_resource(Themes { root, dark, light })
}
