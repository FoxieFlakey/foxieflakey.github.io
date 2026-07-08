// HTML preprocessor i made :3

use std::{borrow::Cow, collections::{HashMap, HashSet}, mem};

use crate::html::{self, Element, Replacer, RootElement};

#[derive(Clone)]
pub enum EnvValue<'a> {
    String(Cow<'a, str>),
    Elements(Vec<html::Element<'a>>),
    Attributes(Vec<html::Attribute<'a>>)
}

pub fn process<'a>(
    tree: &mut Vec<RootElement<'a>>,
    environment: &'a HashMap<String, EnvValue<'a>>
) -> Result<(), String> {
    for child in tree.iter_mut() {
        if let RootElement::Element(elem) = child {
            process_element(elem, |key| environment.get(key).cloned())?;
        }
    }
    Ok(())
}

fn process_element<'a, F>(element: &mut Element<'a>, mut env_fetcher: F) -> Result<(), String>
    where F: FnMut(&str) -> Option<EnvValue<'a>>
{
    let mut attributes_present = HashSet::new();
    let mut normalized_attributes = Vec::with_capacity(element.attributes.len());
    
    // Normalize the attributes
    for attribute in &element.attributes {
        match attribute {
            html::Attribute::Comment(_, _) => (),
            html::Attribute::Replacer(replacer) => {
                match process_replacer(replacer, &mut env_fetcher)? {
                    EnvValue::Elements(_) => return Err("Cannot place Elements in attributes list".to_string()),
                    EnvValue::String(_) => return Err("Cannot place String in attributes list".to_string()),
                    EnvValue::Attributes(attrs) => {
                        for attribute in attrs {
                            match attribute {
                                html::Attribute::Comment(_, _) => (),
                                html::Attribute::Replacer(_) => return Err("Cannot have replacer replacing replacer".to_string()),
                                html::Attribute::Parsed { ref key, .. } => {
                                    if attributes_present.insert(key.1.clone()) {
                                        // First pair
                                        normalized_attributes.push(attribute);
                                    }
                                }
                            }
                        }
                    }
                }
            },
            html::Attribute::Parsed { key, .. } => {
                if attributes_present.insert(key.1.clone()) {
                    // First pair
                    normalized_attributes.push(attribute.clone());
                }
            }
        }
    }
    
    // Replace with normalized one
    element.attributes = normalized_attributes;
    
    // Iterate childs and update it
    let vec = Vec::with_capacity(element.content.len());
    let old_content = mem::replace(&mut element.content, vec);
    
    for child in old_content {
        match child {
            html::ElementContent::Comment(_, _) => (),
            html::ElementContent::Text(_, _) => element.content.push(child.clone()),
            html::ElementContent::Element(mut child) => {
                // Here, the env fetcher is different so can inherit
                process_element(&mut child, &mut env_fetcher)?;
                element.content.push(html::ElementContent::Element(child));
            },
            html::ElementContent::Replacer(replacer) => {
                match process_replacer(&replacer, &mut env_fetcher)? {
                    EnvValue::Elements(elements) => {
                        for replaced in elements {
                            element.content.push(html::ElementContent::Element(replaced.clone()));
                        }
                    }
                    
                    EnvValue::Attributes(_) => {
                        return Err("Attributes replacer cannot be inserted to text portion of element".to_string());
                    }
                    
                    EnvValue::String(replaced) => {
                        element.content.push(html::ElementContent::Text(html::Span {
                            start: html::Location { line: 0, column: 0, byte_offset: 0 },
                            end: html::Location { line: 0, column: 0, byte_offset: 0 },
                            source: Cow::Borrowed("<from external environment>")
                        }, replaced.clone()));
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn process_replacer<'a, F>(replacer: &Replacer<'_>, mut env_fetcher: F) -> Result<EnvValue<'a>, String>
    where F: FnMut(&str) -> Option<EnvValue<'a>>
{
    match replacer {
        html::Replacer::Complex(_, _) => {
            return Err("Complex syntax like ${...} is reserved for something, do not use".to_string());
        }
        
        html::Replacer::Simple(_, var_name) => {
            if let Some(replacement) = env_fetcher(var_name.as_ref()) {
                return Ok(replacement);
            } else {
                return Err(format!("Cannot find environment variable '{}'", var_name.escape_default()));
            }
        }
    }
}

