use std::borrow::Borrow;
use bevy::asset::Handle;
use bevy::prelude::*;
use bevy::utils::HashMap;
use regex::Regex;
use super::*;

pub struct LoadHandler
{
    pattern: Regex,
    handler: Box<dyn Fn(HashMap<String, String>, HashMap<String, Handle<Image>>) -> Vec<(id::Id, Handle<Image>)> + Send + Sync>,
}

impl LoadHandler
{
    pub fn new<F>(
        pattern: impl Borrow<str>,
        handle_closure: F,
    ) -> Result<Self, errors::LoadHandlerError>
    where F: Fn(HashMap<String, String>, HashMap<String, Handle<Image>>) -> Vec<(id::Id, Handle<Image>)> + Send + Sync + 'static
    {
        let regex = Regex::new(pattern.borrow())?;
        let handler = Box::new(handle_closure);
        Ok(Self {
            pattern: regex,
            handler,
        })
    }
    pub fn pattern(&self) -> &Regex {
        &self.pattern
    }

    pub fn handle(&self, captures: HashMap<String, String>, handles: HashMap<String, Handle<Image>>) -> Vec<(id::Id, Handle<Image>)> {
        (self.handler)(captures, handles)
    }
}