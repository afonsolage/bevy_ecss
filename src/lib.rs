mod component;
mod parser;
mod property;
mod selector;
mod stylesheet;
mod system;

use std::{error::Error, fmt::Display};

use bevy::{
    asset::AssetServerSettings,
    ecs::system::SystemState,
    prelude::{
        AddAsset, Button, Component, CoreStage, Entity, ExclusiveSystemDescriptorCoercion,
        IntoExclusiveSystem, ParallelSystemDescriptorCoercion, Plugin, Query, SystemLabel, With,
    },
    text::Text,
    ui::{Interaction, Node, Style, UiColor, UiImage},
};
use property::StyleSheetState;
use stylesheet::StyleSheetLoader;

pub use component::{Class, StyleSheet};
pub use property::{Property, PropertyToken, PropertyValues};
pub use selector::{Selector, SelectorElement};
pub use stylesheet::{CssRules, StyleRule};
use system::{ComponentFilterRegistry, PrepareParams};

#[derive(Debug)]
pub enum EcssError {
    UnsupportedSelector,
    // TODO: Change this to Cow<'static, str>
    UnsupportedProperty(String),
    InvalidPropertyValue(String),
    InvalidSelector,
    UnexpectedToken(String),
}

impl Error for EcssError {}

impl Display for EcssError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EcssError::UnsupportedSelector => {
                write!(f, "Unsupported selector")
            }
            EcssError::UnsupportedProperty(p) => write!(f, "Unsupported property: {}", p),
            EcssError::InvalidPropertyValue(p) => write!(f, "Invalid property value: {}", p),
            EcssError::InvalidSelector => write!(f, "Invalid selector"),
            EcssError::UnexpectedToken(t) => write!(f, "Unexpected token: {}", t),
        }
    }
}

#[derive(SystemLabel)]
pub enum EcssSystem {
    Apply,
    Cleanup,
}

#[derive(Default)]
pub struct EcssPlugin;

impl Plugin for EcssPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Class>()
            .register_type::<StyleSheet>()
            .add_asset::<CssRules>()
            .init_resource::<StyleSheetState>()
            .init_resource::<ComponentFilterRegistry>()
            .init_asset_loader::<StyleSheetLoader>()
            .add_system(system::prepare.exclusive_system().at_start())
            .add_system_to_stage(
                CoreStage::PostUpdate,
                system::clear_state.label(EcssSystem::Cleanup),
            );

        let prepared_state = PrepareParams::new(&mut app.world);
        app.insert_resource(prepared_state);

        register_component_selector(app);
        register_properties(app);

        if let Some(settings) = app.world.get_resource::<AssetServerSettings>() && settings.watch_for_changes {
            app.add_system_to_stage(CoreStage::PreUpdate, system::hot_reload_style_sheets);
        }
    }
}
fn register_component_selector(app: &mut bevy::prelude::App) {
    app.register_component_selector::<UiColor>("ui-color");
    app.register_component_selector::<Text>("text");
    app.register_component_selector::<Button>("button");
    app.register_component_selector::<Node>("node");
    app.register_component_selector::<Style>("style");
    app.register_component_selector::<UiImage>("ui-image");
    app.register_component_selector::<Interaction>("interaction");
}

fn register_properties(app: &mut bevy::prelude::App) {
    use property::impls::*;

    app.register_property::<DisplayProperty>();
    app.register_property::<PositionTypeProperty>();
    app.register_property::<DirectionProperty>();
    app.register_property::<FlexDirectionProperty>();
    app.register_property::<FlexWrapProperty>();
    app.register_property::<AlignItemsProperty>();
    app.register_property::<AlignSelfProperty>();
    app.register_property::<AlignContentProperty>();
    app.register_property::<JustifyContentProperty>();
    app.register_property::<OverflowProperty>();

    app.register_property::<LeftProperty>();
    app.register_property::<RightProperty>();
    app.register_property::<TopProperty>();
    app.register_property::<BottomProperty>();
    app.register_property::<WidthProperty>();
    app.register_property::<HeightProperty>();
    app.register_property::<MinWidthProperty>();
    app.register_property::<MinHeightProperty>();
    app.register_property::<MaxWidthProperty>();
    app.register_property::<MaxHeightProperty>();
    app.register_property::<FlexBasisProperty>();
    app.register_property::<FlexGrowProperty>();
    app.register_property::<FlexShrinkProperty>();
    app.register_property::<AspectRatioProperty>();

    app.register_property::<MarginProperty>();
    app.register_property::<PaddingProperty>();
    app.register_property::<BorderProperty>();

    app.register_property::<FontColorProperty>();
    app.register_property::<FontProperty>();
    app.register_property::<FontSizeProperty>();
    app.register_property::<VerticalAlignProperty>();
    app.register_property::<HorizontalAlignProperty>();
    app.register_property::<TextContentProperty>();

    app.register_property::<UiColorProperty>();
}

pub trait RegisterComponentSelector {
    fn register_component_selector<T>(&mut self, name: &'static str) -> &mut Self
    where
        T: Component;
}

impl RegisterComponentSelector for bevy::prelude::App {
    fn register_component_selector<T>(&mut self, name: &'static str) -> &mut Self
    where
        T: Component,
    {
        let system_state = SystemState::<Query<Entity, With<T>>>::new(&mut self.world);
        let boxed_state = Box::new(system_state);

        self.world
            .get_resource_or_insert_with::<ComponentFilterRegistry>(|| {
                ComponentFilterRegistry(Default::default())
            })
            .0
            .insert(name, boxed_state);

        self
    }
}

pub trait RegisterProperty {
    fn register_property<T>(&mut self) -> &mut Self
    where
        T: Property + 'static;
}

impl RegisterProperty for bevy::prelude::App {
    fn register_property<T>(&mut self) -> &mut Self
    where
        T: Property + 'static,
    {
        self.add_system(
            T::apply_system
                .label(EcssSystem::Apply)
                .with_run_criteria(T::have_property),
        );

        self
    }
}
