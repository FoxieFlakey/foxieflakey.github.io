// HTML preprocessor i made :3

use std::{borrow::Cow, collections::HashMap};

use crate::html::{self, RootElement};

mod phase2;

#[derive(Clone)]
pub enum EnvValue<'a> {
    String(Cow<'a, str>),
    Elements(Vec<html::Element<'a>>),
    Attributes(Vec<html::Attribute<'a>>),
}

pub fn process<'a>(
    tree: &mut Vec<RootElement<'a>>,
    environment: &'a HashMap<String, EnvValue<'a>>,
) -> Result<(), String> {
    phase2::run(tree, |key| environment.get(key).cloned())?;
    Ok(())
}
