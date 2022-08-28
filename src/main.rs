use std::borrow::Cow;

use bevy::{
    asset::{AssetLoader, AssetServerSettings, LoadedAsset},
    ecs::system::SystemParam,
    prelude::*,
    reflect::TypeUuid,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use element::{Property, Selector, SelectorElement};
use parser::StyleRule;
use smallvec::SmallVec;

use crate::parser::parse_stylesheets;

mod colors;
mod element;
mod parser;
mod test;

trait MatchSelectorElement {
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

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .register_type::<CssClass>()
        .init_asset_loader::<StyleSheetLoader>()
        .add_asset::<StyleSheet>()
        .add_startup_system(load_style_sheet)
        .add_startup_system(test::setup)
        .add_system(apply_style_sheet)
        .run();
}

#[derive(Debug, TypeUuid)]
#[uuid = "29c5a90f-fccf-45e1-a8d9-adbfaafc0c88"]
pub struct StyleSheet(Vec<StyleRule>);

#[derive(Default)]
struct StyleSheetLoader;

impl AssetLoader for StyleSheetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let content = std::str::from_utf8(bytes)?;
            let rules = parse_stylesheets(content);
            load_context.set_default_asset(LoadedAsset::new(StyleSheet(rules)));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["css"]
    }
}

struct TestSheetRes(Handle<StyleSheet>);

fn load_style_sheet(asset_server: Res<AssetServer>, mut commands: Commands) {
    let stylesheet: Handle<StyleSheet> = asset_server.load("sheets/test.css");

    commands.insert_resource(stylesheet);
}

#[derive(SystemParam)]
struct CssQueryParam<'w, 's> {
    names: Query<'w, 's, (Entity, &'static Name)>,
    classes: Query<'w, 's, (Entity, &'static CssClass)>,
    children: Query<'w, 's, &'static Children, With<Node>>,
    style: Query<'w, 's, &'static mut Style, With<Node>>,
    text: Query<'w, 's, &'static mut Text, With<Node>>,
}

fn apply_style_sheet(
    mut assets_events: EventReader<AssetEvent<StyleSheet>>,
    sheets: Res<Assets<StyleSheet>>,
    mut css_query: CssQueryParam,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for evt in assets_events.iter() {
        match evt {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                if let Some(sheet) = sheets.get(handle) {
                    for rule in &sheet.0 {
                        let entities = select_entities(&rule.0, &css_query);

                        apply_style_properties(&entities, &mut css_query.style, &rule.1);
                        apply_text_properties(
                            &entities,
                            &mut css_query.text,
                            &rule.1,
                            &asset_server,
                        );
                        apply_color_properties(&entities, &rule.1, &mut commands);
                    }
                }
            }
            _ => (),
        }
    }
}

fn select_entities(selector: &Selector, css_query: &CssQueryParam) -> SmallVec<[Entity; 8]> {
    let mut parent_tree = selector.get_parent_tree();

    if parent_tree.is_empty() {
        return SmallVec::new();
    }

    let mut filter = None;

    loop {
        let node = parent_tree.remove(0);

        let entities = select_entities_node(node, css_query, filter.clone());

        if parent_tree.is_empty() {
            break entities;
        } else {
            let children = entities
                .into_iter()
                .filter_map(|e| css_query.children.get(e).ok())
                .map(|children| get_children_recursively(children, &css_query.children))
                .flatten()
                .collect();
            filter = Some(children);
        }
    }
}

fn select_entities_node(
    node: SmallVec<[&SelectorElement; 8]>,
    css_query: &CssQueryParam,
    filter: Option<SmallVec<[Entity; 8]>>,
) -> SmallVec<[Entity; 8]> {
    node.into_iter()
        .fold(filter, |filter, element| {
            Some(match element {
                SelectorElement::Name(name) => {
                    get_entities_with(name.as_str(), &css_query.names, filter)
                }
                SelectorElement::Class(class) => {
                    get_entities_with(class.as_str(), &css_query.classes, filter)
                }
                SelectorElement::Component(component) => {
                    debug!("Not implemented yet! ({})", component);
                    SmallVec::new()
                }
                SelectorElement::Child => unreachable!(),
            })
        })
        .unwrap_or_default()
}

fn get_entities_with<T>(
    name: &str,
    q_name: &Query<(Entity, &'static T)>,
    filter: Option<SmallVec<[Entity; 8]>>,
) -> SmallVec<[Entity; 8]>
where
    T: Component + MatchSelectorElement,
{
    q_name
        .iter()
        .filter_map(|(e, rhs)| if rhs.matches(name) { Some(e) } else { None })
        .filter(|e| {
            if let Some(filter) = &filter {
                filter.contains(e)
            } else {
                true
            }
        })
        .collect()
}

fn get_children_recursively(
    children: &Children,
    q_childs: &Query<&Children, With<Node>>,
) -> SmallVec<[Entity; 8]> {
    children
        .iter()
        .map(|&e| {
            std::iter::once(e).chain(
                q_childs
                    .get(e)
                    .map_or(SmallVec::new(), |gc| get_children_recursively(gc, q_childs)),
            )
        })
        .flatten()
        .collect()
}

fn apply_style_properties(
    entities: &SmallVec<[Entity; 8]>,
    q_styles: &mut Query<&mut Style, With<Node>>,
    properties: &[Property],
) {
    for entity in entities {
        if let Ok(mut style) = q_styles.get_mut(*entity) {
            for property in properties {
                match property {
                    Property::Display(val) => style.display = val.clone(),
                    Property::PositionType(val) => style.position_type = val.clone(),
                    Property::Direction(val) => style.direction = val.clone(),
                    Property::FlexDirection(val) => style.flex_direction = val.clone(),
                    Property::FlexWrap(val) => style.flex_wrap = val.clone(),
                    Property::AlignItems(val) => style.align_items = val.clone(),
                    Property::AlignSelf(val) => style.align_self = val.clone(),
                    Property::AlignContent(val) => style.align_content = val.clone(),
                    Property::JustifyContent(val) => style.justify_content = val.clone(),
                    Property::PositionLeft(val) => style.position.left = val.clone(),
                    Property::PositionRight(val) => style.position.right = val.clone(),
                    Property::PositionTop(val) => style.position.top = val.clone(),
                    Property::PositionBottom(val) => style.position.bottom = val.clone(),
                    Property::Margin(val) => style.margin = val.clone(),
                    Property::Padding(val) => style.padding = val.clone(),
                    Property::Border(val) => style.border = val.clone(),
                    Property::FlexGrow(val) => style.flex_grow = val.clone(),
                    Property::FlexShrink(val) => style.flex_shrink = val.clone(),
                    Property::FlexBasis(val) => style.flex_basis = val.clone(),
                    Property::SizeWidth(val) => style.size.width = val.clone(),
                    Property::SizeHeight(val) => style.size.height = val.clone(),
                    Property::SizeMinWidth(val) => style.min_size.width = val.clone(),
                    Property::SizeMinHeight(val) => style.min_size.height = val.clone(),
                    Property::SizeMaxWidth(val) => style.max_size.width = val.clone(),
                    Property::SizeMaxHeight(val) => style.max_size.height = val.clone(),
                    Property::AspectRatio(val) => style.aspect_ratio = val.clone(),
                    Property::Overflow(val) => style.overflow = val.clone(),
                    _ => (),
                }
            }
        }
    }
}

fn apply_text_properties(
    entities: &SmallVec<[Entity; 8]>,
    q_texts: &mut Query<&mut Text, With<Node>>,
    properties: &[Property],
    asset_server: &Res<AssetServer>,
) {
    for entity in entities {
        if let Ok(mut text) = q_texts.get_mut(*entity) {
            for property in properties {
                match property {
                    Property::TextVerticalAlign(val) => text.alignment.vertical = val.clone(),
                    Property::TextHorizontalAlign(val) => text.alignment.horizontal = val.clone(),
                    Property::Font(val) => text
                        .sections
                        .iter_mut()
                        .for_each(|s| s.style.font = asset_server.load(val)),
                    Property::FontSize(val) => text
                        .sections
                        .iter_mut()
                        .for_each(|s| s.style.font_size = val.clone()),
                    Property::FontColor(val) => text
                        .sections
                        .iter_mut()
                        .for_each(|s| s.style.color = (*val).into()),
                    _ => (),
                }
            }
        }
    }
}

fn apply_color_properties(
    entities: &SmallVec<[Entity; 8]>,
    properties: &[Property],
    commands: &mut Commands,
) {
    for entity in entities {
        for property in properties {
            match property {
                Property::Color(color) => commands.entity(*entity).insert(UiColor((*color).into())),
                _ => continue,
            };
        }
    }
}
