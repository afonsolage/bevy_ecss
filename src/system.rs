use bevy::{
    ecs::system::SystemParam,
    prelude::{
        debug, AssetEvent, AssetServer, Assets, Children, Commands, Component, Entity, EventReader,
        EventWriter, Name, Query, Res, With,
    },
    text::Text,
    ui::{Node, Style, UiColor},
};
use smallvec::SmallVec;

use crate::{
    component::{Class, MatchSelectorElement},
    property::Property,
    selector::{Selector, SelectorElement},
    ApplyStyleSheet, StyleSheet,
};

#[derive(SystemParam)]
pub(crate) struct CssQueryParam<'w, 's> {
    names: Query<'w, 's, (Entity, &'static Name)>,
    classes: Query<'w, 's, (Entity, &'static Class)>,
    children: Query<'w, 's, &'static Children, With<Node>>,
    style: Query<'w, 's, &'static mut Style, With<Node>>,
    text: Query<'w, 's, &'static mut Text, With<Node>>,
}

pub(crate) fn apply_loaded_style_sheets(
    mut assets_events: EventReader<AssetEvent<StyleSheet>>,
    mut apply_writer: EventWriter<ApplyStyleSheet>,
) {
    for evt in assets_events.iter() {
        match evt {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                apply_writer.send(handle.clone().into())
            }
            _ => (),
        }
    }
}

pub(crate) fn apply_style_sheet(
    mut assets_events: EventReader<ApplyStyleSheet>,
    sheets: Res<Assets<StyleSheet>>,
    mut css_query: CssQueryParam,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for ApplyStyleSheet(handle) in assets_events.iter() {
        if let Some(sheet) = sheets.get(handle) {
            for rule in sheet.iter() {
                let entities = select_entities(&rule.0, &css_query);

                apply_style_properties(&entities, &mut css_query.style, &rule.1);
                apply_text_properties(&entities, &mut css_query.text, &rule.1, &asset_server);
                apply_color_properties(&entities, &rule.1, &mut commands);
            }
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
