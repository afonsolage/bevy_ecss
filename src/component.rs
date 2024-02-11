use std::borrow::Cow;

use bevy::prelude::{Component, Deref, Handle, Name, Reflect, ReflectComponent};

use crate::StyleSheetAsset;

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
///     commands.spawn(Class::new("yellow-button enabled"));
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

    /// Appends a new class name to this component. If the class name is already
    /// present, it will be ignored.
    pub fn add_class(&mut self, class: &str) {
        if self.matches(class) {
            return;
        }

        if self.0.is_empty() {
            self.0.to_mut().push_str(class);
        } else {
            self.0.to_mut().push(' ');
            self.0.to_mut().push_str(class);
        }
    }

    /// Removes a class name from this component. If the class name is not
    /// present, it will be ignored.
    pub fn remove_class(&mut self, class: &str) {
        if !self.matches(class) {
            return;
        }

        self.0 = self
            .0
            .split_ascii_whitespace()
            .filter(move |c| c != &class)
            .collect::<Vec<_>>()
            .join(" ")
            .into();
    }

    /// Replaces all class names with the given one as if a new Class component
    /// was created.
    pub fn set_class(&mut self, class: impl Into<Cow<'static, str>>) {
        self.0 = class.into();
    }
}

/// Applies a [`StyleSheetAsset`] on the entity which has this component.
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
///     commands.spawn(StyleSheet::new(asset_server.load("sheets/fancy.css")));
/// }
///
/// ```
///
#[derive(Component, Debug, Default, Reflect)]
pub struct StyleSheet {
    sheets: Vec<Handle<StyleSheetAsset>>,
}

impl StyleSheet {
    /// Creates a new [`StyleSheet`] from the given asset.
    pub fn new(handle: Handle<StyleSheetAsset>) -> Self {
        Self {
            sheets: vec![handle],
        }
    }

    /// Creates a new [`StyleSheet`] from the given assets.
    pub fn from_handles(handles: Vec<Handle<StyleSheetAsset>>) -> Self {
        Self { sheets: handles }
    }

    /// Reapplies the style sheet on entity and all children.
    pub fn refresh(&mut self) {
        // Just to trigger DerefMut
    }

    /// Internal [`StyleSheetAsset`] handle
    ///
    /// This method always assumes the style sheet has exactly one element,
    /// regardless of the true number of tracked style sheets. This method is
    /// deprecated, use [`StyleSheet::handles`] instead.
    #[deprecated(since = "0.6.0", note = "Use `handles` instead")]
    pub fn handle(&self) -> &Handle<StyleSheetAsset> {
        assert_eq!(self.sheets.len(), 1, "Use `handles` instead");
        self.sheets.first().unwrap()
    }

    /// Internal [`StyleSheetAsset`] handle
    pub fn handles(&self) -> &[Handle<StyleSheetAsset>] {
        &self.sheets
    }

    /// Change the internal [`StyleSheetAsset`] handle.
    /// This will automatically trigger the systems to reapply the style sheet.
    ///
    /// This method always assumes the style sheet has exactly one element,
    /// regardless of the true number of tracked style sheets. This method is
    /// deprecated, use [`StyleSheet::set_handles`] instead.
    #[deprecated(since = "0.6.0", note = "Use `set_handles` instead")]
    pub fn set(&mut self, handle: Handle<StyleSheetAsset>) {
        assert_eq!(self.sheets.len(), 1, "Use `set_handles` instead");
        self.sheets = vec![handle];
    }

    /// Change the internal [`StyleSheetAsset`] list of handles.
    /// This will automatically trigger the systems to reapply the style sheet.
    pub fn set_handles(&mut self, handles: Vec<Handle<StyleSheetAsset>>) {
        self.sheets = handles;
    }
}

impl PartialEq for StyleSheet {
    fn eq(&self, other: &Self) -> bool {
        self.sheets == other.sheets
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
