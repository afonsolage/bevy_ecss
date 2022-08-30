mod colors;
mod component;
mod parser;
mod property;
mod selector;
mod stylesheet;
mod system;

use bevy::{
    asset::AssetServerSettings,
    prelude::{AddAsset, CoreStage, ParallelSystemDescriptorCoercion, Plugin, SystemLabel},
    ui::UiSystem,
};
use property::{DisplayProperty, Property};
use stylesheet::StyleSheetLoader;

pub use component::{Class, StyleSheet};
pub use selector::{Selector, SelectorElement};
pub use stylesheet::CssRules;

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
            .init_asset_loader::<StyleSheetLoader>()
            .add_system_to_stage(
                CoreStage::PostUpdate,
                system::prepare_state
                    .label(EcssSystem::Prepare)
                    .before(EcssSystem::Apply),
            )
            .add_system_to_stage(
                CoreStage::PostUpdate,
                system::clear_state
                    .label(EcssSystem::Cleanup)
                    .after(EcssSystem::Apply)
                    .before(UiSystem::Flex),
            );

        app.register_property::<DisplayProperty>();

        if let Some(settings) = app.world.get_resource::<AssetServerSettings>() && settings.watch_for_changes {
            app.add_system(system::hot_reload_style_sheets);
        }
    }
}

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
        self.add_system_to_stage(
            CoreStage::PostUpdate,
            T::apply_system
                .with_run_criteria(T::have_property)
                .label(EcssSystem::Apply)
                .after(EcssSystem::Prepare)
                .before(EcssSystem::Cleanup),
        );
    }
}
