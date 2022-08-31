use bevy::{
    ecs::system::SystemParam,
    prelude::{
        debug, AssetEvent, Assets, Changed, Children, Component, DetectChanges, Entity,
        EventReader, Name, Query, Res, ResMut, With, warn,
    },
    ui::Node,
};
use smallvec::SmallVec;

use crate::{
    component::{Class, MatchSelectorElement, StyleSheet},
    property::StyleSheetState,
    selector::{Selector, SelectorElement},
    CssRules,
};

#[derive(SystemParam)]
pub(crate) struct CssQueryParam<'w, 's> {
    names: Query<'w, 's, (Entity, &'static Name)>,
    classes: Query<'w, 's, (Entity, &'static Class)>,
    children: Query<'w, 's, &'static Children, With<Node>>,
}

/// Auto reapply style sheets when hot reloading is enabled
pub(crate) fn hot_reload_style_sheets(
    mut assets_events: EventReader<AssetEvent<CssRules>>,
    mut q_sheets: Query<&mut StyleSheet>,
) {
    for evt in assets_events.iter() {
        match evt {
            AssetEvent::Modified { handle } => {
                q_sheets
                    .iter_mut()
                    .filter(|sheet| sheet.handle() == handle)
                    .for_each(|mut sheet| sheet.refresh());
            }
            _ => (),
        }
    }
}

/// Clear temporary state
pub(crate) fn clear_state(mut sheet_rule: ResMut<StyleSheetState>) {
    if sheet_rule.is_changed() {
        sheet_rule.clear();
    }
}

/// Prepare state to be used by [`Property`](crate::Property) systems
pub(crate) fn prepare_state(
    mut sheet_rule: ResMut<StyleSheetState>,
    sheets: Res<Assets<CssRules>>,
    q_nodes: Query<(Entity, Option<&Children>, &StyleSheet), Changed<StyleSheet>>,
    css_query: CssQueryParam,
) {
    for (entity, children, sheet_handle) in &q_nodes {
        if let Some(sheet) = sheets.get(sheet_handle.handle()) {
            for rule in sheet.iter() {
                let entities = select_entities(entity, children, &rule.selector, &css_query);

                sheet_rule
                    .entry(sheet_handle.handle().clone())
                    .or_default()
                    .insert(rule.selector.clone(), entities);
            }
        }
    }
}

/// Select all entities using the given [`Selector`](crate::Selector).
///
/// If no [`Children`] is supplied, then the selector is applied only on root entity.
fn select_entities(
    root: Entity,
    children: Option<&Children>,
    selector: &Selector,
    css_query: &CssQueryParam,
) -> SmallVec<[Entity; 8]> {
    let mut parent_tree = selector.get_parent_tree();

    if parent_tree.is_empty() {
        return SmallVec::new();
    }

    let mut filter = children.map(|children| {
        // Include root, since stylesheet may be applied on root too.
        std::iter::once(root)
            .chain(get_children_recursively(children, &css_query.children).into_iter())
            .collect()
    });

    loop {
        // Rework this to use a index to avoid recreating parent_tree every time the systems runs.
        // This is has little to no impact on performance, since this system doesn't runs often.
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
