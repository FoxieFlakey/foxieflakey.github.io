use std::{borrow::Cow, mem};

use codemap::Span;
use codemap_diagnostic::{Diagnostic, Level, SpanLabel};
use either::Either;

use crate::Preprocessor;
use crate::html::util::TryInsertExt;
use crate::html::{FileContext, GeneratorArgs, Template, parser};

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum TemplateResolver {
    TemplateNameMustBeNonEmpty = 0000,
    TemplateAlreadyDefined = 0001,
    CantFindNameAttribute = 0002,
    UnknownTemplate = 0003,
    FailedExpandingGenerator = 0004,
}

impl TemplateResolver {
    pub fn description(&self) -> &'static str {
        match self {
            TemplateResolver::TemplateNameMustBeNonEmpty => "Name for template must not be empty",
            TemplateResolver::TemplateAlreadyDefined => "Template already defined",
            TemplateResolver::CantFindNameAttribute => {
                "Cannot find 'name' attribute in x-template element, there has to be one"
            }
            TemplateResolver::UnknownTemplate => "Dont know what this template",
            TemplateResolver::FailedExpandingGenerator => "Failed expanding generaator",
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

        format!("TemplateResolver:{letter}{:04}", *self as u16)
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

fn find_template_and_instances(
    context: &mut FileContext,
    input: Vec<(Span, parser::ElementContent)>,
    output: &mut Vec<(Span, parser::ElementContent)>,
) -> Result<(), Vec<Diagnostic>> {
    'outer_loop: for (element_span, content) in input {
        match content {
            parser::ElementContent::Element(ref element) => {
                let Either::Left(name) = &element.name else {
                    let mut instance = element.clone();
                    let input = mem::take(&mut instance.childs);
                    find_template_and_instances(context, input, &mut instance.childs)?;
                    output.push((
                        element_span.clone(),
                        parser::ElementContent::Element(instance),
                    ));
                    continue;
                };

                if !name.starts_with("x-") {
                    let mut instance = element.clone();
                    let input = mem::take(&mut instance.childs);
                    find_template_and_instances(context, input, &mut instance.childs)?;
                    output.push((
                        element_span.clone(),
                        parser::ElementContent::Element(instance),
                    ));
                    continue;
                }

                let attributes = &element.attributes;
                let childs = &element.childs;
                let element_name_span = element.name_span;

                if name == "x-template" {
                    let mut template_name = None;
                    for attr in attributes {
                        match attr {
                            parser::Attribute::Replacer(_, _) => {
                                // Dont know yet if this is valid x-template. So do nothing
                                // x-template dont replace anything inside it
                                output.push((element_span.clone(), content));
                                continue 'outer_loop;
                            }
                            parser::Attribute::EmptyAttribute(key_span) => {
                                return Err(vec![
                                    TemplateResolver::TemplateNameMustBeNonEmpty.to_diagnostic(&[
                                        SpanLabel {
                                            label: None,
                                            span: *key_span,
                                            style: codemap_diagnostic::SpanStyle::Primary,
                                        },
                                        SpanLabel {
                                            label: Some("This template definition".to_string()),
                                            span: element_span,
                                            style: codemap_diagnostic::SpanStyle::Secondary,
                                        },
                                    ]),
                                ]);
                            }
                            parser::Attribute::Attribute(_, data) => {
                                let key = context.resolve_span_to_string(data.key_span);
                                if key != "name" {
                                    continue;
                                }

                                let name = match &data.value.content {
                                    Either::Left(span) => html_escape::decode_html_entities(
                                        context.resolve_span_to_string(*span),
                                    ),
                                    Either::Right(v) => Cow::Borrowed(v.as_ref()),
                                };

                                template_name = Some((name.to_string(), data.value_span));
                                break;
                            }
                        }
                    }

                    let Some((name, name_span)) = template_name else {
                        return Err(vec![TemplateResolver::CantFindNameAttribute.to_diagnostic(
                            &[SpanLabel {
                                label: None,
                                span: element_span,
                                style: codemap_diagnostic::SpanStyle::Primary,
                            }],
                        )]);
                    };

                    if let Err(occupied) = TryInsertExt::try_insert(
                        &mut context.known_templates,
                        name,
                        Template::FromSource(element_name_span, Cow::Owned(childs.clone())),
                    ) {
                        match occupied.entry.get() {
                            Template::FromSource(span, _) => {
                                return Err(vec![
                                    TemplateResolver::TemplateAlreadyDefined.to_diagnostic(&[
                                        SpanLabel {
                                            label: None,
                                            span: name_span,
                                            style: codemap_diagnostic::SpanStyle::Primary,
                                        },
                                        SpanLabel {
                                            label: Some("Previous defined here".to_string()),
                                            span: *span,
                                            style: codemap_diagnostic::SpanStyle::Secondary,
                                        },
                                    ]),
                                ]);
                            }

                            Template::Generator(location, _) => {
                                return Err(vec![
                                    TemplateResolver::TemplateAlreadyDefined.to_diagnostic(&[
                                        SpanLabel {
                                            label: None,
                                            span: name_span,
                                            style: codemap_diagnostic::SpanStyle::Primary,
                                        },
                                    ]),
                                    Diagnostic {
                                        code: None,
                                        level: Level::Note,
                                        message: format!(
                                            "The template was defined by generator at {location}"
                                        ),
                                        spans: vec![],
                                    },
                                ]);
                            }
                        }
                    }
                } else {
                    // Its instantiating template
                    let Some(template) = context.known_templates.get(name) else {
                        return Err(vec![TemplateResolver::UnknownTemplate.to_diagnostic(&[
                            SpanLabel {
                                label: None,
                                span: element_name_span,
                                style: codemap_diagnostic::SpanStyle::Primary,
                            },
                        ])]);
                    };

                    expand_template(
                        context.preprocessor,
                        output,
                        element_span,
                        &childs,
                        &attributes,
                        template,
                    )?;
                }
            }
            _ => output.push((element_span, content.clone())),
        }
    }

    Ok(())
}

pub fn run(
    context: &mut FileContext,
    tree: Vec<(Span, parser::ElementContent)>,
) -> Result<Vec<(Span, parser::ElementContent)>, Vec<Diagnostic>> {
    let mut new_tree = Vec::new();

    find_template_and_instances(context, tree, &mut new_tree)?;

    // util::iter_tree_mut(&mut tree, |(element_span, element)| {
    //     match element {
    //         parser::ElementContent::Element(parser::Element {
    //             name: Either::Left(name),
    //             attributes,
    //             childs,
    //             name_span: element_name_span
    //         }) => {
    //             if !name.starts_with("x-") {
    //                 new_tree.push((element_span.clone(), element.clone()));
    //                 return true;
    //             }

    //             if name == "x-template" {
    //                 let mut template_name = None;
    //                 for attr in attributes {
    //                     match attr {
    //                         parser::Attribute::Replacer(_, _) => return true,
    //                         parser::Attribute::EmptyAttribute(key_span) => {
    //                             errs.push(
    //                                 TemplateResolver::TemplateNameMustBeNonEmpty.to_diagnostic(&[
    //                                     SpanLabel {
    //                                         label: None,
    //                                         span: *key_span,
    //                                         style: codemap_diagnostic::SpanStyle::Primary
    //                                     },
    //                                     SpanLabel {
    //                                         label: Some("This template definition".to_string()),
    //                                         span: *element_span,
    //                                         style: codemap_diagnostic::SpanStyle::Secondary
    //                                     }
    //                                 ])
    //                             );
    //                             return false;
    //                         }
    //                         parser::Attribute::Attribute(_, data) => {
    //                             let key = context.preprocessor.resolve_span_to_string(data.key_span);
    //                             if key != "name" {
    //                                 continue;
    //                             }

    //                             let name = html_escape::decode_html_entities(context.preprocessor.resolve_span_to_string(data.value.content));
    //                             template_name = Some((name.to_string(), data.value_span));
    //                             break;
    //                         }
    //                     }
    //                 }

    //                 let Some((name, name_span)) = template_name else {
    //                     errs.push(
    //                         TemplateResolver::CantFindNameAttribute.to_diagnostic(&[
    //                             SpanLabel {
    //                                 label: None,
    //                                 span: *element_span,
    //                                 style: codemap_diagnostic::SpanStyle::Primary
    //                             }
    //                         ])
    //                     );
    //                     return false;
    //                 };

    //                 if let Err(occupied) = known_templates.try_insert(name, (*element_name_span, childs.clone())) {
    //                     errs.push(
    //                         TemplateResolver::TemplateAlreadyDefined.to_diagnostic(&[
    //                             SpanLabel {
    //                                 label: None,
    //                                 span: name_span,
    //                                 style: codemap_diagnostic::SpanStyle::Primary
    //                             },
    //                             SpanLabel {
    //                                 label: Some("Previous defined here".to_string()),
    //                                 span: occupied.entry.get().0,
    //                                 style: codemap_diagnostic::SpanStyle::Secondary
    //                             }
    //                         ])
    //                     );
    //                     return false;
    //                 }
    //                 return true;
    //             } else {
    //                 // Its instantiating template
    //                 let Some((def_span, template)) = known_templates.get(name) else {
    //                     errs.push(
    //                         TemplateResolver::UnknownTemplate.to_diagnostic(&[
    //                             SpanLabel {
    //                                 label: None,
    //                                 span: *element_name_span,
    //                                 style: codemap_diagnostic::SpanStyle::Primary
    //                             }
    //                         ])
    //                     );
    //                     return false;
    //                 };

    //                 match expand_template(context, &mut new_tree, *element_span, &mem::take(childs), &mem::take(attributes), *def_span, template) {
    //                     Ok(_) => return true,
    //                     Err(e) => {
    //                         errs.extend_from_slice(&e);
    //                         return false;
    //                     }
    //                 }
    //             }
    //         }
    //         _ => {
    //             new_tree.push((*element_span, element.clone()));
    //             return true;
    //         }
    //     }
    // });

    Ok(new_tree)
}

fn expand_template(
    preprocessor: &mut Preprocessor,
    output: &mut Vec<(Span, parser::ElementContent)>,
    // Where the template expanded,
    expansion_span: Span,
    // Childs in the expansion point
    expansion_childs: &Vec<(Span, parser::ElementContent)>,
    // Attribute at expansion point
    expansion_attributes: &Vec<parser::Attribute>,
    template: &Template,
) -> Result<(), Vec<Diagnostic>> {
    let (def_span, template) = match template {
        Template::FromSource(def_span, template) => (def_span, template),
        Template::Generator(location, func) => {
            let generated_src = func(GeneratorArgs {
                attributes: expansion_attributes,
                childs: expansion_childs,
                preprocessor,
            })
            .map_err(|e| {
                return vec![Diagnostic {
                    code: Some(TemplateResolver::FailedExpandingGenerator.to_code()),
                    level: Level::Error,
                    message: format!(
                        "{}: {e}",
                        TemplateResolver::FailedExpandingGenerator.description()
                    ),
                    spans: vec![SpanLabel {
                        label: None,
                        span: expansion_span,
                        style: codemap_diagnostic::SpanStyle::Primary,
                    }],
                }];
            })?;

            let generated = preprocessor
                .add_generated_code(location, generated_src)
                .map_err(|mut diags| {
                    diags.push(Diagnostic {
                        code: None,
                        level: Level::Note,
                        message: format!(
                            "Parsing code generated at {}:{}:{}",
                            location.file(),
                            location.line(),
                            location.column()
                        ),
                        spans: vec![],
                    });
                    diags
                })?;
            output.extend_from_slice(&generated);
            return Ok(());
        }
    };

    for (element_span, element) in template.iter() {
        match element {
            parser::ElementContent::Replacer(replacer) => {
                let replacer = preprocessor.resolve_span_to_string(replacer.content);
                if replacer.starts_with("children") {
                    output.extend_from_slice(&expansion_childs);
                } else {
                    output.push((*element_span, element.clone()))
                }
            }
            parser::ElementContent::Element(template_element) => {
                // Recurse handling
                let mut instance = template_element.clone();
                instance.attributes.clear();

                // Handle the attributes
                for attribute in &template_element.attributes {
                    match attribute {
                        parser::Attribute::Replacer(_, replacer) => {
                            let replacer = preprocessor.resolve_span_to_string(replacer.content);
                            if replacer.starts_with("props") {
                                instance.attributes.extend_from_slice(expansion_attributes);
                            } else {
                                instance.attributes.push(attribute.clone());
                            }
                        }
                        _ => instance.attributes.push(attribute.clone()),
                    }
                }

                // We'll be readding it later, via nested recursion
                instance.childs.clear();

                expand_template(
                    preprocessor,
                    &mut instance.childs,
                    expansion_span,
                    &expansion_childs,
                    &expansion_attributes,
                    &mut Template::FromSource(*def_span, Cow::Borrowed(&template_element.childs)),
                )?;
                output.push((*element_span, parser::ElementContent::Element(instance)))
            }
            _ => output.push((*element_span, element.clone())),
        }
    }

    Ok(())
}
