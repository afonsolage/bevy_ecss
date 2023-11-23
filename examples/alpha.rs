use bevy::{ecs::query::QueryItem, prelude::*};
use bevy_ecss::{prelude::*, EcssError, Property, PropertyValues};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            canvas: Some("#bevy".to_string()),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(EcssPlugin::default())
    .add_systems(Startup, setup);

    app.register_property::<AlphaProperty>();

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .insert(StyleSheet::new(asset_server.load("sheets/alpha.css")))
        .with_children(|parent| {
            // bevy logo (image)
            parent.spawn(ImageBundle::default());
        });
}

#[derive(Default)]
pub(crate) struct AlphaProperty;

impl Property for AlphaProperty {
    // This is the cached value to be used when applying the property value.
    // It is evaluated only on the first time and futures values are cached for performance reasons.
    type Cache = f32;
    // Which components we need when applying the cache. It is the same as using bevy ecs Query.
    type Components = &'static mut BackgroundColor;
    // If this property can be set only when there is another property, we may filter there.
    // It's not recommended to use only With<> and Without<>.
    type Filters = ();

    fn id() -> lightningcss::properties::PropertyId<'static> {
        // The name of property. prefer kebab-case for consistency.
        "alpha".into()
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
        // PropertyValues::f32 tries to parse property value into a numeric value
        if let Some(value) = values.f32() {
            Ok(value)
        } else {
            Err(EcssError::InvalidPropertyValue(Self::id().name().to_string()))
        }
    }

    fn apply<'w>(
        cache: &Self::Cache,
        mut components: QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        components.0.set_a(*cache);
    }
}
