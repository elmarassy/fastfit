use std::hash::Hash;

#[derive(Debug, Clone, Eq)]
pub(crate) struct Variable {
    pub(crate) name: String,
    pub(crate) parameter: bool,
    pub(crate) index: usize,
}

impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.parameter == other.parameter
    }
}

impl Hash for Variable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.parameter.hash(state);
    }
}

impl Variable {
    pub(crate) fn new(name: String, parameter: bool) -> Self {
        Self {
            name,
            parameter,
            index: 0,
        }
    }
}
