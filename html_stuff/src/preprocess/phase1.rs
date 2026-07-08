use crate::{html::RootElement, preprocess::EnvValue};

pub fn run<'a, F>(tree: &mut Vec<RootElement<'a>>, mut env_fetcher: F) -> Result<(), String>
where
    F: FnMut(&str) -> Option<EnvValue<'a>>,
{
    
    Ok(())
}
