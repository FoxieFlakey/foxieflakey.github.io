// HTML preprocessor i made :3

use std::{borrow::Cow, collections::{HashMap, HashSet}, mem};

use crate::html::{self, Element, Replacer, RootElement};

pub enum EnvValue<'a> {
    String(Cow<'a, str>),
    ReplacementElement(html::Element<'a>)
}

pub fn process<'a>(
    tree: &mut Vec<RootElement<'a>>,
    environment: &HashMap<String, EnvValue<'a>>
) -> Result<(), String> {
    for child in tree.iter_mut() {
        if let RootElement::Element(elem) = child {
            process_element(elem, environment)?;
        }
    }
    Ok(())
}

fn process_element<'a>(element: &mut Element<'a>, env: &HashMap<String, EnvValue<'a>>) -> Result<(), String> {
    let mut attributes_present = HashSet::new();
    let mut normalized_attributes = Vec::with_capacity(element.attributes.len());
    
    // Normalize the attributes
    for attribute in &element.attributes {
        match attribute {
            html::Attribute::Comment(_, _) => (),
            html::Attribute::Replacer(replacer) => {
                // TODO: Handle replacer here
            },
            html::Attribute::Parsed { key, .. } => {
                if attributes_present.insert(&key.1) {
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
                process_element(&mut child, env)?;
                element.content.push(html::ElementContent::Element(child));
            },
            html::ElementContent::Replacer(replacer) => {
                match process_replacer(&replacer, env)? {
                    EnvValue::ReplacementElement(replaced) => {
                        element.content.push(html::ElementContent::Element(replaced.clone()));
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

fn process_replacer<'a, 'b>(replacer: &Replacer<'_>, env: &'b HashMap<String, EnvValue<'a>>) -> Result<&'b EnvValue<'a>, String> {
    match replacer {
        html::Replacer::Complex(_, _) => {
            todo!("Handle complex");
        }
        
        html::Replacer::Simple(_, var_name) => {
            if let Some(replacement) = env.get(var_name.as_ref()) {
                return Ok(replacement);
            } else {
                return Err(format!("Cannot find environment variable '{}'", var_name.escape_default()));
            }
        }
    }
}

