use std::{collections::HashMap, hash::Hash};

use crate::expression::Node;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Collection {
    Array(Array),
    Struct(Struct),
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Array {
    pub elements: Vec<*const Node>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Struct {
    pub name_order: Vec<String>,
    pub elements: HashMap<String, *const Node>,
}

impl Hash for Struct {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name_order.iter().for_each(|name| {
            name.hash(state);
            self.elements.get(name).hash(state);
        });
    }
}
