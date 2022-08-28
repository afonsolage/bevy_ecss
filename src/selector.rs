use cssparser::CowRcStr;
use smallvec::SmallVec;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
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

#[derive(Debug, Clone)]
pub struct Selector(pub Vec<SelectorElement>);

impl Selector {
    pub fn get_parent_tree(&self) -> SmallVec<[SmallVec<[&SelectorElement; 8]>; 8]> {
        let mut tree = SmallVec::new();
        let mut current_level = SmallVec::new();
        for element in &self.0 {
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

        for element in &self.0 {
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

impl<'i> From<Vec<CowRcStr<'i>>> for Selector {
    fn from(input: Vec<CowRcStr<'i>>) -> Self {
        let mut elements = vec![];
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

        Selector(elements)
    }
}
