mod colors;
mod component;
mod parser;
mod property;
mod selector;
mod stylesheet;
mod system;

use bevy::{
    asset::AssetServerSettings,
    prelude::{AddAsset, CoreStage, Plugin, SystemLabel, SystemSet},
    ui::UiSystem,
};
use stylesheet::StyleSheetLoader;

pub use component::{Class, StyleSheet};
pub use stylesheet::CssRules;

#[derive(SystemLabel)]
pub enum EcssSystem {
    Apply,
}

#[derive(Default)]
pub struct EcssPlugin;

impl Plugin for EcssPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<Class>()
            .register_type::<StyleSheet>()
            .add_asset::<CssRules>()
            .init_asset_loader::<StyleSheetLoader>()
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .label(EcssSystem::Apply)
                    .with_system(system::apply_style_sheet)
                    .before(UiSystem::Flex),
            );

        if let Some(settings) = app.world.get_resource::<AssetServerSettings>() && settings.watch_for_changes {
            app.add_system(system::reload_style_sheets);
        }
    }
}
