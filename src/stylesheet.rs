use bevy::{reflect::TypeUuid, prelude::Deref};
use smallvec::SmallVec;

use crate::{parser::StyleSheetParser, property::Property, selector::Selector};

#[derive(Debug, TypeUuid, Deref)]
#[uuid = "14b98dd6-5425-4692-a561-5e6ae9180554"]
pub struct StyleSheet(SmallVec<[StyleRule; 8]>);

impl StyleSheet {
    pub fn parse(content: &str) -> Self {
        Self(StyleSheetParser::parse(content))
    }
}

#[derive(Debug, Clone)]
pub struct StyleRule(pub Selector, pub Vec<Property>);
