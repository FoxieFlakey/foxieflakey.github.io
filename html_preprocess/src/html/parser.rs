// Parser turns lexer tokens to actual elements

use std::{sync::Arc, vec::IntoIter};

use codemap::{File, Span};
use codemap_diagnostic::{Diagnostic, Level, SpanLabel, SpanStyle};
use either::Either;
use pushback_iter::PushBackIterator;

use crate::html::{lexer, util};

#[derive(Clone)]
pub struct AttributeData {
    pub value_span: Span,
    pub value: lexer::QuotedString,
    pub key_span: Span,
}

#[derive(Clone)]
pub enum Attribute {
    Attribute(Span, AttributeData),
    EmptyAttribute(Span),
    Replacer(Span, lexer::Replacer),
}

#[derive(Clone)]
pub struct Element {
    pub name_span: Span,
    pub name: Either<String, lexer::Replacer>,
    pub attributes: Vec<Attribute>,
    pub childs: Vec<(Span, ElementContent)>,
}

#[derive(Clone)]
pub enum ElementContent {
    Comment(lexer::Comment),
    Replacer(lexer::Replacer),
    Element(Element),
    Text,
    TextReplaced(String)
}

struct ParseState {
    file: Arc<File>,
    eof_span: Span,
    token_iterator: PushBackIterator<IntoIter<(Span, lexer::Token)>>,
}

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum ParseCodes {
    ExpectingElementOrTextOrcomment = 0000,
    ExpectingElementNameOrReplacerName = 0001,
    ExpectingAttributeOrReplacer = 0002,
    ExpectingGreaterThanAfterSlash = 0003,
    ExpectingEqualOrNextAttribute = 0004,
    ExpectingAttributeValue = 0005,
    ExpectingClosingTag = 0006,
    ExpectingSlashForClosingTag = 0007,
    ExpectingOptionalNameOrGreaterThan = 0008,
    ReplacerUnsupportedInClosingTag = 0009,
    MismatchedNameBetweenOpeningAndClosing = 0010,
    MismatchedNameBetweenOpeningAndClosingWithReplacer = 0011,
    ExpectingGreaterThanForClosing = 0012,
    ExpectingChildOrClosingTag = 0013,
    ExpectingIdentiferForAttributeName = 0014,
}

impl ParseCodes {
    pub fn description(&self) -> &'static str {
        match self {
            ParseCodes::ExpectingElementOrTextOrcomment => {
                "Expecting element or identifer or comment"
            }
            ParseCodes::ExpectingElementNameOrReplacerName => {
                "Expecting name of element or replacer for the name"
            }
            ParseCodes::ExpectingAttributeOrReplacer => "Expecting attribute list or replacer",
            ParseCodes::ExpectingGreaterThanAfterSlash => {
                "Expecting '>' after '/' for void element"
            }
            ParseCodes::ExpectingEqualOrNextAttribute => "Expecting next attribute or equal sign",
            ParseCodes::ExpectingAttributeValue => "Expecting attribute value",
            ParseCodes::ExpectingClosingTag => "Expecting close tag either </> or </name>",
            ParseCodes::ExpectingSlashForClosingTag => "Expecting '/' after '<' for closing tag",
            ParseCodes::ExpectingOptionalNameOrGreaterThan => {
                "Expecting '>' or name for closing tag to finish either </> or </name>"
            }
            ParseCodes::ReplacerUnsupportedInClosingTag => {
                "Using replacer in closing tag is unsupported please use </> instead"
            }
            ParseCodes::MismatchedNameBetweenOpeningAndClosing => {
                "Opening and closing has mismatched tag name"
            }
            ParseCodes::MismatchedNameBetweenOpeningAndClosingWithReplacer => {
                "Opening with replacer will never match with closing (closing has to be </>)"
            }
            ParseCodes::ExpectingGreaterThanForClosing => {
                "Expecting '>' after content of closing tag"
            }
            ParseCodes::ExpectingChildOrClosingTag => {
                "Expecting child element, closing tag or text"
            }
            ParseCodes::ExpectingIdentiferForAttributeName => {
                "Expecting identifier for attribute name"
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

        format!("Parser:{letter}{:04}", *self as u16)
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

impl ParseState {
    fn parse_one(&mut self) -> Result<(Span, ElementContent), Vec<Diagnostic>> {
        let start;
        match self.token_iterator.next() {
            Some((span, lexer::Token::Comment(comment))) => {
                return Ok((span, ElementContent::Comment(comment)));
            }
            Some((span, lexer::Token::Text)) => return Ok((span, ElementContent::Text)),

            Some((span, lexer::Token::Replacer(replacer))) => {
                return Ok((span, ElementContent::Replacer(replacer)));
            }

            // Let match done we're parsing element now
            Some((span, lexer::Token::LessThan)) => {
                start = span.low() - self.file.span.low();
            }

            Some((span, _)) => {
                return Err(vec![
                    ParseCodes::ExpectingElementOrTextOrcomment.to_diagnostic(&[SpanLabel {
                        label: None,
                        span,
                        style: SpanStyle::Primary,
                    }]),
                ]);
            }
            None => {
                return Err(vec![
                    ParseCodes::ExpectingElementOrTextOrcomment.to_diagnostic(&[SpanLabel {
                        label: None,
                        span: self.eof_span.clone(),
                        style: SpanStyle::Primary,
                    }]),
                ]);
            }
        }

        let name;
        let name_span;

        // part 1: find the name
        match self.token_iterator.next() {
            Some((span, lexer::Token::Replacer(replacer))) => {
                name = Either::Right(replacer);
                name_span = span;
            }
            Some((span, lexer::Token::Identifier)) => {
                name = Either::Left(self.file.source_slice(span).to_string());
                name_span = span;
            }
            Some((span, _)) => {
                return Err(vec![
                    ParseCodes::ExpectingElementNameOrReplacerName.to_diagnostic(&[SpanLabel {
                        label: None,
                        span,
                        style: SpanStyle::Primary,
                    }]),
                ]);
            }
            None => {
                return Err(vec![
                    ParseCodes::ExpectingElementNameOrReplacerName.to_diagnostic(&[SpanLabel {
                        label: None,
                        span: self.eof_span.clone(),
                        style: SpanStyle::Primary,
                    }]),
                ]);
            }
        }

        let mut this = Element {
            attributes: Vec::new(),
            childs: Vec::new(),
            name,
            name_span,
        };

        // part 2: parse attributes
        let tag_open_end;
        loop {
            let (span, token) = self.token_iterator.next().ok_or_else(|| {
                vec![
                    ParseCodes::ExpectingAttributeOrReplacer.to_diagnostic(&[SpanLabel {
                        label: None,
                        span: self.eof_span.clone(),
                        style: SpanStyle::Primary,
                    }]),
                ]
            })?;

            match token {
                lexer::Token::GreaterThan => {
                    // End of opening tag
                    tag_open_end = span.high() - self.file.span.low();
                    break;
                }

                lexer::Token::Slash => {
                    // Void element
                    // finish parsing and return
                    match self.token_iterator.next() {
                        Some((span, lexer::Token::GreaterThan)) => {
                            return Ok((
                                self.file
                                    .span
                                    .subspan(start, span.high() - self.file.span.low()),
                                ElementContent::Element(this),
                            ));
                        }
                        Some((span, _)) => {
                            return Err(vec![
                                ParseCodes::ExpectingGreaterThanAfterSlash.to_diagnostic(&[
                                    SpanLabel {
                                        label: None,
                                        span: util::one_char_span(
                                            &self.file,
                                            span.low() - self.file.span.low(),
                                        ),
                                        style: SpanStyle::Primary,
                                    },
                                ]),
                            ]);
                        }
                        None => {
                            return Err(vec![
                                ParseCodes::ExpectingGreaterThanAfterSlash.to_diagnostic(&[
                                    SpanLabel {
                                        label: None,
                                        span: self.eof_span.clone(),
                                        style: SpanStyle::Primary,
                                    },
                                ]),
                            ]);
                        }
                    }
                }

                // Replacer attribute
                lexer::Token::Replacer(replacer) => {
                    this.attributes.push(Attribute::Replacer(span, replacer));
                }

                // Key/value attribute pair
                lexer::Token::Identifier => {
                    self.token_iterator
                        .push_back((span.clone(), lexer::Token::Identifier));
                    this.attributes.push(self.parse_attribute()?);
                }

                _ => {
                    return Err(vec![
                        ParseCodes::ExpectingElementNameOrReplacerName.to_diagnostic(&[
                            SpanLabel {
                                label: None,
                                span: util::one_char_span(
                                    &self.file,
                                    span.low() - self.file.span.low(),
                                ),
                                style: SpanStyle::Primary,
                            },
                        ]),
                    ]);
                }
            }
        }

        // part 3: parse child
        loop {
            // Check if its </ or not
            let (span, token) = self.token_iterator.next().ok_or_else(|| {
                vec![ParseCodes::ExpectingChildOrClosingTag.to_diagnostic(&[
                    SpanLabel {
                        label: None,
                        span: self.eof_span.clone(),
                        style: SpanStyle::Primary,
                    },
                    SpanLabel {
                        label: Some("Parsing child of this".to_string()),
                        span: this.name_span,
                        style: SpanStyle::Secondary,
                    },
                ])]
            })?;

            let (_, peeked) = self.token_iterator.peek().ok_or_else(|| {
                vec![ParseCodes::ExpectingChildOrClosingTag.to_diagnostic(&[
                    SpanLabel {
                        label: None,
                        span: self.eof_span.clone(),
                        style: SpanStyle::Primary,
                    },
                    SpanLabel {
                        label: Some("Parsing child of this".to_string()),
                        span: this.name_span,
                        style: SpanStyle::Secondary,
                    },
                ])]
            })?;

            match (&token, peeked) {
                (lexer::Token::LessThan, lexer::Token::Slash) => {
                    // Put the '<' back
                    self.token_iterator.push_back((span, token));
                    break;
                }

                (_, _) => (),
            }

            self.token_iterator.push_back((span, token));

            let result = self.parse_one().map_err(|mut x| {
                x.push(Diagnostic {
                    code: None,
                    level: Level::Note,
                    message: "Parsing child of this".to_string(),
                    spans: vec![SpanLabel {
                        label: None,
                        span: self.file.span.subspan(start, tag_open_end),
                        style: SpanStyle::Primary,
                    }],
                });
                x
            })?;
            this.childs.push(result);
        }

        // part 4: parse the closing </> or </name>
        let end = self.parse_closing_tag(&this)?;

        Ok((
            self.file.span.subspan(start, end),
            ElementContent::Element(this),
        ))
    }

    fn parse_attribute(&mut self) -> Result<Attribute, Vec<Diagnostic>> {
        let key_span;
        // Check identifier for the key portion
        match self.token_iterator.next() {
            Some((span, lexer::Token::Identifier)) => {
                key_span = span;
            }

            Some((span, _)) => {
                return Err(vec![
                    ParseCodes::ExpectingIdentiferForAttributeName.to_diagnostic(&[SpanLabel {
                        label: None,
                        span,
                        style: SpanStyle::Primary,
                    }]),
                ]);
            }

            None => {
                return Err(vec![
                    ParseCodes::ExpectingIdentiferForAttributeName.to_diagnostic(&[SpanLabel {
                        label: None,
                        span: self.eof_span.clone(),
                        style: SpanStyle::Primary,
                    }]),
                ]);
            }
        }

        // Check if there '=', i.e. non empty attribute
        match self.token_iterator.peek() {
            Some((_, lexer::Token::Equal)) => {
                self.token_iterator.next().unwrap();

                // Key value attribute
                match self.token_iterator.next() {
                    Some((span, lexer::Token::QuotedString(x))) => {
                        let full_span = self.file.span.subspan(
                            key_span.low() - self.file.span.low(),
                            span.low() - self.file.span.low(),
                        );

                        return Ok(Attribute::Attribute(
                            full_span,
                            AttributeData {
                                key_span,
                                value_span: span,
                                value: x,
                            },
                        ));
                    }
                    Some((span, _)) => {
                        return Err(vec![ParseCodes::ExpectingAttributeValue.to_diagnostic(&[
                            SpanLabel {
                                label: None,
                                span: util::one_char_span(
                                    &self.file,
                                    span.low() - self.file.span.low(),
                                ),
                                style: SpanStyle::Primary,
                            },
                        ])]);
                    }
                    None => {
                        return Err(vec![ParseCodes::ExpectingAttributeValue.to_diagnostic(&[
                            SpanLabel {
                                label: None,
                                span: self.eof_span.clone(),
                                style: SpanStyle::Primary,
                            },
                        ])]);
                    }
                }
            }
            Some(_) => {
                // Empty attribute
                return Ok(Attribute::EmptyAttribute(key_span));
            }
            None => {
                return Err(vec![
                    ParseCodes::ExpectingEqualOrNextAttribute.to_diagnostic(&[SpanLabel {
                        label: None,
                        span: self.eof_span.clone(),
                        style: SpanStyle::Primary,
                    }]),
                ]);
            }
        }
    }

    // return the last offset, after the closing
    fn parse_closing_tag(&mut self, this: &Element) -> Result<u64, Vec<Diagnostic>> {
        match self.token_iterator.next() {
            Some((_, lexer::Token::LessThan)) => {
                match self.token_iterator.next() {
                    Some((_, lexer::Token::Slash)) => {
                        // Optionally a name or immediately '>'
                        match self.token_iterator.next() {
                            Some((span, lexer::Token::GreaterThan)) => {
                                return Ok(span.high() - self.file.span.low());
                            }
                            Some((span, lexer::Token::Identifier)) => {
                                match &this.name {
                                    Either::Left(name) => {
                                        if name != self.file.source_slice(span) {
                                            return Err(vec![
                                                ParseCodes::MismatchedNameBetweenOpeningAndClosing
                                                    .to_diagnostic(&[
                                                        SpanLabel {
                                                            label: None,
                                                            span,
                                                            style: SpanStyle::Primary,
                                                        },
                                                        SpanLabel {
                                                            label: Some(
                                                                "Original name".to_string(),
                                                            ),
                                                            span: this.name_span,
                                                            style: SpanStyle::Primary,
                                                        },
                                                    ]),
                                            ]);
                                        }
                                    }
                                    Either::Right(_) => {
                                        return Err(vec![
                                            ParseCodes::MismatchedNameBetweenOpeningAndClosingWithReplacer.to_diagnostic(&[
                                                SpanLabel {
                                                    label: None,
                                                    span,
                                                    style: SpanStyle::Primary
                                                },
                                                SpanLabel {
                                                    label: Some("Original name".to_string()),
                                                    span: this.name_span,
                                                    style: SpanStyle::Primary
                                                },
                                            ])
                                        ]);
                                    }
                                }

                                // And finally final '>'
                                match self.token_iterator.next() {
                                    Some((span, lexer::Token::GreaterThan)) => {
                                        return Ok(span.high() - self.file.span.low());
                                    }
                                    Some((span, _)) => {
                                        return Err(vec![
                                            ParseCodes::ExpectingGreaterThanForClosing
                                                .to_diagnostic(&[SpanLabel {
                                                    label: None,
                                                    span: util::one_char_span(
                                                        &self.file,
                                                        span.low() - self.file.span.low(),
                                                    ),
                                                    style: SpanStyle::Primary,
                                                }]),
                                        ]);
                                    }
                                    None => {
                                        return Err(vec![
                                            ParseCodes::ExpectingGreaterThanForClosing
                                                .to_diagnostic(&[SpanLabel {
                                                    label: None,
                                                    span: self.eof_span.clone(),
                                                    style: SpanStyle::Primary,
                                                }]),
                                        ]);
                                    }
                                }
                            }
                            Some((span, lexer::Token::Replacer(_))) => {
                                return Err(vec![
                                    ParseCodes::ReplacerUnsupportedInClosingTag.to_diagnostic(&[
                                        SpanLabel {
                                            label: None,
                                            span,
                                            style: SpanStyle::Primary,
                                        },
                                    ]),
                                ]);
                            }
                            Some((span, _)) => {
                                return Err(vec![
                                    ParseCodes::ExpectingOptionalNameOrGreaterThan.to_diagnostic(
                                        &[SpanLabel {
                                            label: None,
                                            span: util::one_char_span(
                                                &self.file,
                                                span.low() - self.file.span.low(),
                                            ),
                                            style: SpanStyle::Primary,
                                        }],
                                    ),
                                ]);
                            }
                            None => {
                                return Err(vec![
                                    ParseCodes::ExpectingOptionalNameOrGreaterThan.to_diagnostic(
                                        &[SpanLabel {
                                            label: None,
                                            span: self.eof_span.clone(),
                                            style: SpanStyle::Primary,
                                        }],
                                    ),
                                ]);
                            }
                        }
                    }
                    Some((span, _)) => {
                        return Err(vec![ParseCodes::ExpectingSlashForClosingTag.to_diagnostic(
                            &[SpanLabel {
                                label: None,
                                span: util::one_char_span(
                                    &self.file,
                                    span.low() - self.file.span.low(),
                                ),
                                style: SpanStyle::Primary,
                            }],
                        )]);
                    }
                    None => {
                        return Err(vec![ParseCodes::ExpectingSlashForClosingTag.to_diagnostic(
                            &[SpanLabel {
                                label: None,
                                span: self.eof_span.clone(),
                                style: SpanStyle::Primary,
                            }],
                        )]);
                    }
                }
            }

            Some((span, _)) => {
                return Err(vec![ParseCodes::ExpectingClosingTag.to_diagnostic(&[
                    SpanLabel {
                        label: None,
                        span: util::one_char_span(&self.file, span.low() - self.file.span.low()),
                        style: SpanStyle::Primary,
                    },
                ])]);
            }

            None => {
                return Err(vec![ParseCodes::ExpectingClosingTag.to_diagnostic(&[
                    SpanLabel {
                        label: None,
                        span: self.eof_span.clone(),
                        style: SpanStyle::Primary,
                    },
                ])]);
            }
        }
    }
}

pub fn run(
    file: Arc<File>,
    tokens: Vec<(Span, lexer::Token)>,
) -> Result<Vec<(Span, ElementContent)>, Vec<Diagnostic>> {
    let mut parsed = Vec::new();
    let mut state = ParseState {
        eof_span: file.span.subspan(
            file.span.high() - file.span.low(),
            file.span.high() - file.span.low(),
        ),
        file,
        token_iterator: PushBackIterator::from(tokens.into_iter()),
    };

    while state.token_iterator.peek().is_some() {
        parsed.push(state.parse_one()?);
    }

    Ok(parsed)
}
