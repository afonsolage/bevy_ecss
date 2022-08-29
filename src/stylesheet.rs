use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::Deref,
    reflect::TypeUuid,
};
use smallvec::SmallVec;

use crate::{parser::StyleSheetParser, property::Property, selector::Selector};

#[derive(Debug, TypeUuid, Deref)]
#[uuid = "14b98dd6-5425-4692-a561-5e6ae9180554"]
pub struct CssRules(SmallVec<[StyleRule; 8]>);

impl CssRules {
    pub fn parse(content: &str) -> Self {
        Self(StyleSheetParser::parse(content))
    }
}

#[derive(Debug, Clone)]
pub struct StyleRule(pub Selector, pub SmallVec<[Property; 8]>);

#[derive(Default)]
pub(crate) struct StyleSheetLoader;

impl AssetLoader for StyleSheetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let content = std::str::from_utf8(bytes)?;
            let stylesheet = CssRules::parse(content);
            load_context.set_default_asset(LoadedAsset::new(stylesheet));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["css"]
    }
}
