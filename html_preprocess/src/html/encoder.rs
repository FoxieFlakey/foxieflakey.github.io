// Encodes HTML tree to actual html files

use std::io::{self, Write};

use codemap::Span;
use either::Either;

use crate::html::{FileContext, encoder::EncodeError::Message, parser};

pub enum EncodeError {
    Message(String),
    IO(io::Error),
}

impl From<String> for EncodeError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<io::Error> for EncodeError {
    fn from(value: io::Error) -> Self {
        Self::IO(value)
    }
}

pub fn encode<W>(
    file_context: &FileContext,
    writer: &mut W,
    tree: &Vec<(Span, parser::ElementContent)>,
) -> Result<(), EncodeError>
where
    W: Write + ?Sized,
{
    let mut stack = Vec::new();
    stack.push((None, true, tree.iter()));

    while stack.len() > 0 {
        let Some((current_span, current)) = stack.last_mut().unwrap().2.next() else {
            let (name, write_closing, _) = stack.pop().unwrap();
            // Write closing tag
            if write_closing {
                if let Some(name) = name {
                    write!(writer, "</{name}>")?;
                }
            }
            continue;
        };

        match current {
            parser::ElementContent::Element(element) => {
                let Either::Left(name) = &element.name else {
                    return Err("Encountered replacers in element name! Which encoded HTML cannot represent".to_string().into());
                };

                write!(writer, "<{name}")?;

                for attribute in &element.attributes {
                    match attribute {
                        parser::Attribute::EmptyAttribute(span) => {
                            let key = file_context.resolve_span_to_string(*span);
                            write!(writer, " {key}")?;
                        }

                        parser::Attribute::Attribute(_, data) => {
                            let key = file_context.resolve_span_to_string(data.key_span);
                            let value = match &data.value.content {
                                Either::Left(span) => file_context.resolve_span_to_string(*span),
                                Either::Right(v) => v.as_ref(),
                            };
                            write!(writer, " {key}=")?;

                            if data.value.is_double_quote {
                                write!(writer, "\"{value}\"")?;
                            } else {
                                write!(writer, "'{value}'")?;
                            }
                        }

                        parser::Attribute::Replacer(_, _) => {
                            return Err("Encountered replacers in attribute list! Which encoded HTML cannot represent".to_string().into());
                        }
                    }
                }

                // HTML's void element is different than
                // "my HTML parser's void". These are hardcoded. This
                // does not break my parser as my parser works with any
                // voids
                //
                // Source: https://developer.mozilla.org/en-US/docs/Glossary/Void_element
                let known_voids = [
                    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta",
                    "param", "source", "track", "wbr",
                ];

                if known_voids.binary_search(&name.as_str()).is_ok() {
                    write!(writer, "/>")?;
                    if element.childs.len() != 0 {
                        return Err(Message(format!(
                            "Encountered HTML void element which has children, it cannot have one"
                        )));
                    }
                } else {
                    write!(writer, ">")?;
                    stack.push((Some(name), true, element.childs.iter()));
                }
            }

            parser::ElementContent::Comment(comment) => {
                let content = file_context.resolve_span_to_string(comment.content);
                write!(writer, "<!--{content}--->")?;
            }

            parser::ElementContent::Text => {
                let content = file_context.resolve_span_to_string(*current_span);
                write!(writer, "{content}")?;
            }

            parser::ElementContent::TextReplaced(replaced_with) => {
                write!(writer, "{replaced_with}")?;
            }

            parser::ElementContent::Replacer(_) => {
                return Err("Encountered replacers! Which encoded HTML cannot represent"
                    .to_string()
                    .into());
            }
        }
    }

    Ok(())
}
