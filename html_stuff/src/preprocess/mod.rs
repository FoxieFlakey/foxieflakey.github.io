// HTML preprocessor i made :3

use std::{borrow::Cow, collections::HashMap};

use crate::html::{self, RootElement};

mod phase3;
mod phase2;

#[derive(Clone)]
pub enum EnvValue<'a> {
    String(Cow<'a, str>),
    #[expect(unused)]
    Elements(Vec<html::Element<'a>>),
    Attributes(Vec<html::Attribute<'a>>),
}

pub fn process<'a>(
    tree: &mut Vec<RootElement<'a>>,
    environment: &'a HashMap<String, EnvValue<'a>>,
) -> Result<(), String> {
    // Phase 2: resolve templates
    phase2::run(tree, |key| environment.get(key).cloned())?;
    
    // Phase 3: Replacing all remaining replacers after template
    // inserted
    phase3::run(tree, |key| environment.get(key).cloned())?;
    Ok(())
}
