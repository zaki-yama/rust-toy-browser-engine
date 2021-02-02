use std::collections::HashMap;

use crate::{
    css::{Selector, SimpleSelector, Value},
    dom::{ElementData, Node},
};

/// Map from CSS property names to values.
type PropertyMap = HashMap<String, Value>;

/// A node with associated style data.
struct StyledNode<'a> {
    node: &'a Node, // pointer to a DOM node
    specified_values: PropertyMap,
    children: Vec<StyledNode<'a>>,
}

fn matches(elem: &ElementData, selector: &Selector) -> bool {
    match *selector {
        Selector::Simple(ref simple_selector) => matches_simple_selector(elem, simple_selector),
    }
}

fn matches_simple_selector(elem: &ElementData, selector: &SimpleSelector) -> bool {
    // Check type selector
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // Check ID selector
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // Check class selectors
    let elem_classes = elem.classes();
    if selector
        .class
        .iter()
        .any(|class| !elem_classes.contains(&**class))
    {
        return false;
    }

    // We didn't find any non-matching selector components.
    true
}
