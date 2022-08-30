use std::any::Any;

use bevy::{
    ecs::{
        query::{Fetch, WorldQuery, WorldQueryGats},
        schedule::ShouldRun,
    },
    prelude::{
        AssetServer, Assets, Commands, Deref, DerefMut, Entity, Handle, Local, Query, Res, With,
    },
    ui::{Display, Node, PositionType, Style},
    utils::HashMap,
};

use cssparser::Token;
use smallvec::SmallVec;

use crate::{parser::EcssError, selector::Selector, CssRules};

#[derive(Debug, Clone)]
pub enum PropertyToken {
    Percentage(f32),
    Dimension(f32),
    Number(f32),
    Identifier(String),
    Hash(String),
    String(String),
}

#[derive(Debug, Default, Clone, Deref)]
pub struct PropertyValues(pub SmallVec<[PropertyToken; 8]>);

impl<'i> TryFrom<Token<'i>> for PropertyToken {
    type Error = ();

    fn try_from(token: Token<'i>) -> Result<Self, Self::Error> {
        match token {
            Token::Ident(val) => Ok(Self::Identifier(val.to_string())),
            Token::Hash(val) => Ok(Self::Hash(val.to_string())),
            Token::IDHash(val) => Ok(Self::Hash(val.to_string())),
            Token::QuotedString(val) => Ok(Self::String(val.to_string())),
            Token::Number { value, .. } => Ok(Self::Number(value)),
            Token::Percentage { unit_value, .. } => Ok(Self::Percentage(unit_value)),
            Token::Dimension { value, .. } => Ok(Self::Dimension(value)),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct CachedProperties<T>(HashMap<Selector, T>);

#[derive(Debug, Default, Deref, DerefMut)]
pub struct PropertyMeta<T: Property>(HashMap<u64, CachedProperties<T::Cache>>);

impl<T: Property> PropertyMeta<T> {
    fn get_or_parse(&mut self, rules: &CssRules, selector: Selector) -> &T::Cache {
        self.entry(rules.hash())
            .or_insert(CachedProperties(Default::default()))
            .entry(selector)
            .or_insert_with_key(|selector| {
                rules
                    .get_properties(selector, T::name())
                    .map(|values| {
                        T::parse(&*values)
                            .expect("This function should be called only when there is a property")
                    })
                    .unwrap()
            })
    }
}

#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct SelectedEntities(HashMap<Selector, SmallVec<[Entity; 8]>>);

#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct StyleSheetState(HashMap<Handle<CssRules>, SelectedEntities>);

pub trait Property: Default + Sized + Send + Sync + 'static {
    type Cache: Default + Any + Send + Sync;
    type Components: WorldQuery;
    type Filters: WorldQuery;

    fn name() -> &'static str;
    fn parse<'a>(token: &[PropertyToken]) -> Result<Self::Cache, EcssError>;

    fn apply<'w>(
        cache: &Self::Cache,
        components: <<Self::Components as WorldQueryGats<'w>>::Fetch as Fetch<'w>>::Item,
        asset_server: &AssetServer,
        commands: &mut Commands,
    );

    fn have_property(
        apply_sheets: Res<StyleSheetState>,
        assets: Res<Assets<CssRules>>,
    ) -> ShouldRun {
        if apply_sheets
            .iter()
            .filter_map(|(handle, _)| assets.get(handle))
            .any(|rules| rules.has_property(Self::name()))
        {
            ShouldRun::Yes
        } else {
            ShouldRun::No
        }
    }

    fn apply_system(
        mut local: Local<PropertyMeta<Self>>,
        assets: Res<Assets<CssRules>>,
        apply_sheets: Res<StyleSheetState>,
        mut q_nodes: Query<Self::Components, Self::Filters>,
        asset_server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        for (handle, selected) in apply_sheets.iter() {
            if let Some(rules) = assets.get(handle) {
                for (selector, entities) in selected.iter() {
                    let cached = local.get_or_parse(rules, selector.clone());
                    for entity in entities {
                        if let Ok(components) = q_nodes.get_mut(*entity) {
                            Self::apply(cached, components, &asset_server, &mut commands);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct DisplayProperty;

impl Property for DisplayProperty {
    type Cache = Display;

    type Components = &'static mut Style;

    type Filters = With<Node>;

    fn name() -> &'static str {
        "display"
    }

    fn parse<'a>(_token: &[PropertyToken]) -> Result<Self::Cache, EcssError> {
        Ok(Display::Flex)
    }

    fn apply<'w>(
        cache: &Self::Cache,
        components: <<Self::Components as WorldQueryGats<'w>>::Fetch as Fetch<'w>>::Item,
        _asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        let mut style = components;
        style.display = *cache;
    }
}

#[derive(Default)]
pub(crate) struct PositionTypeProperty;

impl Property for PositionTypeProperty {
    type Cache = PositionType;

    type Components = &'static mut Style;

    type Filters = With<Node>;

    fn name() -> &'static str {
        "position-type"
    }

    fn parse<'a>(_token: &[PropertyToken]) -> Result<Self::Cache, EcssError> {
        Ok(PositionType::Relative)
    }

    fn apply<'w>(
        cache: &Self::Cache,
        components: <<Self::Components as WorldQueryGats<'w>>::Fetch as Fetch<'w>>::Item,
        _asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        let mut style = components;
        style.position_type = *cache;
    }
}
