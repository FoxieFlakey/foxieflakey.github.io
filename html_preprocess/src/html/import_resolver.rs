use std::borrow::Cow;

use codemap::Span;
use codemap_diagnostic::{Diagnostic, Level, SpanLabel, SpanStyle};
use either::Either;

use crate::html::{FileContext, parser};

// Processes only the <import> tags like. The
// content is ignored (future: warning about this)
//
// <import src="..."></import>
//
// Which ofc can be void tag like
//
// <import src="..." />

// Maximum nesting imports before errors
const MAX_IMPORT_DEPTH: usize = 64;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum ImportResolverCodes {
    EmptySrcAttribute = 0000,
    CannotImport = 0001,
    ImportNestTooDeep = 0002,
}

impl ImportResolverCodes {
    pub fn description(&self) -> &'static str {
        match self {
            ImportResolverCodes::EmptySrcAttribute => {
                "Attempting to use empty attribute like <import src /> it does not make sense. Give a path"
            }
            ImportResolverCodes::CannotImport => "Cannot import file",
            ImportResolverCodes::ImportNestTooDeep => {
                "The preprocess unable to handle import that is too deeply nested"
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

        format!("ImportResolver:{letter}{:04}", *self as u16)
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
    tree: Vec<(Span, parser::ElementContent)>,
) -> Result<Vec<(Span, parser::ElementContent)>, Vec<Diagnostic>> {
    run_impl(context, tree)
}

fn run_impl(
    context: &mut FileContext,
    tree: Vec<(Span, parser::ElementContent)>,
) -> Result<Vec<(Span, parser::ElementContent)>, Vec<Diagnostic>> {
    let mut new_tree = Vec::with_capacity(tree.len());
    let mut iteration_stack = vec![tree.into_iter()];

    'outer_loop: while iteration_stack.len() > 0 {
        let Some((element_span, element)) = iteration_stack.last_mut().unwrap().next() else {
            // The top iterator already exhausted
            iteration_stack.pop();
            continue;
        };

        if let parser::ElementContent::Element(parser::Element {
            name: Either::Left(name),
            attributes,
            ..
        }) = &element
        {
            if name == &"import" {
                let attributes = attributes.iter().map(|x| match x {
                    parser::Attribute::EmptyAttribute(span) => (
                        span.clone(),
                        Some(context.resolve_span_to_string(span.clone())),
                        None,
                    ),

                    parser::Attribute::Attribute(span, data) => (
                        span.clone(),
                        Some(context.resolve_span_to_string(data.key_span.clone())),
                        Some(data),
                    ),

                    parser::Attribute::Replacer(span, _) => (span.clone(), None, None),
                });

                for (attribute_span, key, data) in attributes {
                    let Some(key) = key else {
                        // <import> element is not ready yet
                        // it still has unresolved stuffs
                        continue 'outer_loop;
                    };

                    if key != "src" {
                        // We're not interested in uninteresting attribute
                        // todo: put warning diagnostics here
                        continue;
                    }

                    let Some(data) = data else {
                        // Empty attribute, it always make no sense
                        return Err(vec![ImportResolverCodes::EmptySrcAttribute.to_diagnostic(
                            &[
                                SpanLabel {
                                    label: None,
                                    span: attribute_span,
                                    style: SpanStyle::Primary,
                                },
                                SpanLabel {
                                    label: Some("Parsing this import".to_string()),
                                    span: element_span,
                                    style: SpanStyle::Secondary,
                                },
                            ],
                        )]);
                    };

                    if iteration_stack.len() > MAX_IMPORT_DEPTH {
                        return Err(vec![Diagnostic {
                            code: Some(ImportResolverCodes::ImportNestTooDeep.to_code()),
                            level: Level::Error,
                            message: format!(
                                "{} (nested {MAX_IMPORT_DEPTH} times)",
                                ImportResolverCodes::ImportNestTooDeep.description(),
                            ),
                            spans: vec![SpanLabel {
                                label: None,
                                span: element_span,
                                style: SpanStyle::Primary,
                            }],
                        }]);
                    }

                    // Got the path
                    let import_path = match &data.value.content {
                        Either::Left(span) => {
                            html_escape::decode_html_entities(context.resolve_span_to_string(*span))
                        }
                        Either::Right(v) => Cow::Borrowed(v.as_ref()),
                    }
                    .to_string();
                    let context_diag = Diagnostic {
                        code: None,
                        level: Level::Note,
                        message: "Imported from ".to_string(),
                        spans: vec![SpanLabel {
                            label: None,
                            span: element_span,
                            style: SpanStyle::Primary,
                        }],
                    };
                    let importer = context.find_src_file(element_span).clone();
                    let imported =
                        context
                            .import_file(&importer, &import_path)
                            .map_err(|x| match x {
                                Either::Left(mut diag) => {
                                    diag.push(context_diag);
                                    diag
                                }

                                Either::Right(err) => {
                                    vec![Diagnostic {
                                        code: Some(ImportResolverCodes::CannotImport.to_code()),
                                        level: Level::Error,
                                        message: format!(
                                            "{} because {}",
                                            ImportResolverCodes::CannotImport.description(),
                                            err
                                        ),
                                        spans: vec![SpanLabel {
                                            label: None,
                                            span: element_span,
                                            style: SpanStyle::Primary,
                                        }],
                                    }]
                                }
                            })?;

                    iteration_stack.push(imported.1.clone().into_iter());
                    break;
                }
                continue;
            }
        }

        new_tree.push((element_span, element));
    }

    Ok(new_tree)
}
