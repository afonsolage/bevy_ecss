use std::hash::{Hash, Hasher};

use bevy::utils::AHasher;
use cssparser::CowRcStr;
use smallvec::{smallvec, SmallVec};

/// Represents a selector element on a style sheet rule.
/// A single selector can have multiple elements, for instance a selector of `button.enabled`
/// Would generated two elements, one for `button` and another for `.enabled`.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum SelectorElement {
    /// A name selector element, like `#score_window`. On CSS used on web, this is as known as id.
    Name(String),
    /// A component selector element, like `window` or `button`
    Component(String),
    /// A class name component selector element, `.border`
    Class(String),
    /// Indicates a parent-child relation between previous elements and next elements, like `window .border`
    Child,
    /// A keyword added to a selector that specifies a special state of the selected element(s), like `button:hover`
    PseudoClass(PseudoClassElement),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum PseudoClassElement {
    Hover,
    Unsupported,
}

impl std::fmt::Display for PseudoClassElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PseudoClassElement::Hover => write!(f, "hover"),
            PseudoClassElement::Unsupported => write!(f, "unsupported"),
        }
    }
}

impl<'a> From<&'a CowRcStr<'a>> for PseudoClassElement {
    fn from(value: &'a CowRcStr<'a>) -> Self {
        match value.as_ref() {
            "hover" => PseudoClassElement::Hover,
            _ => PseudoClassElement::Unsupported,
        }
    }
}

/// A selector parsed from a `css` rule. Each selector has a internal hash used to differentiate between many rules in the same sheet.
#[derive(Debug, Default, Clone)]
pub struct Selector {
    hash: u64,
    elements: SmallVec<[SelectorElement; 8]>,
}

impl Selector {
    /// Creates a new selector for the given elements.
    pub fn new(elements: SmallVec<[SelectorElement; 8]>) -> Self {
        let hasher = AHasher::default();

        let hasher = elements.iter().fold(hasher, |mut hasher, el| {
            el.hash(&mut hasher);
            hasher
        });

        let hash = hasher.finish();

        Self { elements, hash }
    }

    /// Builds a selector tree for this selector.
    /// Each node in the tree is composed of many elements, also each node is parent of the next one.
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
                SelectorElement::PseudoClass(c) => {
                    result.push(':');
                    result.push_str(&c.to_string());
                }
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
        Some(self.cmp(other))
    }
}

impl Ord for Selector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.hash.cmp(&other.hash) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.elements.cmp(&other.elements)
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

        for value in input.into_iter().filter(|v| !v.is_empty()) {
            if value.as_ref() == "." {
                next_is_class = true;
                continue;
            }

            if let Some(value) = value.strip_prefix('#') {
                elements.push(SelectorElement::Name(value.to_string()));
            } else if next_is_class {
                elements.push(SelectorElement::Class(value.to_string()))
            } else {
                elements.push(SelectorElement::Component(value.to_string()))
            }

            next_is_class = false;
        }

        Self::new(elements)
    }
}
