use std::{collections::HashMap, mem};

use crate::{html::{self, Element, ElementContent, Identifier, RootElement}, preprocess::EnvValue};

enum FetchResult<'a, 'b> {
    Env(#[expect(unused)] EnvValue<'a>),
    Template(&'b Vec<ElementContent<'a>>)
}

pub fn run<'a, F>(tree: &mut Vec<RootElement<'a>>, mut env_fetcher: F) -> Result<(), String>
where
    F: FnMut(&str) -> Option<EnvValue<'a>>,
{
    for child in tree.iter_mut() {
        if let RootElement::Element(elem) = child {
            process_element(elem, &mut |key| {
                env_fetcher(key)
                    .map(FetchResult::Env)
            })?;
        }
    }
    Ok(())
}

fn process_element<'tree: 'template_borrow, 'template_borrow>(element: &mut Element<'tree>, env_fetcher: &mut dyn FnMut(&str) -> Option<FetchResult<'tree, 'template_borrow>>) -> Result<(), String>
{
    let mut templates = HashMap::new();
    let new_content = Vec::with_capacity(element.content.len());
    
    for mut child in mem::replace(&mut element.content, new_content) {
        match child {
            html::ElementContent::Element(ref mut child_element) => {
                if let Identifier::Parsed(_, name) = &child_element.name {
                    if name.as_ref() == "x-template" {
                        let template_name_attr = child_element.attributes.iter().find(|x| {
                                match x {
                                    html::Attribute::Comment(_, _) => false,
                                    html::Attribute::Replacer(_) => false,
                                    html::Attribute::Parsed { key, .. } => {
                                        key.1 == "name"
                                    }
                                }
                            }).ok_or("x-template must contain 'name' attribute".to_string())?;
                        let template_name = match template_name_attr {
                            html::Attribute::Parsed { value, .. } => {
                                value.as_ref().map(|x| x.1.clone())
                            },
                            _ => unreachable!()
                        }.ok_or("x-template must contain non empty 'name' attribute".to_string())?;
                        
                        // This is the x-template
                        // avoid adding it to result
                        // of phase 1
                        
                        if templates.insert(template_name.clone(), child_element.content.clone()).is_some() {
                            return Err(format!("x-template named '{}' already exist", template_name.escape_default()));
                        }
                        continue;
                    } else if name.starts_with("x-") {
                        // Its trying to concretifying a template
                        let template_content = templates.get(name)
                            .or_else(|| {
                                // Try delegate to upper level
                                match env_fetcher(name)? {
                                    FetchResult::Env(_) => None,
                                    FetchResult::Template(elem) => Some(elem)
                                }
                            })
                            .ok_or_else(|| format!("Cannot find template named '{}'", name))?;
                        
                        for child in template_content {
                            match child {
                                ElementContent::Comment(_, _) => (),
                                ElementContent::Element(elem) => {
                                    let mut cloned = elem.clone();
                                    process_concreted_template(&mut cloned, &child_element)?;
                                    element.content.push(ElementContent::Element(cloned));
                                }
                                ElementContent::Replacer(replacer) => {
                                    match replacer {
                                        html::Replacer::Complex(_, _) => return Err("Complex replacer are not supported at the moment".to_string()),
                                        html::Replacer::Simple(_, name) => {
                                            if name == "children" {
                                                // Paste children
                                                for child in &child_element.content {
                                                    let mut cloned = child.clone();
                                                    match &mut cloned {
                                                        ElementContent::Element(element) => {
                                                            process_concreted_template_attributes(element, &child_element)?;
                                                        }
                                                        _ => ()
                                                    }
                                                    element.content.push(cloned);
                                                }
                                            } else {
                                                // Passthru the replacer for next stage
                                                element.content.push(ElementContent::Replacer(replacer.clone()));
                                            }
                                        }
                                    }
                                }
                                ElementContent::Text(span, text) => {
                                    element.content.push(ElementContent::Text(span.clone(), text.clone()));
                                }
                            }
                        }
                        
                        // Don't add the instantiation to the result
                        continue;
                    } else {
                        // Proceed deeper
                        process_element(child_element, &mut |key| {
                            templates.get(key)
                                .map(FetchResult::Template)
                                .or_else(|| env_fetcher(key))
                        })?
                    }
                }
            }
            
            // Phase 1 only care x-* tags
            // for concretifying templates
            _ => ()
        }
        element.content.push(child);
    }
    
    Ok(())
}

// This one like process_concreted_template_attributes_base but it goes nest deep
fn process_concreted_template_attributes<'tree>(
    element: &mut Element<'tree>,
    template_instance_site: &Element<'tree>
) -> Result<(), String> {
    process_concreted_template_attributes_base(element, template_instance_site)?;
    
    for child in &mut element.content {
        match child {
            ElementContent::Element(elem) => {
                process_concreted_template_attributes(elem, template_instance_site)?;
            }
            _ => ()
        }
    }
    
    Ok(())
}

fn process_concreted_template_attributes_base<'tree>(
    element: &mut Element<'tree>,
    template_instance_site: &Element<'tree>
) -> Result<(), String> {
    let new_attributes = Vec::with_capacity(element.attributes.len());
    for attribute in mem::replace(&mut element.attributes, new_attributes) {
        match attribute {
            html::Attribute::Replacer(html::Replacer::Simple(_, name)) => {
                if name == "props" {
                    element.attributes.extend_from_slice(&template_instance_site.attributes);
                }
            }
            x => element.attributes.push(x),
        }
    }
    
    Ok(())
}

fn process_concreted_template<'tree>(
    element: &mut Element<'tree>,
    template_instance_site: &Element<'tree>
) -> Result<(), String> {
    process_concreted_template_attributes_base(element, template_instance_site)?;
    let new_content = Vec::with_capacity(element.content.len());
    for content in mem::replace(&mut element.content, new_content) {
        match content {
            ElementContent::Comment(_, _) => (),
            ElementContent::Element(mut elem) => {
                process_concreted_template(&mut elem, template_instance_site)?;
                element.content.push(ElementContent::Element(elem));
            }
            ElementContent::Replacer(replacer) => {
                match &replacer {
                    html::Replacer::Complex(_, _) => return Err("Complex replacer are not supported at the moment".to_string()),
                    html::Replacer::Simple(_, name) => {
                        if name == "children" {
                            // Paste children
                            for child in &template_instance_site.content {
                                element.content.push(child.clone());
                            }
                        } else {
                            // Passthru the replacer for next stage
                            element.content.push(ElementContent::Replacer(replacer));
                        }
                    }
                }
            }
            ElementContent::Text(span, text) => {
                element.content.push(ElementContent::Text(span.clone(), text.clone()));
            }
        }
    }
    Ok(())
}

