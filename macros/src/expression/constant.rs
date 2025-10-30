use std::hash::Hash;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Constant {
    pub(crate) value: f64,
}

impl Eq for Constant {}

impl PartialEq<f64> for Constant {
    fn eq(&self, other: &f64) -> bool {
        self.value == *other
    }
}

impl Hash for Constant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.to_string().hash(state)
    }
}
