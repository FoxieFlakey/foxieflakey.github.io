// Encode html from Vec<html::RootElement<'a>> to a String
// all must not contain replacers

use crate::html::{self, Identifier};

#[derive(Default)]
pub struct EncodeConfig {
    pub preserve_comment: bool,
    
    // Behaviour of this is not 100% correct for some
    // elements like <pre> but each element should say
    // that its text musn't be stripped
    pub strip_whitespace: bool
}

#[derive(Debug)]
pub struct ThereReplacer;

pub fn encode(root: &Vec<html::RootElement<'_>>, config: &EncodeConfig) -> Result<String, ThereReplacer> {
    let mut buf = String::new();
    
    for child in root {
        match child {
            html::RootElement::Element(child) => encode_element(&mut buf, child, config)?,
            html::RootElement::Comment(_, comment) => encode_comment(&mut buf, comment, config)
        }
    }
    
    Ok(buf)
}

fn encode_comment(buf: &mut String, comment: &str, config: &EncodeConfig) {
    if config.preserve_comment {
        buf.push_str("<!--");
        buf.push_str(comment);
        buf.push_str("-->");
    }
}

fn encode_element(buf: &mut String, element: &html::Element<'_>, config: &EncodeConfig) -> Result<(), ThereReplacer> {
    buf.push('<');
    buf.push_str(unwrap_identifier(&element.name)?);
    
    if element.attributes.len() > 0 {
        for attribute in element.attributes.iter() {
            buf.push(' ');
            match attribute {
                html::Attribute::Replacer(_) => return Err(ThereReplacer),
                html::Attribute::Comment(_, comment) => encode_comment(buf, comment, config),
                html::Attribute::Parsed { key, value, value_is_double_quote, .. } => {
                    buf.push_str(key);
                    buf.push('=');
                    if *value_is_double_quote {
                        buf.push('"');
                        buf.push_str(value);
                        buf.push('"');
                    } else {
                        buf.push('\'');
                        buf.push_str(value);
                        buf.push('\'');
                    }
                }
            }
        }
    }
    
    buf.push('>');
    
    for child in element.content.iter() {
        match child { 
            html::ElementContent::Replacer(_) => return Err(ThereReplacer),
            html::ElementContent::Comment(_, comment) => encode_comment(buf, comment, config),
            html::ElementContent::Element(child) => encode_element(buf, child, config)?,
            html::ElementContent::Text(_, text) => {
                if !config.strip_whitespace {
                    buf.push_str(text);
                } else {
                    for (idx, text) in text.split_whitespace().enumerate() {
                        if idx > 0 {
                            buf.push(' ');
                        }
                        buf.push_str(text);
                    }
                }
            }
        }
    }
    
    buf.push_str("</");
    buf.push_str(unwrap_identifier(&element.name)?);
    buf.push('>');
    
    Ok(())
}

fn unwrap_identifier<'a>(ident: &'a Identifier<'a>) -> Result<&'a str, ThereReplacer> {
    match ident {
        Identifier::Parsed(_, raw) => Ok(raw),
        Identifier::Replacer(_) => Err(ThereReplacer)
    }
}

