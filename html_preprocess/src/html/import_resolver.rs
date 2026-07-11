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

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum ImportResolverCodes {
    EmptySrcAttribute = 0000,
    CannotImport = 0001
}

impl ImportResolverCodes {
    pub fn description(&self) -> &'static str {
        match self {
            ImportResolverCodes::EmptySrcAttribute => {
                "Attempting to use empty attribute like <import src /> it does not make sense. Give a path"
            },
            ImportResolverCodes::CannotImport => {
                "Cannot import file"
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

pub fn run(context: &mut FileContext, tree: Vec<(Span, parser::ElementContent)>) -> Result<Vec<(Span, parser::ElementContent)>, Vec<Diagnostic>> {
    run_impl(context, tree)
}

fn run_impl(context: &mut FileContext, tree: Vec<(Span, parser::ElementContent)>) -> Result<Vec<(Span, parser::ElementContent)>, Vec<Diagnostic>> {
    let mut new_tree = Vec::with_capacity(tree.len());
    'outer_loop: for (element_span, element) in tree {
        if let parser::ElementContent::Element(parser::Element {
            name: Either::Left(name),
            attributes,
            ..
        }) = &element
        {
            if name == &"import" {
                let attributes = attributes.iter()
                    .map(|x| {
                        match x {
                            parser::Attribute::EmptyAttribute(span) => {
                                (span.clone(), Some(context.file.source_slice(span.clone())), None)
                            }
                            
                            parser::Attribute::Attribute(span, data) => {
                                (span.clone(), Some(context.file.source_slice(data.key_span.clone())), Some(data))
                            }
                            
                            parser::Attribute::Replacer(span, _) => {
                                (span.clone(), None, None)
                            }
                        }
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
                        return Err(vec![
                            ImportResolverCodes::EmptySrcAttribute.to_diagnostic(&[
                                SpanLabel {
                                    label: None,
                                    span: attribute_span,
                                    style: SpanStyle::Primary
                                },
                                
                                SpanLabel {
                                    label: Some("Parsing this import".to_string()),
                                    span: element_span,
                                    style: SpanStyle::Secondary
                                }
                            ])
                        ]);
                    };
                    
                    // Got the path
                    let import_path = html_escape::decode_html_entities(context.file.source_slice(data.value.content));
                    let imported = context.preprocessor.load_file(&import_path)
                        .map_err(|err| {
                            vec![
                                Diagnostic {
                                    code: Some(ImportResolverCodes::CannotImport.to_code()),
                                    level: Level::Error,
                                    message: format!("{} because {}", ImportResolverCodes::CannotImport.description(), err),
                                    spans: vec![
                                        SpanLabel {
                                            label: None,
                                            span: element_span,
                                            style: SpanStyle::Primary
                                        }
                                    ]
                                }
                            ]
                        })?;
                    break;
                }
                continue;
            }
        }

        new_tree.push((element_span, element));
    }

    Ok(new_tree)
}
