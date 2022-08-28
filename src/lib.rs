mod colors;
mod component;
mod parser;
mod property;
mod selector;
mod stylesheet;
mod system;

use bevy::{
    prelude::{AddAsset, CoreStage, Deref, Handle, Plugin, SystemLabel, SystemSet},
    ui::UiSystem,
    utils::default,
};
use component::StyleSheetLoader;

pub use component::CssClass;
pub use stylesheet::StyleSheet;

#[derive(SystemLabel)]
pub enum EcssSystem {
    Apply,
}

#[derive(Default)]
pub struct EcssPlugin {
    auto_apply_on_load: bool,
}

impl EcssPlugin {
    pub fn new() -> Self {
        default()
    }

    pub fn apply_on_load() -> Self {
        Self {
            auto_apply_on_load: true,
        }
    }
}

impl Plugin for EcssPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<CssClass>()
            .add_asset::<StyleSheet>()
            .add_event::<ApplyStyleSheet>()
            .init_asset_loader::<StyleSheetLoader>()
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .label(EcssSystem::Apply)
                    .with_system(system::apply_style_sheet)
                    .before(UiSystem::Flex),
            );

        if self.auto_apply_on_load {
            app.add_system(system::apply_loaded_style_sheets);
        }
    }
}

#[derive(Debug, Deref)]
pub struct ApplyStyleSheet(Handle<StyleSheet>);

impl ApplyStyleSheet {
    pub fn new(handle: Handle<StyleSheet>) -> Self {
        Self(handle)
    }
}

impl From<Handle<StyleSheet>> for ApplyStyleSheet {
    fn from(handle: Handle<StyleSheet>) -> Self {
        Self::new(handle)
    }
}
