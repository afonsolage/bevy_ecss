use bevy::{
    ecs::{
        component::ComponentTicks,
        system::{SystemParam, SystemState},
    },
    log::{debug, error, trace},
    prelude::{
        AssetEvent, AssetId, Assets, Changed, Children, Component, Deref, DerefMut, Entity,
        EventReader, Mut, Name, Query, Res, ResMut, Resource, With, World,
    },
    ui::{Interaction, Node},
    utils::HashMap,
};
use smallvec::SmallVec;

use crate::{
    component::{Class, MatchSelectorElement, StyleSheet},
    property::{SelectedEntities, StyleSheetState, TrackedEntities},
    selector::{PseudoClassElement, Selector, SelectorElement},
    StyleSheetAsset,
};

/// Utility trait which helps to deal with dynamic components
/// Each trait is implemented for a [`SystemState<T>`] with a single `[Component]`
pub(crate) trait ComponentFilter {
    /// Query the world and returns only the which has the component.
    fn filter(&mut self, world: &World) -> SmallVec<[Entity; 8]>;

    /// Return the change ticks of the component on the given entity.
    fn get_change_ticks(&self, world: &World, entity: Entity) -> Option<ComponentTicks>;
}

impl<'w, 's, T: Component> ComponentFilter for SystemState<Query<'w, 's, Entity, With<T>>> {
    fn filter(&mut self, world: &World) -> SmallVec<[Entity; 8]> {
        self.get(world).iter().collect()
    }

    fn get_change_ticks(&self, world: &World, entity: Entity) -> Option<ComponentTicks> {
        world
            .get_entity(entity)
            .and_then(|e| e.get_change_ticks::<T>())
    }
}

/// Holds the registered [`ComponentFilter`] using the component name as key.
#[derive(Default, Resource, Deref, DerefMut)]
pub(crate) struct ComponentFilterRegistry(
    pub HashMap<&'static str, Box<dyn ComponentFilter + Send + Sync>>,
);

/// An utility [`SystemParam`] query which is used in [`prepare`] system.
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
    any: Query<'w, 's, Entity, With<Node>>,
}

/// Holds an previous prepared [`CssQueryParam`];
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

            if state.has_any_selected_entities() {
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
    css_query: CssQueryParam,
    registry: &mut ComponentFilterRegistry,
) -> StyleSheetState {
    let mut state = StyleSheetState::default();

    for (root, maybe_children, sheet_handle) in &css_query.nodes {
        for id in sheet_handle.handles().iter().map(|h| h.id()) {
            if let Some(sheet) = css_query.assets.get(id) {
                let mut tracked_entities = TrackedEntities::default();
                let mut selected_entities = SelectedEntities::default();
                debug!("Applying style {}", sheet.path());

                for rule in sheet.iter() {
                    let entities = select_entities(
                        root,
                        maybe_children,
                        &rule.selector,
                        world,
                        &css_query,
                        registry,
                        &mut tracked_entities,
                    );

                    trace!(
                        "Applying rule ({}) on {} entities",
                        rule.selector.to_string(),
                        entities.len()
                    );

                    selected_entities.push((rule.selector.clone(), entities));
                }

                selected_entities.sort_by(|(a, _), (b, _)| a.weight.cmp(&b.weight));
                state.push((id, tracked_entities, selected_entities));
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

#[derive(Debug, Default, Clone, Deref, DerefMut)]
struct FilteredEntities(SmallVec<[Entity; 8]>);

#[derive(Debug, Default, Clone, Deref, DerefMut)]
struct MatchedEntities(SmallVec<[Entity; 8]>);

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
        let (filtered, matched) = match element {
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
                get_entities_with_pseudo_class(world, *pseudo_class, entities.clone())
            }
            SelectorElement::Any => get_entities_with_any_component(&css_query.any, entities),
            // All child elements are filtered by [`get_parent_tree`](Selector::get_parent_tree)
            SelectorElement::Child => unreachable!(),
        };

        if !matched.is_empty() {
            trace!("Tracking element {:?}: {}", element, matched.len());

            tracked_entities
                .entry(element.clone())
                .or_default()
                .extend(matched.0);
        }

        filtered.0
    })
}

/// Utility function to filter any entities by using a component with implements [`MatchSelectorElement`]
/// Returns new filtered list of entities and a list of entities matched by the query.
fn get_entities_with<T>(
    name: &str,
    query: &Query<(Entity, &'static T)>,
    entities: SmallVec<[Entity; 8]>,
) -> (FilteredEntities, MatchedEntities)
where
    T: Component + MatchSelectorElement,
{
    let entities = query
        .iter()
        .filter_map(|(e, rhs)| {
            if entities.contains(&e) && rhs.matches(name) {
                Some(e)
            } else {
                None
            }
        })
        .collect::<SmallVec<_>>();

    (
        FilteredEntities(entities.clone()),
        MatchedEntities(entities),
    )
}

/// Utility function to filter any entities matching a [`PseudoClassElement`]
/// Returns new filtered list of entities and a list of entities matched by the query.
fn get_entities_with_pseudo_class(
    world: &World,
    pseudo_class: PseudoClassElement,
    entities: SmallVec<[Entity; 8]>,
) -> (FilteredEntities, MatchedEntities) {
    match pseudo_class {
        PseudoClassElement::Hover => {
            get_entities_with_pseudo_class_interaction(world, entities, &Interaction::Hovered)
        }
        PseudoClassElement::Active => {
            get_entities_with_pseudo_class_interaction(world, entities, &Interaction::Pressed)
        }
        PseudoClassElement::Unsupported => (FilteredEntities(entities), Default::default()),
    }
}

/// Utility function to filter any entities matching a [`PseudoClassElement::Hover`] or
/// [`PseudoClassElement::Active`] variant
/// This function looks for [`Interaction`] component with [`Interaction::Hovered`] or
/// [`Interaction::Pressed`] variant.
/// Returns a list with entities which are hovered or pressed and a list of entities which where matched.
fn get_entities_with_pseudo_class_interaction(
    world: &World,
    entities: SmallVec<[Entity; 8]>,
    interaction: &Interaction,
) -> (FilteredEntities, MatchedEntities) {
    let filtered = entities
        .iter()
        .copied()
        .filter(|&e| {
            world
                .get_entity(e)
                .and_then(|e| e.get::<Interaction>())
                .is_some_and(|i| i == interaction)
        })
        .collect::<SmallVec<_>>();

    (FilteredEntities(filtered), MatchedEntities(entities))
}

/// Filters entities which have the components specified on selector, like "a" or "button".
///
/// The component must be registered on [`ComponentFilterRegistry`]
fn get_entities_with_component(
    name: &str,
    world: &World,
    components: &mut ComponentFilterRegistry,
    entities: SmallVec<[Entity; 8]>,
) -> (FilteredEntities, MatchedEntities) {
    if let Some(query) = components.0.get_mut(name) {
        let filtered = query
            .filter(world)
            .into_iter()
            .filter(|e| entities.contains(e))
            .collect::<SmallVec<_>>();

        (
            FilteredEntities(filtered.clone()),
            MatchedEntities(filtered),
        )
    } else {
        error!("Unregistered component selector {}", name);
        Default::default()
    }
}

/// Filters entities which have a [`Node`] component.
/// This is to mimic the "*" selector on CSS.
fn get_entities_with_any_component(
    query: &Query<Entity, With<Node>>,
    entities: SmallVec<[Entity; 8]>,
) -> (FilteredEntities, MatchedEntities) {
    let filtered = query
        .iter()
        .filter(|e| entities.contains(e))
        .collect::<SmallVec<_>>();

    (
        FilteredEntities(filtered.clone()),
        MatchedEntities(filtered),
    )
}

/// Traverse the children hierarchy three and returns all entities.
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
                .filter(|sheet| sheet.handles().iter().any(|h| h.id() == *id))
                .for_each(|mut sheet| {
                    debug!("Refreshing sheet {:?} due to asset reload", sheet);
                    sheet.refresh();
                });
        }
    }
}

/// Clear selected entities, but keep tracked ones.
pub(crate) fn clear_state(mut sheet_rule: ResMut<StyleSheetState>) {
    if sheet_rule.has_any_selected_entities() {
        debug!("Finished applying style sheet.");
        sheet_rule.clear_selected_entities();
    }
}

/// Watch for changes on entities which is children of a Entith with [`StyleSheet`].
/// This system uses a cached list of entities which was matched by some [`SelectorElement`]
/// when applying some [`StyleSheetAsset`].
///
/// Whenever a single child has a single component changed, the entire style sheet is applied again.
pub(crate) fn watch_tracked_entities(world: &mut World) {
    if world.is_resource_changed::<StyleSheetState>() {
        trace!("StyleSheetState resource changed! Skipping watch tracked entities");
        return;
    }

    let Some(state) = world.get_resource::<StyleSheetState>() else {
        return;
    };

    let changed_assets = check_for_changed_assets(state, world);

    // This is done separated to isolate where we need &mut World.
    if !changed_assets.is_empty() {
        let mut query_state: SystemState<Query<&mut StyleSheet>> = SystemState::new(world);
        for asset_id in changed_assets {
            let mut query = query_state.get_mut(world);
            for mut stylesheet in query.iter_mut() {
                if stylesheet.handles().iter().any(|h| h.id() == asset_id) {
                    debug!("Refreshing sheet {:?} due to changed entities", stylesheet);
                    stylesheet.refresh();
                }
            }
        }
    }
}

/// Check if any entity has a component which is styled by any asset, was changed.
/// If it does, return the [`AssetId<T>`] so it can be refreshed.
fn check_for_changed_assets(
    state: &StyleSheetState,
    world: &World,
) -> Vec<AssetId<StyleSheetAsset>> {
    let mut changed_assets = vec![];
    for (asset_id, tracked_entities, _) in state.iter() {
        for (element, entities) in tracked_entities.iter() {
            if entities.is_empty() {
                continue;
            }

            let changed = match element {
                SelectorElement::Name(_) => any_component::<Name>(world, entities),
                SelectorElement::Component(c) => any_component_changed_by_name(world, entities, c),
                SelectorElement::Class(_) => any_component::<Class>(world, entities),
                SelectorElement::PseudoClass(pseudo_class) => {
                    any_component_changed_by_pseudo_class(world, entities, *pseudo_class)
                }
                SelectorElement::Any => any_component::<Node>(world, entities),
                _ => unreachable!(),
            };

            if changed {
                trace!("Changed! {:?}", element);
                changed_assets.push(*asset_id);
                break;
            }
        }
    }

    changed_assets
}

/// Checks if any entity on the given list has it's component changed.
fn any_component<T: Component>(world: &World, entities: &SmallVec<[Entity; 8]>) -> bool {
    let this_run = world.read_change_tick();
    let last_run = world.last_change_tick();
    for e in entities {
        if let Some(ticks) = world.get_entity(*e).and_then(|e| e.get_change_ticks::<T>()) {
            if ticks.is_changed(last_run, this_run) {
                return true;
            }
        }
    }
    false
}

/// Checks if any entity on the given list has it's component changed.
fn any_component_changed_by_name(
    world: &World,
    entities: &SmallVec<[Entity; 8]>,
    component_name: &str,
) -> bool {
    let this_run = world.read_change_tick();
    let last_run = world.last_change_tick();

    let Some(registry) = world.get_resource::<ComponentFilterRegistry>() else {
        return false;
    };
    let Some(boxed_state) = registry.get(component_name) else {
        return false;
    };

    for e in entities {
        if let Some(ticks) = boxed_state.get_change_ticks(world, *e) {
            if ticks.is_changed(last_run, this_run) {
                return true;
            }
        }
    }
    false
}

/// Checks if any entity on the given list has it's component changed.
fn any_component_changed_by_pseudo_class(
    world: &World,
    entities: &SmallVec<[Entity; 8]>,
    pseudo_class: PseudoClassElement,
) -> bool {
    match pseudo_class {
        PseudoClassElement::Hover | PseudoClassElement::Active => {
            any_component::<Interaction>(world, entities)
        }
        PseudoClassElement::Unsupported => false,
    }
}
