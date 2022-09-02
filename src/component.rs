use std::borrow::Cow;

use bevy::prelude::{Component, Deref, Handle, Name, Reflect, ReflectComponent};

use crate::CssRules;

/// Sets the entities class to be matched by selectors in on`css`.
///
/// The behavior mimics CSS so a single class name can given or a list separated by spaces.
///
/// # Examples
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_ecss::prelude::*;
/// fn system(mut commands: Commands) {
///     // This entity can be selected by either ".yellow-button", ".enabled"
///     // or even ".yellow-button.enabled"
///     commands.spawn().insert(Class::new("yellow-button enabled"));
/// }
/// ```
#[derive(Debug, Reflect, Component, Default, Clone, Deref)]
#[reflect(Component)]
pub struct Class(Cow<'static, str>);

impl Class {
    /// Creates a new [`Class`] with the given class names.
    ///
    /// Multiple class names can be used separated by spaces.
    pub fn new(class: impl Into<Cow<'static, str>>) -> Self {
        Self(class.into())
    }

    /// Checks if any of this class names matches the given class name
    fn matches(&self, class: &str) -> bool {
        self.0.split_ascii_whitespace().any(|c| c == class)
    }
}

/// Applied a [`CssRules`] asset on the entity which has this component.
///
/// Note that style rules are applied only once when the component is added, or if the asset is changed
/// and [hot_reloading](https://github.com/bevyengine/bevy/blob/main/examples/asset/hot_asset_reloading.rs) is enabled.
/// If you want to reapply the stylesheet, like when new children was added, use [`StyleSheet::refresh`].
///
/// # Examples
///
/// ```
/// # use bevy::prelude::*;
/// use bevy_ecss::prelude::*;
///
/// fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
///     commands.spawn().insert(StyleSheet::new(asset_server.load("sheets/fancy.css")));
/// }
///
/// ```
///
#[derive(Component, Debug, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct StyleSheet {
    sheet: Handle<CssRules>,
}

impl StyleSheet {
    /// Creates a new [`StyleSheet`] from the given asset.
    pub fn new(handle: Handle<CssRules>) -> Self {
        Self { sheet: handle }
    }

    /// Reapplies the style sheet on entity and all children.
    pub fn refresh(&mut self) {
        // Just to trigger DerefMut
    }

    /// Internal [`CssRules`] asset handle
    pub fn handle(&self) -> &Handle<CssRules> {
        &self.sheet
    }

    /// Change the internal [`CssRules`] asset handle.
    /// This will automatically trigger the systems to reapply the style sheet.
    pub fn set(&mut self, handle: Handle<CssRules>) {
        self.sheet = handle;
    }
}

impl PartialEq for StyleSheet {
    fn eq(&self, other: &Self) -> bool {
        self.sheet == other.sheet
    }
}

/// Convenience trait which matches matches a component against a named element selector.
pub(crate) trait MatchSelectorElement {
    fn matches(&self, element: &str) -> bool;
}

impl MatchSelectorElement for Class {
    fn matches(&self, element: &str) -> bool {
        self.matches(element)
    }
}

impl MatchSelectorElement for Name {
    fn matches(&self, element: &str) -> bool {
        self.as_str() == element
    }
}
