use bevy::{
    ecs::{
        component::ComponentTicks,
        system::{SystemParam, SystemState},
    },
    log::{debug, error, trace},
    prelude::{
        AssetEvent, Assets, Changed, Children, Component, Deref, DerefMut, Entity, EventReader,
        Mut, Name, Query, Res, ResMut, Resource, With, World,
    },
    ui::{Interaction, Node},
    utils::HashMap,
};
use smallvec::SmallVec;

use crate::{
    component::{Class, MatchSelectorElement, StyleSheet},
    property::{StyleSheetState, TrackedEntities},
    selector::{PseudoClassElement, Selector, SelectorElement},
    StyleSheetAsset,
};

pub(crate) trait ComponentFilter {
    fn filter(&mut self, world: &World) -> SmallVec<[Entity; 8]>;
    fn get_component_changed_ticks(&self, world: &World, entity: Entity) -> Option<ComponentTicks>;
}

impl<'w, 's, T: Component> ComponentFilter for SystemState<Query<'w, 's, Entity, With<T>>> {
    fn filter(&mut self, world: &World) -> SmallVec<[Entity; 8]> {
        self.get(world).iter().collect()
    }

    fn get_component_changed_ticks(&self, world: &World, entity: Entity) -> Option<ComponentTicks> {
        world.entity(entity).get_change_ticks::<T>()
    }
}

#[derive(Default, Resource, Deref, DerefMut)]
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

            if state.has_selected_entities() {
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
        let id = sheet_handle.handle().id();
        if let Some(sheet) = params.assets.get(id) {
            let (tracked_entities, selected_entities) = state.entry(id).or_default();
            debug!("Applying style {}", sheet.path());

            for rule in sheet.iter() {
                let entities = select_entities(
                    entity,
                    children,
                    &rule.selector,
                    world,
                    &params,
                    registry,
                    tracked_entities,
                );

                trace!(
                    "Applying rule ({}) on {} entities",
                    rule.selector.to_string(),
                    entities.len()
                );

                selected_entities.push((rule.selector.clone(), entities));
            }

            selected_entities.sort_by(|(a, _), (b, _)| a.weight.cmp(&b.weight));
        }
    }

    state
}

/// Select all entities using the given [`Selector`](crate::Selector).
///
/// If no [`Children`] is supplied, then the selector is applied only on root entity.
fn select_entities(
    root: Entity,
    maybe_children: Option<&Children>,
    selector: &Selector,
    world: &World,
    css_query: &CssQueryParam,
    registry: &mut ComponentFilterRegistry,
    tracked_entities: &mut TrackedEntities,
) -> SmallVec<[Entity; 8]> {
    let mut parent_tree = selector.get_parent_tree();

    if parent_tree.is_empty() {
        return SmallVec::new();
    }

    // Build an entity tree with all entities that may be selected.
    // This tree is composed of the entity root and all descendants entities.
    let mut entity_tree = std::iter::once(root)
        .chain(
            maybe_children
                .map(|children| get_children_recursively(children, &css_query.children))
                .unwrap_or_default(),
        )
        .collect::<SmallVec<_>>();

    loop {
        // TODO: Rework this to use a index to avoid recreating parent_tree every time the systems runs.
        // This is has little to no impact on performance, since this system doesn't runs often.
        let node = parent_tree.remove(0);

        let entities = select_entities_node(
            node,
            world,
            css_query,
            registry,
            entity_tree.clone(),
            tracked_entities,
        );

        if parent_tree.is_empty() {
            break entities;
        } else {
            entity_tree = entities
                .into_iter()
                .filter_map(|e| css_query.children.get(e).ok())
                .flat_map(|children| get_children_recursively(children, &css_query.children))
                .collect();
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
    entities: SmallVec<[Entity; 8]>,
    tracked_entities: &mut TrackedEntities,
) -> SmallVec<[Entity; 8]> {
    node.into_iter().fold(entities, |entities, element| {
        let matched_entities = match element {
            SelectorElement::Name(name) => {
                get_entities_with(name.as_str(), &css_query.names, entities)
            }
            SelectorElement::Class(class) => {
                get_entities_with(class.as_str(), &css_query.classes, entities)
            }
            SelectorElement::Component(component) => {
                get_entities_with_component(component.as_str(), world, registry, entities)
            }
            SelectorElement::PseudoClass(pseudo_class) => {
                get_entities_with_pseudo_class(world, *pseudo_class, entities)
            }
            // All child elements are filtered by [`get_parent_tree`](Selector::get_parent_tree)
            SelectorElement::Child => unreachable!(),
        };

        tracked_entities
            .entry(element.clone())
            .or_default()
            .extend(matched_entities.iter().copied());

        matched_entities
    })
}

/// Utility function to filter any entities by using a component with implements [`MatchSelectorElement`]
fn get_entities_with<T>(
    name: &str,
    query: &Query<(Entity, &'static T)>,
    entities: SmallVec<[Entity; 8]>,
) -> SmallVec<[Entity; 8]>
where
    T: Component + MatchSelectorElement,
{
    query
        .iter()
        .filter_map(|(e, rhs)| {
            if entities.contains(&e) && rhs.matches(name) {
                Some(e)
            } else {
                None
            }
        })
        .collect()
}

/// Utility function to filter any entities matching a [`PseudoClassElement`]
fn get_entities_with_pseudo_class(
    world: &World,
    pseudo_class: PseudoClassElement,
    entities: SmallVec<[Entity; 8]>,
) -> SmallVec<[Entity; 8]> {
    match pseudo_class {
        PseudoClassElement::Hover => get_entities_with_pseudo_class_hover(world, entities),
        PseudoClassElement::Unsupported => entities,
    }
}

/// Utility function to filter any entities matching a [`PseudoClassElement::Hover`] variant
///
/// This function looks for [`Interaction`] component with [`Interaction::Hovered`] variant.
fn get_entities_with_pseudo_class_hover(
    world: &World,
    entities: SmallVec<[Entity; 8]>,
) -> SmallVec<[Entity; 8]> {
    entities
        .into_iter()
        .filter(|e| {
            world
                .entity(*e)
                .get::<Interaction>()
                .is_some_and(|interaction| matches!(interaction, Interaction::Hovered))
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
    entities: SmallVec<[Entity; 8]>,
) -> SmallVec<[Entity; 8]> {
    if let Some(query) = components.0.get_mut(name) {
        query
            .filter(world)
            .into_iter()
            .filter(|e| entities.contains(e))
            .collect()
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
    for evt in assets_events.read() {
        if let AssetEvent::Modified { id } = evt {
            q_sheets
                .iter_mut()
                .filter(|sheet| sheet.handle().id() == *id)
                .for_each(|mut sheet| {
                    debug!("Refreshing sheet {:?}", sheet);
                    sheet.refresh();
                });
        }
    }
}

/// Clear temporary state
pub(crate) fn clear_state(mut sheet_rule: ResMut<StyleSheetState>) {
    if !sheet_rule.has_selected_entities() {
        debug!("Finished applying style sheet.");
        sheet_rule.clear_selected_entities();
    }
}

pub(crate) fn watch_tracked_entities(world: &mut World) {
    world.resource_scope(|world, state: Mut<StyleSheetState>| {
        let mut query_state: SystemState<Query<&mut StyleSheet>> = SystemState::new(world);

        for (asset_id, (tracked_entities, _)) in state.iter() {
            for (element, entities) in tracked_entities.iter() {
                if entities.is_empty() {
                    continue;
                }

                let changed = match element {
                    SelectorElement::Name(_) => any_changed::<Name>(world, entities),
                    SelectorElement::Component(c) => any_component_changed(world, entities, c),
                    SelectorElement::Class(_) => any_changed::<Class>(world, entities),
                    SelectorElement::PseudoClass(pseudo_class) => {
                        any_pseudo_class_changed(world, entities, *pseudo_class)
                    }
                    _ => unreachable!(),
                };

                if changed {
                    let mut query = query_state.get_mut(world);
                    for mut stylesheet in query.iter_mut() {
                        if stylesheet.handle().id() == *asset_id {
                            stylesheet.refresh();
                        }
                    }
                    break;
                }
            }
        }
    });
}

fn any_changed<T: Component>(world: &World, entities: &SmallVec<[Entity; 8]>) -> bool {
    let tick = world.last_change_tick();
    for e in entities {
        if let Some(changed_tick) = world.entity(*e).get_change_ticks::<T>() {
            if changed_tick.is_changed(changed_tick.last_changed_tick(), tick) {
                return true;
            }
        }
    }
    false
}

fn any_component_changed(
    world: &World,
    entities: &SmallVec<[Entity; 8]>,
    component_name: &str,
) -> bool {
    let Some(registry) = world.get_resource::<ComponentFilterRegistry>() else {
        return false;
    };
    let Some(boxed_state) = registry.get(component_name) else {
        return false;
    };

    let tick = world.last_change_tick();

    for e in entities {
        if let Some(changed_tick) = boxed_state.get_component_changed_ticks(world, *e) {
            if changed_tick.is_changed(changed_tick.last_changed_tick(), tick) {
                return true;
            }
        }
    }
    false
}

fn any_pseudo_class_changed(
    world: &World,
    entities: &SmallVec<[Entity; 8]>,
    pseudo_class: PseudoClassElement,
) -> bool {
    match pseudo_class {
        PseudoClassElement::Hover => any_changed::<Interaction>(world, entities),
        PseudoClassElement::Unsupported => false,
    }
}
