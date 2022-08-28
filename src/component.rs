use std::borrow::Cow;

use bevy::{
    asset::{AssetLoader, LoadedAsset},
    prelude::{Component, Deref, Name, ReflectComponent},
    reflect::Reflect,
};
use smallvec::SmallVec;

use crate::StyleSheet;

pub trait MatchSelectorElement {
    fn matches(&self, element: &str) -> bool;
}

#[derive(Debug, Reflect, Component, Default, Clone, Deref)]
#[reflect(Component)]
pub struct CssClass(SmallVec<[Cow<'static, str>; 4]>);

impl CssClass {
    pub fn new<T>(classes: &[T]) -> Self
    where
        T: Into<Cow<'static, str>> + Clone,
    {
        Self(classes.into_iter().map(|t| t.clone().into()).collect())
    }

    fn matches(&self, class: &str) -> bool {
        self.0.iter().any(|c| c.as_ref() == class)
    }
}

impl MatchSelectorElement for CssClass {
    fn matches(&self, element: &str) -> bool {
        self.matches(element)
    }
}

impl MatchSelectorElement for Name {
    fn matches(&self, element: &str) -> bool {
        self.as_str() == element
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
            let stylesheet = StyleSheet::parse(content);
            load_context.set_default_asset(LoadedAsset::new(stylesheet));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["css"]
    }
}
