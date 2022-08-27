use std::fs;

use bevy::{prelude::*, ecs::system::SystemParam};
use element::{SelectorElement, Selector};
use parser::StyleRule;

use crate::parser::parse_stylesheets;

mod colors;
mod element;
mod parser;

#[derive(Debug, Component, Clone, Deref)]
pub struct CssClass(String);

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
    names: Query<'w, 's, &'static Name>,
    classes: Query<'w, 's, &'static Name>,
}

fn select_entities(selector: Selector, css_query: CssQueryParam) -> Vec<Entity> {
    

    todo!()
}

fn get_entities_with_name(name: &str, q_name: Query<(Entity, &Name)>) -> Vec<Entity> {
    q_name
        .iter()
        .filter_map(|(e, rhs)| if name == &**rhs { Some(e) } else { None })
        .collect()
}

fn get_entities_with_class(class: &str, q_class: Query<(Entity, &CssClass)>) -> Vec<Entity> {
    q_class
        .iter()
        .filter_map(|(e, rhs)| if class == rhs.as_str() { Some(e) } else { None })
        .collect()
}
