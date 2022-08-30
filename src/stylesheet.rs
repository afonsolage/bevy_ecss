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
pub struct CssRules {
    hash: u64,
    rules: SmallVec<[StyleRule; 8]>,
}

impl CssRules {
    pub fn parse(content: &str) -> Self {
        let mut hasher = AHasher::default();
        content.hash(&mut hasher);
        let hash = hasher.finish();

        Self {
            hash,
            rules: StyleSheetParser::parse(content),
        }
    }

    pub fn get_properties(&self, selector: &Selector, name: &str) -> Option<&PropertyValues> {
        self.rules
            .iter()
            .find(|&rule| &rule.selector == selector)
            .map(|rule| rule.tokens.get(name).map(|prop| &*prop))
            .flatten()
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.rules.iter().any(|rule| rule.has_property(name))
    }

    pub fn iter(&self) -> impl Iterator<Item = &StyleRule> {
        self.rules.iter()
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }
}

#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector: Selector,
    pub tokens: HashMap<String, PropertyValues>,
}

impl StyleRule {
    pub(crate) fn has_property(&self, name: &str) -> bool {
        self.tokens.keys().any(|str| str == name)
    }
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
            let stylesheet = CssRules::parse(content);
            load_context.set_default_asset(LoadedAsset::new(stylesheet));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["css"]
    }
}
