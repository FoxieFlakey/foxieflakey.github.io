use std::mem;

use codemap::Span;
use codemap_diagnostic::{Diagnostic, Level, SpanLabel, SpanStyle};
use either::Either;

use crate::html::{
    FileContext,
    lexer::{self, parse_only_replacers},
    parser, util,
};

// Resolves replacer first, excluding one that starts with props and children
// (those two are special for template resolver to handle)

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum ReplacerResolver {
    UnknownReplacer = 0000,
    SpecialReplacerCannotBeUsed = 0001,
}

impl ReplacerResolver {
    pub fn description(&self) -> &'static str {
        match self {
            ReplacerResolver::UnknownReplacer => "Unknown replacer variable",
            ReplacerResolver::SpecialReplacerCannotBeUsed => {
                "Special replacer variable like children and props cannot be used"
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
        fn try_resolve_replacer(
            context: &mut FileContext,
            span: Span,
            replacer: &lexer::Replacer,
            is_in_attribute: bool,
        ) -> Result<Option<String>, Diagnostic> {
            let content = context.resolve_span_to_string(replacer.content);
            if is_in_attribute && content.starts_with("props") {
                return Ok(None);
            } else if !is_in_attribute && content.starts_with("children") {
                return Ok(None);
            }

            if content == "current_file" {
                return Ok(Some(context.current_file_path.clone()));
            }

            if let Some(val) = context.get_env(content) {
                return Ok(Some(val.clone()));
            } else {
                println!("{content}");
                return Err(
                    ReplacerResolver::UnknownReplacer.to_diagnostic(&[SpanLabel {
                        label: None,
                        span,
                        style: SpanStyle::Primary,
                    }]),
                );
            }
        }

        match element {
            parser::ElementContent::Element(element) => {
                if let Either::Right(replacer) = &element.name {
                    match try_resolve_replacer(context, element.name_span, replacer, false) {
                        Ok(Some(val)) => {
                            element.name = Either::Left(val);
                        }
                        Ok(None) => (),
                        Err(diag) => {
                            diags.push(diag);
                            return false;
                        }
                    }
                }

                // Replaces attributes in string, by reconstructing new one
                for attribute in mem::take(&mut element.attributes) {
                    match attribute {
                        parser::Attribute::Attribute(span, mut data) => {
                            if let Either::Left(content) = &data.value.content {
                                let file = context.find_src_file(data.value_span);
                                let portions =
                                    parse_only_replacers(file, *content).map_err(|mut x| {
                                        x.push(Diagnostic {
                                            code: None,
                                            level: Level::Error,
                                            message: "While parsing replacer in here".to_string(),
                                            spans: vec![SpanLabel {
                                                label: None,
                                                span: data.value_span,
                                                style: SpanStyle::Primary,
                                            }],
                                        });
                                        x
                                    });

                                let mut constructed = String::new();
                                match portions {
                                    Err(e) => {
                                        diags.extend_from_slice(&e);
                                        return false;
                                    }

                                    Ok(portions) => {
                                        for (span, maybe_replacer) in portions {
                                            if let Some(replacer) = maybe_replacer {
                                                match try_resolve_replacer(
                                                    context, span, &replacer, true,
                                                ) {
                                                    Ok(Some(v)) => {
                                                        constructed.push_str(&v);
                                                    }
                                                    Ok(None) => {
                                                        diags.push(ReplacerResolver::SpecialReplacerCannotBeUsed.to_diagnostic(&[
                                                            SpanLabel {
                                                                label: None,
                                                                span,
                                                                style: SpanStyle::Primary
                                                            },
                                                            SpanLabel {
                                                                label: Some("In this attribute value".to_string()),
                                                                span: data.value_span,
                                                                style: SpanStyle::Secondary
                                                            }
                                                        ]));
                                                        return false;
                                                    }
                                                    Err(e) => {
                                                        diags.push(e);
                                                        return false;
                                                    }
                                                }
                                            } else {
                                                constructed
                                                    .push_str(context.resolve_span_to_string(span));
                                            }
                                        }
                                    }
                                }
                                data.value.content = Either::Right(constructed);
                            }
                            element
                                .attributes
                                .push(parser::Attribute::Attribute(span, data))
                        }
                        x => element.attributes.push(x),
                    }
                }
            }

            parser::ElementContent::Replacer(replacer) => {
                match try_resolve_replacer(context, span.clone(), replacer, false) {
                    Ok(Some(val)) => {
                        *element = parser::ElementContent::TextReplaced(val);
                    }
                    Ok(None) => (),
                    Err(diag) => {
                        diags.push(diag);
                        return false;
                    }
                }
            }
            _ => (),
        }
        true
    });

    if diags.len() > 0 {
        Err(diags)
    } else {
        Ok(tree)
    }
}
