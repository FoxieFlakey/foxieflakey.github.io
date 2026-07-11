use codemap::Span;
use codemap_diagnostic::{Diagnostic, Level, SpanLabel, SpanStyle};

use crate::html::{FileContext, parser, util};

// Resolves replacer first, excluding one that starts with props and children
// (those two are special for template resolver to handle)

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum ReplacerResolver {
    UnknownReplacer = 0000,
}

impl ReplacerResolver {
    pub fn description(&self) -> &'static str {
        match self {
            ReplacerResolver::UnknownReplacer => {
                "Unknown replacer variable"
            }
        }
    }

    pub fn level(&self) -> Level {
        Level::Error
    }

    pub fn to_code(&self) -> String {
        let letter = match self.level() {
            Level::Note => "I",
            Level::Help => "H",
            Level::Warning => "W",
            Level::Bug => "B",
            Level::Error => "E",
        };

        format!("ReplacerResolver:{letter}{:04}", *self as u16)
    }

    pub fn to_diagnostic(&self, span_labels: &[SpanLabel]) -> Diagnostic {
        Diagnostic {
            code: Some(self.to_code()),
            level: self.level(),
            message: self.description().into(),
            spans: span_labels.into(),
        }
    }
}

pub fn run(
    context: &mut FileContext,
    mut tree: Vec<(Span, parser::ElementContent)>,
) -> Result<Vec<(Span, parser::ElementContent)>, Vec<Diagnostic>> {
    let mut diags = Vec::new();
    
    util::iter_tree_mut(&mut tree, |(span, element)| {
        match element {
            parser::ElementContent::Replacer(replacer) => {
                let content = context.preprocessor.resolve_span_to_string(replacer.content);
                if content.starts_with("props") || content.starts_with("children") {
                    return true;
                }
                
                if let Some(val) = context.preprocessor.get_env(content) {
                    *element = parser::ElementContent::TextReplaced(val.clone());
                } else {
                    diags.push(ReplacerResolver::UnknownReplacer.to_diagnostic(&[
                        SpanLabel {
                            label: None,
                            span: span.clone(),
                            style: SpanStyle::Primary
                        }
                    ]));
                    return false;
                }
            },
            _ => ()
        }
        true
    });
    
    if diags.len() > 0 {
        Err(diags)
    } else {
        Ok(tree)
    }
}
