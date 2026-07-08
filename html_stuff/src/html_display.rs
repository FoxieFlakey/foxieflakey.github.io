// Displays HTML elements as tree

use std::fmt::{self, Display, Write};

use crate::{
    html::{self, Element},
    prefix_writer::PrefixWriter,
};

pub struct AsTree<'a>(pub &'a Vec<html::RootElement<'a>>);

const DOWN_RIGHT: char = '└';
const DOWN_BRANCH_RIGHT: char = '├';
const STRAIGHT_DOWN: char = '│';

impl Display for AsTree<'_> {
    fn fmt(&self, mut f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "root ({} childs)", self.0.len())?;
        for (idx, child) in self.0.iter().enumerate() {
            let header_prefix;
            let content_prefix;

            // Print the header, like name of element
            if idx == self.0.len() - 1 {
                header_prefix = DOWN_RIGHT;
                content_prefix = ' ';
            } else {
                header_prefix = DOWN_BRANCH_RIGHT;
                content_prefix = STRAIGHT_DOWN;
            }

            match child {
                html::RootElement::Element(element) => {
                    print_element(&mut f, header_prefix, content_prefix, element)?
                }

                html::RootElement::Comment(_, comment) => {
                    writeln!(f, " {header_prefix} Comment '{}'", comment)?;
                }
            }
        }
        writeln!(f, "End of root")?;

        Ok(())
    }
}

// Print the content of element (excluding the name of elements)
// its already printed by caller
fn print_element<W: Write + ?Sized>(
    f: &mut W,
    header_prefix: char,
    content_prefix: char,
    element: &Element<'_>,
) -> std::fmt::Result {
    // Print the header, like name of element
    writeln!(
        f,
        " {header_prefix} Element '{}'",
        DisplayIdentifier(&element.name)
    )?;
    let mut f = PrefixWriter::new(format!(" {content_prefix} "), f);

    writeln!(f, "Attributes: (count {})", element.attributes.len())?;
    for (idx, attribute) in element.attributes.iter().enumerate() {
        let header_prefix;

        // Print the header, like name of element
        if idx == element.attributes.len() - 1 {
            header_prefix = DOWN_RIGHT;
        } else {
            header_prefix = DOWN_BRANCH_RIGHT;
        }

        match attribute {
            html::Attribute::Parsed { key, value, .. } => match value {
                Some(value) => writeln!(
                    f,
                    " {header_prefix} '{}'='{}'",
                    key.1,
                    value.1.escape_default()
                )?,
                None => writeln!(f, " {header_prefix} '{}' is empty string", key.1)?,
            },
            html::Attribute::Replacer(replacer) => {
                writeln!(
                    f,
                    " {header_prefix} Replacer: '{}'",
                    DisplayReplacer(replacer)
                )?;
            }
            html::Attribute::Comment(_, comment) => {
                writeln!(
                    f,
                    " {header_prefix} Comment: '{}'",
                    comment.escape_default()
                )?;
            }
        }
    }

    writeln!(f, "Content: (count {})", element.content.len())?;
    for (idx, child) in element.content.iter().enumerate() {
        let header_prefix;
        let content_prefix;

        // Print the header, like name of element
        if idx == element.content.len() - 1 {
            header_prefix = DOWN_RIGHT;
            content_prefix = ' ';
        } else {
            header_prefix = DOWN_BRANCH_RIGHT;
            content_prefix = STRAIGHT_DOWN;
        }

        match child {
            html::ElementContent::Element(element) => {
                print_element(
                    &mut f as &mut dyn Write,
                    header_prefix,
                    content_prefix,
                    element,
                )?;
            }
            html::ElementContent::Comment(_, comment) => {
                writeln!(
                    f,
                    " {header_prefix} Comment: '{}'",
                    comment.escape_default()
                )?;
            }
            html::ElementContent::Text(_, text) => {
                writeln!(f, " {header_prefix} Text: '{}'", text.escape_default())?;
            }
            html::ElementContent::Replacer(replacer) => {
                writeln!(
                    f,
                    " {header_prefix} Replacer: '{}'",
                    DisplayReplacer(replacer)
                )?;
            }
        }
    }

    Ok(())
}

pub struct DisplayReplacer<'a>(pub &'a html::Replacer<'a>);

impl Display for DisplayReplacer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            html::Replacer::Complex(_, x) => write!(f, "Replacer ${{{}}}", x)?,
            html::Replacer::Simple(_, x) => write!(f, "Replacer ${}", x)?,
        }
        Ok(())
    }
}

pub struct DisplayIdentifier<'a>(pub &'a html::Identifier<'a>);

impl Display for DisplayIdentifier<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            html::Identifier::Parsed(_, text) => {
                write!(f, "{text}")?;
            }

            html::Identifier::Replacer(replacer) => {
                write!(f, "{}", DisplayReplacer(replacer))?;
            }
        }

        Ok(())
    }
}
