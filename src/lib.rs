mod component;
mod parser;
mod property;
mod selector;
mod stylesheet;
mod system;

use bevy::{
    asset::AssetServerSettings,
    prelude::{AddAsset, ParallelSystemDescriptorCoercion, Plugin, SystemLabel},
};
use property::{Property, StyleSheetState};
use stylesheet::StyleSheetLoader;

pub use component::{Class, StyleSheet};
pub use selector::{Selector, SelectorElement};
pub use stylesheet::CssRules;

pub trait RegisterProperty {
    fn register_property<T>(&mut self)
    where
        T: Property + 'static;
}

impl RegisterProperty for bevy::prelude::App {
    fn register_property<T>(&mut self)
    where
        T: Property + 'static,
    {
        self.add_system(
            T::apply_system
                .label(EcssSystem::Apply)
                .before(EcssSystem::Cleanup)
                .after(EcssSystem::Prepare), // .with_run_criteria(T::have_property)
        );
    }
}

#[derive(SystemLabel)]
pub enum EcssSystem {
    Prepare,
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
            .init_asset_loader::<StyleSheetLoader>()
            .add_system(
                system::clear_state
                    .label(EcssSystem::Cleanup)
                    .after(EcssSystem::Apply)
                    .after(EcssSystem::Prepare),
            )
            .add_system(
                system::prepare_state
                    .label(EcssSystem::Prepare)
                    .before(EcssSystem::Apply)
                    .before(EcssSystem::Cleanup),
            );

        register_properties(app); 

        if let Some(settings) = app.world.get_resource::<AssetServerSettings>() && settings.watch_for_changes {
            app.add_system(system::hot_reload_style_sheets.before(EcssSystem::Prepare));
        }
    }
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

    app.register_property::<UiColorProperty>();
}
