use std::borrow::Cow;

use bevy::prelude::{Component, Deref, Handle, Name, Reflect, ReflectComponent};

use crate::CssRules;

#[derive(Debug, Reflect, Component, Default, Clone, Deref)]
#[reflect(Component)]
pub struct Class(Cow<'static, str>);

impl Class {
    pub fn new(class: impl Into<Cow<'static, str>>) -> Self {
        Self(class.into())
    }

    fn matches(&self, class: &str) -> bool {
        self.0.split_ascii_whitespace().any(|c| c == class)
    }
}

#[derive(Component, Debug, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct StyleSheet {
    sheet: Handle<CssRules>,
}

impl StyleSheet {
    pub fn new(handle: Handle<CssRules>) -> Self {
        Self { sheet: handle }
    }

    pub fn refresh(&mut self) {
        // Just to trigger DerefMut
    }

    pub fn handle(&self) -> &Handle<CssRules> {
        &self.sheet
    }

    pub fn set(&mut self, handle: Handle<CssRules>) {
        self.sheet = handle;
    }
}

impl PartialEq for StyleSheet {
    fn eq(&self, other: &Self) -> bool {
        self.sheet == other.sheet
    }
}

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
