use std::any::Any;

use bevy::{
    ecs::query::{QueryItem, ReadOnlyWorldQuery, WorldQuery},
    log::{error, trace},
    prelude::{
        AssetServer, Assets, Color, Commands, Deref, DerefMut, Entity, Handle, Local, Query, Res,
        Resource,
    },
    ui::{UiRect, Val},
    utils::HashMap,
};

use cssparser::Token;
use smallvec::SmallVec;

use crate::{selector::Selector, EcssError, StyleSheetAsset};

mod colors;
pub(crate) mod impls;

/// A property value token which was parsed from a CSS rule.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PropertyToken {
    /// A value which was parsed percent value, like `100%` or `73.23%`.
    Percentage(f32),
    /// A value which was parsed dimension value, like `10px` or `35em.
    ///
    /// Currently there is no distinction between [`length-values`](https://developer.mozilla.org/en-US/docs/Web/CSS/length).
    Dimension(f32),
    /// A numeric float value, like `31.1` or `43`.
    Number(f32),
    /// A plain identifier, like `none` or `center`.
    Identifier(String),
    /// A identifier prefixed by a hash, like `#001122`.
    Hash(String),
    /// A quoted string, like `"some value"`.
    String(String),
}

/// A list of [`PropertyToken`] which was parsed from a single property.
#[derive(Debug, Default, Clone, Deref)]
pub struct PropertyValues(pub(crate) SmallVec<[PropertyToken; 8]>);

impl PropertyValues {
    /// Tries to parses the current values as a single [`String`].
    pub fn string(&self) -> Option<String> {
        self.0.iter().find_map(|token| match token {
            PropertyToken::String(id) => {
                if id.is_empty() {
                    None
                } else {
                    Some(id.clone())
                }
            }
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Color`].
    ///
    /// Currently only [named colors](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color)
    /// and [hex-colors](https://developer.mozilla.org/en-US/docs/Web/CSS/hex-color) are supported.
    pub fn color(&self) -> Option<Color> {
        if self.0.len() == 1 {
            match &self.0[0] {
                PropertyToken::Identifier(name) => colors::parse_named_color(name.as_str()),
                PropertyToken::Hash(hash) => colors::parse_hex_color(hash.as_str()),
                _ => None,
            }
        } else {
            // TODO: Implement color function like rgba(255, 255, 255, 255)
            // https://developer.mozilla.org/en-US/docs/Web/CSS/color_value
            None
        }
    }

    /// Tries to parses the current values as a single identifier.
    pub fn identifier(&self) -> Option<&str> {
        self.0.iter().find_map(|token| match token {
            PropertyToken::Identifier(id) => {
                if id.is_empty() {
                    None
                } else {
                    Some(id.as_str())
                }
            }
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Val`].
    ///
    /// Only [`Percentage`](PropertyToken::Percentage) and [`Dimension`](PropertyToken::Dimension`) are considered valid values,
    /// where former is converted to [`Val::Percent`] and latter is converted to [`Val::Px`].
    pub fn val(&self) -> Option<Val> {
        self.0.iter().find_map(|token| match token {
            PropertyToken::Percentage(val) => Some(Val::Percent(*val)),
            PropertyToken::Dimension(val) => Some(Val::Px(*val)),
            PropertyToken::Identifier(val) if val == "auto" => Some(Val::Auto),
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`f32`].
    ///
    /// Only [`Percentage`](PropertyToken::Percentage), [`Dimension`](PropertyToken::Dimension`) and [`Number`](PropertyToken::Number`)
    /// are considered valid values.
    pub fn f32(&self) -> Option<f32> {
        self.0.iter().find_map(|token| match token {
            PropertyToken::Percentage(val)
            | PropertyToken::Dimension(val)
            | PropertyToken::Number(val) => Some(*val),
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Option<f32>`].
    ///
    /// This function is useful for properties where either a numeric value or a `none` value is expected.
    ///
    /// If a [`Option::None`] is returned, it means some invalid value was found.
    ///
    /// If there is a [`Percentage`](PropertyToken::Percentage), [`Dimension`](PropertyToken::Dimension`) or [`Number`](PropertyToken::Number`) token,
    /// a [`Option::Some`] with parsed [`Option<f32>`] is returned.
    /// If there is a identifier with a `none` value, then [`Option::Some`] with [`None`] is returned.
    pub fn option_f32(&self) -> Option<Option<f32>> {
        self.0.iter().find_map(|token| match token {
            PropertyToken::Percentage(val)
            | PropertyToken::Dimension(val)
            | PropertyToken::Number(val) => Some(Some(*val)),
            PropertyToken::Identifier(ident) => match ident.as_str() {
                "none" => Some(None),
                _ => None,
            },
            _ => None,
        })
    }

    /// Tries to parses the current values as a single [`Option<UiRect<Val>>`].
    ///
    /// Optional values are handled by this function, so if only one value is present it is used as `top`, `right`, `bottom` and `left`,
    /// otherwise values are applied in the following order: `top`, `right`, `bottom` and `left`.
    ///
    /// Note that it is not possible to create a [`UiRect`] with only `top` value, since it'll be understood to replicated it on all fields.
    pub fn rect(&self) -> Option<UiRect> {
        if self.0.len() == 1 {
            self.val().map(UiRect::all)
        } else {
            self.0
                .iter()
                .fold((None, 0), |(rect, idx), token| {
                    let val = match token {
                        PropertyToken::Percentage(val) => Val::Percent(*val),
                        PropertyToken::Dimension(val) => Val::Px(*val),
                        PropertyToken::Identifier(val) if val == "auto" => Val::Auto,
                        _ => return (rect, idx),
                    };
                    let mut rect: UiRect = rect.unwrap_or_default();

                    match idx {
                        0 => rect.top = val,
                        1 => rect.right = val,
                        2 => rect.bottom = val,
                        3 => rect.left = val,
                        _ => (),
                    }
                    (Some(rect), idx + 1)
                })
                .0
        }
    }
}

impl<'i> TryFrom<Token<'i>> for PropertyToken {
    type Error = ();

    fn try_from(token: Token<'i>) -> Result<Self, Self::Error> {
        match token {
            Token::Ident(val) => Ok(Self::Identifier(val.to_string())),
            Token::Hash(val) => Ok(Self::Hash(val.to_string())),
            Token::IDHash(val) => Ok(Self::Hash(val.to_string())),
            Token::QuotedString(val) => Ok(Self::String(val.to_string())),
            Token::Number { value, .. } => Ok(Self::Number(value)),
            Token::Percentage { unit_value, .. } => Ok(Self::Percentage(unit_value * 100.0)),
            Token::Dimension { value, .. } => Ok(Self::Dimension(value)),
            _ => Err(()),
        }
    }
}

/// Internal cache state. Used by [`CachedProperties`] to avoid parsing properties of the same rule on same sheet.
#[derive(Default, Debug, Clone)]
pub enum CacheState<T> {
    /// No parse was performed yet
    #[default]
    None,
    /// Parse was performed and yielded a valid value.
    Ok(T),
    /// Parse was performed but returned an error.
    Error,
}

/// Internal cache map. Used by [`PropertyMeta`] to keep track of which properties was already parsed.
#[derive(Debug, Default, Deref, DerefMut)]
pub struct CachedProperties<T>(HashMap<Selector, CacheState<T>>);

/// Internal property cache map. Used by [`Property::apply_system`] to keep track of which properties was already parsed.
#[derive(Debug, Default, Deref, DerefMut)]
pub struct PropertyMeta<T: Property>(HashMap<u64, CachedProperties<T::Cache>>);

impl<T: Property> PropertyMeta<T> {
    /// Gets a cached property value or try to parse.
    ///
    /// If there are some error while parsing, a [`CacheState::Error`] is stored to avoid trying to parse again on next try.
    fn get_or_parse(
        &mut self,
        rules: &StyleSheetAsset,
        selector: &Selector,
    ) -> &CacheState<T::Cache> {
        let cached_properties = self.entry(rules.hash()).or_default();

        // Avoid using HashMap::entry since it requires ownership of key
        if cached_properties.contains_key(selector) {
            cached_properties.get(selector).unwrap()
        } else {
            let new_cache = rules
                .get_properties(selector, T::id().name())
                .map(|values| match T::parse(values) {
                    Ok(cache) => CacheState::Ok(cache),
                    Err(err) => {
                        error!("Failed to parse property {}. Error: {}", T::id().name(), err);
                        // TODO: Clear cache state when the asset is reloaded, since values may be changed.
                        CacheState::Error
                    }
                })
                .unwrap_or(CacheState::None);

            cached_properties.insert(selector.clone(), new_cache);
            cached_properties.get(selector).unwrap()
        }
    }
}

/// Maps which entities was selected by a [`Selector`]
#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct SelectedEntities(HashMap<Selector, SmallVec<[Entity; 8]>>);

/// Maps sheets for each [`StyleSheetAsset`].
#[derive(Debug, Clone, Default, Deref, DerefMut, Resource)]
pub struct StyleSheetState(HashMap<Handle<StyleSheetAsset>, SelectedEntities>);

/// Determines how a property should interact and modify the [ecs world](`bevy::prelude::World`).
///
/// Each implementation of this trait should be registered with [`RegisterProperty`](crate::RegisterProperty) trait, where
/// will be converted into a `system` and run whenever a matched, specified by [`name()`](`Property::name()`) property is found.
///
/// These are the associated types that must by specified by implementors:
/// - [`Cache`](Property::Cache) is a cached value to be applied by this trait.
/// On the first time the `system` runs it'll call [`parse`](`Property::parse`) and cache the value.
/// Subsequential runs will only fetch the cached value.
/// - [`Components`](Property::Components) is which components will be send to [`apply`](`Property::apply`) function whenever a
/// valid cache exists and a matching property was found on any sheet rule. Check [`WorldQuery`] for more.
/// - [`Filters`](Property::Filters) is used to filter which entities will be applied the property modification.
/// Entities are first filtered by [`selectors`](`Selector`), but it can be useful to also ensure some behavior for safety reasons,
/// like only inserting [`TextAlignment`](bevy::prelude::TextAlignment) if the entity also has a [`Text`](bevy::prelude::Text) component.
///  Check [`WorldQuery`] for more.
///
/// These are tree functions required to be implemented:
/// - [`name`](Property::name) indicates which property name should matched for.
/// - [`parse`](Property::parse) parses the [`PropertyValues`] into the [`Cache`](Property::Cache) value to be reused across multiple entities.
/// - [`apply`](Property::apply) applies on the given [`Components`](Property::Components) the [`Cache`](Property::Cache) value.
/// Additionally, an [`AssetServer`] and [`Commands`] parameters are provided for more complex use cases.
///
/// Also, there one function which have default implementations:
/// - [`apply_system`](Property::apply_system) is a [`system`](https://docs.rs/bevy_ecs/0.8.1/bevy_ecs/system/index.html) which interacts with
/// [ecs world](`bevy::prelude::World`) and call the [`apply`](Property::apply) function on every matched entity.
pub trait Property: Default + Sized + Send + Sync + 'static {
    /// The cached value type to be applied by property.
    type Cache: Default + Any + Send + Sync;
    /// Which components should be queried when applying the modification. Check [`WorldQuery`] for more.
    type Components: WorldQuery;
    /// Filters conditions to be applied when querying entities by this property. Check [`ReadOnlyWorldQuery`] for more.
    type Filters: ReadOnlyWorldQuery;

    /// Indicates which property name should matched for. Must match the same property name as on `css` file.
    ///
    /// For compliance, use always `lower-case` and `kebab-case` names.
    fn id() -> lightningcss::properties::PropertyId<'static>;

    /// Parses the [`PropertyValues`] into the [`Cache`](Property::Cache) value to be reused across multiple entities.
    ///
    /// This function is called only once, on the first time a matching property is found while applying style rule.
    /// If an error is returned, it is also cached so no more attempt are made.
    fn parse(values: &PropertyValues) -> Result<Self::Cache, EcssError>;

    /// Applies on the given [`Components`](Property::Components) the [`Cache`](Property::Cache) value.
    /// Additionally, an [`AssetServer`] and [`Commands`] parameters are provided for more complex use cases.
    ///
    /// If mutability is desired while applying the changes, declare [`Components`](Property::Components) as mutable.
    fn apply(
        cache: &Self::Cache,
        components: QueryItem<Self::Components>,
        asset_server: &AssetServer,
        commands: &mut Commands,
    );

    /// The [`system`](https://docs.rs/bevy_ecs/0.8.1/bevy_ecs/system/index.html) which interacts with
    /// [ecs world](`bevy::prelude::World`) and call [`apply`](Property::apply) function on every matched entity.
    ///
    /// The default implementation will cover most use cases, by just implementing [`apply`](Property::apply)
    fn apply_system(
        mut local: Local<PropertyMeta<Self>>,
        assets: Res<Assets<StyleSheetAsset>>,
        apply_sheets: Res<StyleSheetState>,
        mut q_nodes: Query<Self::Components, Self::Filters>,
        asset_server: Res<AssetServer>,
        mut commands: Commands,
    ) {
        for (handle, selected) in apply_sheets.iter() {
            if let Some(rules) = assets.get(handle) {
                for (selector, entities) in selected.iter() {
                    if let CacheState::Ok(cached) = local.get_or_parse(rules, selector) {
                        trace!(
                            r#"Applying property "{}" from sheet "{}" ({})"#,
                            Self::id().name(),
                            rules.path(),
                            selector
                        );
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
}

