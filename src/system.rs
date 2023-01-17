use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::{
        debug, error, trace, AssetEvent, Assets, Changed, Children, Component, Deref, DerefMut,
        Entity, EventReader, Mut, Name, Query, Res, ResMut, Resource, With, World,
    },
    ui::Node,
    utils::HashMap,
};
use smallvec::SmallVec;

use crate::{
    component::{Class, MatchSelectorElement, StyleSheet},
    property::StyleSheetState,
    selector::{Selector, SelectorElement},
    StyleSheetAsset,
};

pub(crate) trait ComponentFilter {
    fn filter(&mut self, world: &World) -> SmallVec<[Entity; 8]>;
}

impl<'w, 's, T: Component> ComponentFilter for SystemState<Query<'w, 's, Entity, With<T>>> {
    fn filter(&mut self, world: &World) -> SmallVec<[Entity; 8]> {
        self.get(world).iter().collect()
    }
}

#[derive(Default, Resource)]
pub(crate) struct ComponentFilterRegistry(
    pub HashMap<&'static str, Box<dyn ComponentFilter + Send + Sync>>,
);

#[derive(SystemParam)]
pub(crate) struct CssQueryParam<'w, 's> {
    assets: Res<'w, Assets<StyleSheetAsset>>,
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

#[derive(Deref, DerefMut, Resource)]
pub(crate) struct PrepareParams(SystemState<CssQueryParam<'static, 'static>>);

impl PrepareParams {
    pub fn new(world: &mut World) -> Self {
        Self(SystemState::new(world))
    }
}

/// Exclusive system which selects all entities and prepare the internal state used by [`Property`](crate::Property) systems.
pub(crate) fn prepare(world: &mut World) {
    world.resource_scope(|world, mut params: Mut<PrepareParams>| {
        world.resource_scope(|world, mut registry: Mut<ComponentFilterRegistry>| {
            let css_query = params.get(world);
            let state = prepare_state(world, css_query, &mut registry);

            if !state.is_empty() {
                let mut state_res = world
                    .get_resource_mut::<StyleSheetState>()
                    .expect("Should be added by plugin");

                *state_res = state;
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

                trace!(
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
        // Include root, since style sheet may be applied on root too.
        std::iter::once(root)
            .chain(get_children_recursively(children, &css_query.children).into_iter())
            .collect()
    });

    loop {
        // TODO: Rework this to use a index to avoid recreating parent_tree every time the systems runs.
        // This is has little to no impact on performance, since this system doesn't runs often.
        let node = parent_tree.remove(0);

        let entities = select_entities_node(node, world, css_query, registry, filter.clone());

        if parent_tree.is_empty() {
            break entities;
        } else {
            let children = entities
                .into_iter()
                .filter_map(|e| css_query.children.get(e).ok())
                .flat_map(|children| get_children_recursively(children, &css_query.children))
                .collect();
            filter = Some(children);
        }
    }
}

/// Filter entities matching the given selectors.
/// This function is called once per node on tree returned by [`get_parent_tree`](Selector::get_parent_tree)
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
                // All child elements are filtered by [`get_parent_tree`](Selector::get_parent_tree)
                SelectorElement::Child => unreachable!(),
            })
        })
        .unwrap_or_default()
}

/// Utility function to filter any entities by using a component with implements [`MatchSelectorElement`]
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

/// Filters entities which have the components specified on selector, like "a" or "button".
///
/// The component must be registered on [`ComponentFilterRegistry`]
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
        .flat_map(|&e| {
            std::iter::once(e).chain(
                q_childs
                    .get(e)
                    .map_or(SmallVec::new(), |gc| get_children_recursively(gc, q_childs)),
            )
        })
        .collect()
}

/// Auto reapply style sheets when hot reloading is enabled
pub(crate) fn hot_reload_style_sheets(
    mut assets_events: EventReader<AssetEvent<StyleSheetAsset>>,
    mut q_sheets: Query<&mut StyleSheet>,
) {
    for evt in assets_events.iter() {
        if let AssetEvent::Modified { handle } = evt {
            q_sheets
                .iter_mut()
                .filter(|sheet| sheet.handle() == handle)
                .for_each(|mut sheet| {
                    debug!("Refreshing sheet {:?}", sheet);
                    sheet.refresh();
                });
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
