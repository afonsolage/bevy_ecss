use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::{
        debug, error, AssetEvent, Assets, Changed, Children, Component, Entity, EventReader, Mut,
        Name, Query, Res, ResMut, With, World,
    },
    ui::Node,
    utils::HashMap,
};
use smallvec::SmallVec;

use crate::{
    component::{Class, MatchSelectorElement, StyleSheet},
    property::StyleSheetState,
    selector::{Selector, SelectorElement},
    CssRules,
};

pub(crate) trait ComponentFilter {
    fn filter(&mut self, world: &World) -> SmallVec<[Entity; 8]>;
}

impl<'w, 's, T: Component> ComponentFilter for SystemState<Query<'w, 's, Entity, With<T>>> {
    fn filter(&mut self, world: &World) -> SmallVec<[Entity; 8]> {
        self.get(world).iter().collect()
    }
}

#[derive(Default)]
pub(crate) struct ComponentFilterRegistry(
    pub HashMap<&'static str, Box<dyn ComponentFilter + Send + Sync>>,
);

#[derive(SystemParam)]
pub(crate) struct CssQueryParam<'w, 's> {
    assets: Res<'w, Assets<CssRules>>,
    nodes: Query<
        'w,
        's,
        (Entity, Option<&'static Children>, &'static StyleSheet),
        Changed<StyleSheet>,
    >,
    names: Query<'w, 's, (Entity, &'static Name)>,
    classes: Query<'w, 's, (Entity, &'static Class)>,
    children: Query<'w, 's, &'static Children, With<Node>>,
}

pub(crate) struct PrepareState<'w, 's>(SystemState<CssQueryParam<'w, 's>>);

impl<'w, 's> PrepareState<'w, 's> {
    pub fn new(world: &mut World) -> Self {
        Self(SystemState::new(world))
    }
}

pub(crate) fn prepare(world: &mut World) {
    world.resource_scope(|world, mut state: Mut<PrepareState>| {
        world.resource_scope(|world, mut registry: Mut<ComponentFilterRegistry>| {
            let css_query = state.0.get(world);
            let state = prepare_state(world, css_query, &mut registry);
            if state.is_empty() == false {
                world.insert_resource(state);
            }
        });
    });
}

/// Prepare state to be used by [`Property`](crate::Property) systems
pub(crate) fn prepare_state(
    world: &World,
    params: CssQueryParam,
    registry: &mut ComponentFilterRegistry,
) -> StyleSheetState {
    let mut state = StyleSheetState::default();

    for (entity, children, sheet_handle) in &params.nodes {
        if let Some(sheet) = params.assets.get(sheet_handle.handle()) {
            debug!("Applying style {}", sheet.path());

            for rule in sheet.iter() {
                let entities =
                    select_entities(entity, children, &rule.selector, world, &params, registry);

                debug!(
                    "Applying rule ({}) on {} entities",
                    rule.selector.to_string(),
                    entities.len()
                );

                state
                    .entry(sheet_handle.handle().clone())
                    .or_default()
                    .insert(rule.selector.clone(), entities);
            }
        }
    }

    state
}

/// Select all entities using the given [`Selector`](crate::Selector).
///
/// If no [`Children`] is supplied, then the selector is applied only on root entity.
fn select_entities(
    root: Entity,
    children: Option<&Children>,
    selector: &Selector,
    world: &World,
    css_query: &CssQueryParam,
    registry: &mut ComponentFilterRegistry,
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

        let entities = select_entities_node(node, world, css_query, registry, filter.clone());

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
    world: &World,
    css_query: &CssQueryParam,
    registry: &mut ComponentFilterRegistry,
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
                    get_entities_with_component(component.as_str(), world, registry, filter)
                }
                SelectorElement::Child => unreachable!(),
            })
        })
        .unwrap_or_default()
}

fn get_entities_with<T>(
    name: &str,
    query: &Query<(Entity, &'static T)>,
    filter: Option<SmallVec<[Entity; 8]>>,
) -> SmallVec<[Entity; 8]>
where
    T: Component + MatchSelectorElement,
{
    query
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

fn get_entities_with_component(
    name: &str,
    world: &World,
    components: &mut ComponentFilterRegistry,
    filter: Option<SmallVec<[Entity; 8]>>,
) -> SmallVec<[Entity; 8]> {
    if let Some(query) = components.0.get_mut(name) {
        if let Some(filter) = filter {
            query
                .filter(world)
                .into_iter()
                .filter(|e| filter.contains(e))
                .collect()
        } else {
            query.filter(world)
        }
    } else {
        error!("Unregistered component selector {}", name);
        SmallVec::new()
    }
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
    if sheet_rule.len() > 0 {
        debug!("Finished applying style sheet.");
        sheet_rule.clear();
    }
}
