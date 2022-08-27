use std::fs;

use bevy::{ecs::system::SystemParam, prelude::*};
use element::{Selector, SelectorElement};
use parser::StyleRule;
use smallvec::{smallvec, SmallVec};

use crate::parser::parse_stylesheets;

mod colors;
mod element;
mod parser;

#[derive(Debug, Component, Clone, Deref)]
pub struct CssClass(String);

impl AsRef<str> for CssClass {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

fn main() {
    let content = fs::read_to_string("test.css").unwrap();

    let rules = parse_stylesheets(content.as_str());

    println!("Rules: {:#?}", rules);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(rules)
        .run();
}

#[derive(SystemParam)]
struct CssQueryParam<'w, 's> {
    names: Query<'w, 's, (Entity, &'static Name)>,
    classes: Query<'w, 's, (Entity, &'static CssClass)>,
}

fn select_entities(selector: &Selector, css_query: &CssQueryParam) -> SmallVec<[Entity; 8]> {
    let parent_tree = selector.get_parent_tree();

    for node in parent_tree {
        let entities = select_entities_node(node, css_query);

        // Children??
    }

    todo!();
}

fn select_entities_node(
    node: SmallVec<[&SelectorElement; 8]>,
    css_query: &CssQueryParam,
) -> SmallVec<[Entity; 8]> {
    node.into_iter()
        .fold(None, |filter, element| {
            Some(match element {
                SelectorElement::Name(name) => {
                    get_entities_with(name.as_str(), &css_query.names, filter)
                }
                SelectorElement::Class(class) => {
                    get_entities_with(class.as_str(), &css_query.classes, filter)
                }
                SelectorElement::Component(_) => todo!(),
                SelectorElement::Child => todo!(),
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
    T: Component + AsRef<str>,
{
    q_name
        .iter()
        .filter_map(|(e, rhs)| if name == rhs.as_ref() { Some(e) } else { None })
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
    q_childs: &Query<&Children, With<Parent>>,
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
