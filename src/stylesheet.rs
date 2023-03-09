use std::hash::{Hash, Hasher};

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::debug,
    reflect::TypeUuid,
    utils::{AHasher, HashMap},
};
use smallvec::SmallVec;

use crate::{parser::StyleSheetParser, property::PropertyValues, selector::Selector};

#[derive(Debug, TypeUuid)]
#[uuid = "14b98dd6-5425-4692-a561-5e6ae9180554"]
/// A cascading style sheet (`css`) asset file.
///
/// _Note_: This asset only store intermediate data, like rules and properties.
/// The parsing to final ECS component values is done by a internal `exclusive_system` and is
/// cached on [`Local`](bevy::prelude::Local) resources, which isn't possible to get outside the system.
pub struct StyleSheetAsset {
    path: String,
    hash: u64,
    rules: SmallVec<[StyleRule; 8]>,
}

impl StyleSheetAsset {
    /// Parses a string with a valid CSS into a list of [`StyleRule`]s.
    ///
    /// This used by internal asset loader to keep track of where each asset came from.
    /// If you are creating this struct by hand, you can safely supply an  empty string as path.
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

    /// Returns the [`PropertyValues`] on the given [`Selector`] with the given name.
    pub fn get_properties(&self, selector: &Selector, name: &str) -> Option<&PropertyValues> {
        self.rules
            .iter()
            .find(|&rule| &rule.selector == selector)
            .and_then(|rule| rule.properties.get(name))
    }

    /// Iterates over all existing rules
    pub fn iter(&self) -> impl Iterator<Item = &StyleRule> {
        self.rules.iter()
    }

    /// Internal hash computed from content and used for equality and ordering comparison
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Asset path, for debug reasons only
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// Represents a single rule inside a style sheet with a [`Selector`] which determines which entities
/// should be applied the [`PropertyValues`].
///
/// Note that this struct holds intermediate data, the final value is parsed by [`Property`](crate::Property) on
/// the first time it's [`system`](crate::Property::apply_system) is invoked.
#[derive(Debug, Clone)]
pub struct StyleRule {
    /// Selector used to match entities to apply properties.
    pub selector: Selector,
    /// Properties values to be applied on selected entities.
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
                StyleSheetAsset::parse(load_context.path().to_str().unwrap_or_default(), content);
            load_context.set_default_asset(LoadedAsset::new(stylesheet));
            debug!("Loaded!");
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["css"]
    }
}
