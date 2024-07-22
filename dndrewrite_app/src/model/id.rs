#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Debug, Clone, Default, Deref, DerefMut)]
pub struct Id(pub(super) Vec<String>);


macro_rules! id {
    ($($val: expr),*) => {
        Id::new(&[$($val),*])
    };
}
use std::fmt::Display;
use bevy::prelude::{Deref, DerefMut};
pub(crate) use id;

impl Id {
    pub fn new(values: &[&str]) -> Self {
        Id(values.into_iter().map(|x| x.to_string()).collect())
    }

    pub fn push(&mut self, value: impl Into<String>) {
        self.0.push(value.into());
    }

    pub fn extend(&self, value: impl Into<String>) -> Self {
        let mut id = self.clone();
        id.push(value.into());
        id
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.0.get(index).map(String::as_str)
    }

    pub fn init(&self) -> Self {
        let mut id = self.clone();
        id.pop();
        id
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("/"))
    }
}