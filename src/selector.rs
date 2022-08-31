use std::hash::{Hash, Hasher};

use bevy::utils::AHasher;
use cssparser::CowRcStr;
use smallvec::{smallvec, SmallVec};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum SelectorElement {
    // #score_window
    Name(String),
    // window
    Component(String),
    // .border
    Class(String),
    // window .border
    Child,
}

#[derive(Debug, Default, Clone)]
pub struct Selector {
    hash: u64,
    elements: SmallVec<[SelectorElement; 8]>,
}

impl Selector {
    pub fn new(elements: SmallVec<[SelectorElement; 8]>) -> Self {
        let hasher = AHasher::default();

        let hasher = elements.iter().fold(hasher, |mut hasher, el| {
            el.hash(&mut hasher);
            hasher
        });

        let hash = hasher.finish();

        Self { elements, hash }
    }

    pub fn get_parent_tree(&self) -> SmallVec<[SmallVec<[&SelectorElement; 8]>; 8]> {
        let mut tree = SmallVec::new();
        let mut current_level = SmallVec::new();
        for element in &self.elements {
            match element {
                SelectorElement::Child => {
                    tree.push(current_level);
                    current_level = SmallVec::new();
                }
                _ => current_level.push(element),
            }
        }
        tree.push(current_level);

        tree
    }
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        for element in &self.elements {
            match element {
                SelectorElement::Name(n) => {
                    result.push('#');
                    result.push_str(n);
                }
                SelectorElement::Component(c) => result.push_str(c),
                SelectorElement::Class(c) => {
                    result.push('.');
                    result.push_str(c);
                }
                SelectorElement::Child => result.push(' '),
            }
        }

        write!(f, "{}", result)
    }
}

impl PartialEq for Selector {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Selector {}

impl PartialOrd for Selector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.hash.partial_cmp(&other.hash) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.elements.partial_cmp(&other.elements)
    }
}

impl Ord for Selector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl Hash for Selector {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl<'i> From<Vec<CowRcStr<'i>>> for Selector {
    fn from(input: Vec<CowRcStr<'i>>) -> Self {
        let mut elements = smallvec![];
        let mut next_is_class = false;

        for value in input.into_iter().filter(|v| v.is_empty() == false) {
            if value.as_ref() == "." {
                next_is_class = true;
                continue;
            } else {
                false;
            }

            if value.chars().next().unwrap() == '#' {
                elements.push(SelectorElement::Name(value[1..].to_string()));
            } else if next_is_class {
                elements.push(SelectorElement::Class(value.to_string()))
            } else {
                elements.push(SelectorElement::Component(value.to_string()))
            }
        }

        Self::new(elements)
    }
}
