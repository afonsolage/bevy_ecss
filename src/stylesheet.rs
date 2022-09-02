use std::hash::{Hash, Hasher};

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    reflect::TypeUuid,
    utils::{AHasher, HashMap},
};
use smallvec::SmallVec;

use crate::{parser::StyleSheetParser, property::PropertyValues, selector::Selector};

#[derive(Debug, TypeUuid)]
#[uuid = "14b98dd6-5425-4692-a561-5e6ae9180554"]
// Find a better name for the StyleSheet asset.
pub struct CssRules {
    path: String,
    hash: u64,
    rules: SmallVec<[StyleRule; 8]>,
}

impl CssRules {
    pub fn parse(path: &str, content: &str) -> Self {
        let mut hasher = AHasher::default();
        content.hash(&mut hasher);
        let hash = hasher.finish();

        Self {
            path: path.to_string(),
            hash,
            rules: StyleSheetParser::parse(content),
        }
    }

    pub fn get_properties(&self, selector: &Selector, name: &str) -> Option<&PropertyValues> {
        self.rules
            .iter()
            .find(|&rule| &rule.selector == selector)
            .map(|rule| rule.properties.get(name).map(|prop| &*prop))
            .flatten()
    }

    pub fn iter(&self) -> impl Iterator<Item = &StyleRule> {
        self.rules.iter()
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector: Selector,
    pub properties: HashMap<String, PropertyValues>,
}

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
            let stylesheet =
                CssRules::parse(load_context.path().to_str().unwrap_or_default(), content);
            load_context.set_default_asset(LoadedAsset::new(stylesheet));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["css"]
    }
}
