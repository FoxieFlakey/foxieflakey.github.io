use codemap::{File, Span};
use codemap_diagnostic::{Diagnostic, Level, SpanLabel, SpanStyle};
use either::Either;
use pushback_iter::PushBackIterator;

use crate::html::util;

// The entire <!-- ... -->
#[derive(Clone)]
pub struct Comment {
    pub content: Span,
}

#[derive(Clone)]
pub struct QuotedString {
    // Left is when the string is as it is in source
    // Right is when its replaced
    pub content: Either<Span, String>,

    // Whether the content enclosed by double quote
    // or single quote. Which dictates how content
    // can be safely quoted. If its double quoted
    // when encoding it should retain double quotes
    pub is_double_quote: bool,
}

#[derive(Clone)]
pub struct Replacer {
    pub content: Span,
    // Is it just $abc or ${abc}
    pub is_simple: bool,
}

#[derive(Clone)]
pub enum Token {
    Comment(Comment),

    // The '<' sign
    LessThan,

    // The '>' sign
    GreaterThan,

    // Identifier, specifically matchs this regex
    // [-0-9A-Za-z_:@.]+
    //
    // This only appear in between LessThan and GreaterThan (or called tag mode)
    Identifier,

    // The '=' sign
    // This also only appear in between LessThan and GreaterThan (or called tag mode)
    Equal,

    // The quoted string
    // its either '...' or "..."
    // This also only appear in between LessThan and GreaterThan (or called tag mode)
    QuotedString(QuotedString),

    // Bare text in tag
    // Only appear
    // 1. In between start of sourc code and LessThan
    // 2. In between GreaterThan and end of source code
    // 3. In between GreaterThan and LessThan
    //
    // Or in short not in tag mode
    Text,

    // The '/' symbol
    // This also only appear in between LessThan and GreaterThan
    // (or called tag mode)
    Slash,

    // My extensions, which is "replacers"
    // it does not get unescaped
    Replacer(Replacer),
}

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum LexerCodes {
    ExpectingAmpersand = 0000,
    ExpectingHTMLEntityName = 0001,
    UnknownEntityName = 0002,
    UnexpectedTagStartInsideTag = 0003,
    UnterminatedString = 0004,
    ExpectingMoreIdentifierOrNotCharacter = 0005,
    ExpectingContentOfTagOrClosing = 0006,
    UnterminatedComment = 0007,
    MalformedCommentStart = 0008,
    ExpectedCommentStartGotEOF = 0009,
    ExpectingReplacer = 0010,
    UnterminatedReplacer = 0011,
    ExpectingIdentifierCharacterOrBraceForReplacer = 0012,
}

impl LexerCodes {
    pub fn description(&self) -> &'static str {
        match self {
            LexerCodes::ExpectingAmpersand => {
                "Expecting '&' got unexpected character while parsing for HTML escape"
            }
            LexerCodes::ExpectingHTMLEntityName => "Expecting HTML entity name after '&'",
            LexerCodes::UnknownEntityName => "Unknown HTML entity name",
            LexerCodes::UnexpectedTagStartInsideTag => {
                "Unexpected tag inside tag, wasnt expecting '<' inside tag. If this is needed use &lt;"
            }
            LexerCodes::UnterminatedString => "String unterminated (met EOF, before closing quote)",
            LexerCodes::ExpectingMoreIdentifierOrNotCharacter => {
                "Expecting more identifier character or not identifer"
            }
            LexerCodes::ExpectingContentOfTagOrClosing => {
                "Expecting identifier character, whitespace, '/' or closing tag"
            }
            LexerCodes::UnterminatedComment => "Expecting more content for comment got EOF",
            LexerCodes::MalformedCommentStart => "Malformed start of comment it has to be <!--",
            LexerCodes::ExpectedCommentStartGotEOF => "Expected comment start of <!-- got EOF",
            LexerCodes::ExpectingReplacer => "Expecting '$' to be replacer",
            LexerCodes::UnterminatedReplacer => "Expecting more characters for replacer got EOF",
            LexerCodes::ExpectingIdentifierCharacterOrBraceForReplacer => {
                "Expecting identifier character or { to start replacer"
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

        format!("Lexer:{letter}{:04}", *self as u16)
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

// Assumed the caller already consumed the '&', so this parse the body
// the start is pointing the position of '&'
fn parse_html_escape<I>(
    file: &File,
    start: u64,
    iter: &mut I,
) -> Result<(Span, char), Vec<Diagnostic>>
where
    I: Iterator<Item = (u64, char)>,
{
    if !file.source()[start.try_into().unwrap()..].starts_with('&') {
        return Err(vec![LexerCodes::ExpectingAmpersand.to_diagnostic(&[
            SpanLabel {
                label: Some("Here!".to_string()),
                span: util::one_char_span(file, start),
                style: SpanStyle::Primary,
            },
        ])]);
    }

    let mut name = String::new();
    let end;
    loop {
        let (offset, c) = iter.next().ok_or_else(|| {
            vec![
                LexerCodes::ExpectingHTMLEntityName.to_diagnostic(&[SpanLabel {
                    label: Some("Here!".to_string()),
                    span: util::one_char_span(file, start),
                    style: SpanStyle::Primary,
                }]),
            ]
        })?;

        if c == ';' {
            end = util::inc_char_offset(offset, &file.source());
            break;
        }

        name.push(c);
    }

    const _: () = {
        // Because its &str in html_escape source, i have to statically check
        // at compile time if it does not contain more than 1 character
        let mut i = 0;
        while i < html_escape::NAMED_ENTITIES.len() {
            // https://www.rfc-editor.org/info/rfc3629/ section 3. UTF-8 definition
            //
            // Quote
            //
            // In UTF-8, characters from the U+0000..U+10FFFF range (the UTF-16
            // accessible range) are encoded using sequences of 1 to 4 octets.
            // ..
            //
            // The table below summarizes the format of these different octet types.
            // The letter x indicates bits available for encoding bits of the
            // character number.
            //
            // Char. number range  |        UTF-8 octet sequence
            //     (hexadecimal)    |              (binary)
            // --------------------+---------------------------------------------
            // 0000 0000-0000 007F | 0xxxxxxx
            // 0000 0080-0000 07FF | 110xxxxx 10xxxxxx
            // 0000 0800-0000 FFFF | 1110xxxx 10xxxxxx 10xxxxxx
            // 0001 0000-0010 FFFF | 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx

            // Check first byte, to get number of bytes for a given char
            let actual_length = html_escape::NAMED_ENTITIES[0].1.len();
            if actual_length == 0 {
                panic!("An entity in html_escape::NAMED_ENTITIES replaced by empty string?!");
            }

            let expected_length;
            let byte = html_escape::NAMED_ENTITIES[0].1.as_bytes()[0];
            if byte & 0b1_0000000 == 0 {
                // Leading bit is 0, its ASCII
                expected_length = 1;
            } else if byte & 0b111_00000 == 0b110_00000 {
                // Leading bit is 110, its 2 bytes long
                expected_length = 2;
            } else if byte & 0b1111_0000 == 0b1110_0000 {
                // Leading bit is 1110, its 2 bytes long
                expected_length = 3;
            } else if byte & 0b11111_000 == 0b11110_000 {
                // Leading bit is 11110, its 4 bytes long
                expected_length = 4;
            } else {
                panic!("Invalid UTF-8");
            }

            if actual_length > expected_length {
                panic!(
                    "There entity in html_escape's NAMED_ENTITIES that replaces to more than 1 characters!"
                );
            }

            i += 1;
        }
    };

    let replacement = html_escape::NAMED_ENTITIES
        .binary_search_by_key(&name.as_bytes(), |x| x.0)
        .ok()
        .map(|found_idx| html_escape::NAMED_ENTITIES[found_idx].1)
        .map(|string| string.chars().next().unwrap());

    let this_span = file.span.subspan(start, end);
    if let Some(c) = replacement {
        Ok((this_span, c))
    } else {
        Err(vec![LexerCodes::UnknownEntityName.to_diagnostic(&[
            SpanLabel {
                label: Some("Here!".to_string()),
                span: util::one_char_span(file, start),
                style: SpanStyle::Primary,
            },
        ])])
    }
}

enum WhatToDoWithText {
    DoNothing,
    SaveThenPushTokenIgnoreCurrent((Span, Token)),
    DontSaveOnlyPushTokenAndIgnoreCurrent((Span, Token)),
}

fn is_identifier(c: char) -> bool {
    match c {
        '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | ':' | '-' | '@' | '.' => true,
        _ => false,
    }
}

fn check_chars<I>(iterator: &mut I, chars: &[char]) -> Result<(), Option<u64>>
where
    I: Iterator<Item = (u64, char)>,
{
    for &expected in chars {
        let (offset, actual) = iterator.next().ok_or(None)?;

        if actual != expected {
            return Err(Some(offset));
        }
    }

    Ok(())
}

// Utility function reusing the same parse_replacer to parse replacers
// inside arbitrary stuffs (mainly used by replacer_resolver to parse
// string without duplicating functions)
pub fn parse_only_replacers(
    file: &File,
    span: Span,
) -> Result<Vec<(Span, Option<Replacer>)>, Vec<Diagnostic>> {
    let initial_offset = span.low() - file.span.low();
    let end_offset = span.high() - file.span.low();

    let mut iterator = PushBackIterator::from(
        file.source_slice(span)
            .char_indices()
            .map(|(offset, c)| (u64::try_from(offset).unwrap() + initial_offset, c)),
    );
    let mut portions = Vec::new();

    let mut last_start = None;
    loop {
        let Some((offset, char)) = iterator.next() else {
            break;
        };

        match char {
            '$' => {
                if let Some(last_offset) = last_start.take() {
                    portions.push((span.subspan(last_offset, offset), None));
                }

                let (span, replacer) =
                    parse_replacer(file, offset, Some(end_offset), &mut iterator, true)?;
                portions.push((span, Some(replacer)));
            }
            _ => {
                if last_start.is_none() {
                    last_start = Some(offset);
                }
            }
        }
    }

    if let Some(offset) = last_start {
        portions.push((
            file.span.subspan(offset, span.high() - file.span.low()),
            None,
        ));
    }

    Ok(portions)
}

// Assumed the caller already consumed the '$', so this parse
// the start is pointing the position of character after $
fn parse_replacer<I>(
    file: &File,
    start: u64,
    // only needed if EOF okay
    optional_end: Option<u64>,
    iterator: &mut PushBackIterator<I>,
    is_eof_okay: bool,
) -> Result<(Span, Replacer), Vec<Diagnostic>>
where
    I: Iterator<Item = (u64, char)>,
{
    if is_eof_okay {
        assert!(
            optional_end.is_some(),
            "End offset is required if is_eof_okay is true (has to be Some(offset))"
        );
    } else {
        assert!(
            optional_end.is_none(),
            "End offset is not required if is_eof_okay is false (has to be None)"
        );
    }

    if !file.source()[start.try_into().unwrap()..].starts_with('$') {
        return Err(vec![LexerCodes::ExpectingReplacer.to_diagnostic(&[
            SpanLabel {
                label: Some("Here!".to_string()),
                span: util::one_char_span(file, start),
                style: SpanStyle::Primary,
            },
        ])]);
    }

    let (offset, first_char) = iterator.peek().copied().ok_or_else(|| {
        vec![LexerCodes::ExpectingReplacer.to_diagnostic(&[SpanLabel {
            label: Some("Here!".to_string()),
            span: file.span.subspan(
                file.span.high() - file.span.low(),
                file.span.high() - file.span.low(),
            ),
            style: SpanStyle::Primary,
        }])]
    })?;

    let content_start;
    let end;
    let content_end;
    let is_simple;

    let context_span = SpanLabel {
        label: Some("Replacer started here".to_string()),
        span: util::one_char_span(file, start),
        style: SpanStyle::Secondary,
    };

    if first_char == '{' {
        is_simple = false;
        content_start = util::inc_char_offset(offset, file.source());
        loop {
            let (offset, c) = iterator.next().ok_or_else(|| {
                vec![LexerCodes::UnterminatedReplacer.to_diagnostic(&[
                    SpanLabel {
                        label: Some("Here!".to_string()),
                        span: file.span.subspan(
                            file.span.high() - file.span.low(),
                            file.span.high() - file.span.low(),
                        ),
                        style: SpanStyle::Primary,
                    },
                    context_span.clone(),
                ])]
            })?;

            if c == '}' {
                end = util::inc_char_offset(offset, file.source());
                content_end = offset;
                break;
            }
        }
    } else if is_identifier(first_char) {
        is_simple = true;
        content_start = offset;
        loop {
            let Some((offset, c)) = iterator.next() else {
                if !is_eof_okay {
                    return Err(vec![LexerCodes::UnterminatedReplacer.to_diagnostic(&[
                        SpanLabel {
                            label: Some("Here!".to_string()),
                            span: file.span.subspan(
                                file.span.high() - file.span.low(),
                                file.span.high() - file.span.low(),
                            ),
                            style: SpanStyle::Primary,
                        },
                        context_span.clone(),
                    ])]);
                } else {
                    end = optional_end.unwrap();
                    content_end = optional_end.unwrap();
                    break;
                }
            };

            if !is_identifier(c) {
                end = offset;
                content_end = end;
                iterator.push_back((offset, c));
                break;
            }
        }
    } else {
        return Err(vec![
            LexerCodes::ExpectingIdentifierCharacterOrBraceForReplacer.to_diagnostic(&[
                SpanLabel {
                    label: Some("Here!".to_string()),
                    span: file.span.subspan(
                        file.span.high() - file.span.low(),
                        file.span.high() - file.span.low(),
                    ),
                    style: SpanStyle::Primary,
                },
            ]),
        ]);
    }

    Ok((
        file.span.subspan(start, end),
        Replacer {
            content: file.span.subspan(content_start, content_end),
            is_simple,
        },
    ))
}

// The error printed on stderr directly
// no special errors because this crate
// NOT intended to be used as library
//
// That simplify managements
pub fn run(file: &File) -> Result<Vec<(Span, Token)>, Vec<Diagnostic>> {
    let mut tokens = vec![];
    let source = file.source();
    let eof_span = file.span.subspan(
        file.span.high() - file.span.low(),
        file.span.high() - file.span.low(),
    );
    let mut iterator = PushBackIterator::from(
        source
            .char_indices()
            .map(|(x, y)| (u64::try_from(x).unwrap(), y)),
    );
    let mut text_start = None;

    let mut tag_started_since = None;
    while let Some((offset, c)) = iterator.next() {
        let current_char_span = file
            .span
            .subspan(offset, util::inc_char_offset(offset, file.source()));

        // The token that is succeesfully parsed or None
        // if there no token got yet
        //
        // If a token is parsed, then the current text is saved
        let token;

        if c.is_whitespace() {
            if text_start.is_none() && tag_started_since.is_none() {
                text_start = Some(offset);
            }
            continue;
        }

        match c {
            '<' => {
                if let Some((since, _)) = tag_started_since.as_ref() {
                    return Err(vec![LexerCodes::UnexpectedTagStartInsideTag.to_diagnostic(
                        &[
                            SpanLabel {
                                label: Some("Here!".to_string()),
                                span: util::one_char_span(file, offset),
                                style: SpanStyle::Primary,
                            },
                            SpanLabel {
                                label: Some("Previous tag opened here!".to_string()),
                                span: util::one_char_span(file, *since),
                                style: SpanStyle::Secondary,
                            },
                        ],
                    )]);
                } else {
                    // Look ahead if its identifier characters or '/' or '$', then its definite open else not :3
                    if let Some((_, c)) = iterator.peek() {
                        if is_identifier(*c) || *c == '/' || *c == '$' {
                            token = WhatToDoWithText::SaveThenPushTokenIgnoreCurrent((
                                current_char_span,
                                Token::LessThan,
                            ));
                            tag_started_since = Some((
                                offset,
                                // Storing diagnostic, to refer context of current tag
                                // that is currently lexing. So other places can put
                                // this as context
                                Diagnostic {
                                    code: None,
                                    level: Level::Note,
                                    message: "Parsing content of this tag".to_string(),
                                    spans: vec![SpanLabel {
                                        label: Some("Here!".to_string()),
                                        span: current_char_span.clone(),
                                        style: SpanStyle::Primary,
                                    }],
                                },
                            ));
                        } else if *c == '!' {
                            // Else its comment
                            let (exclamation_start, _) = iterator.next().unwrap();

                            let start = offset;
                            check_chars(&mut iterator, &['-', '-']).map_err(|offset| {
                                vec![if let Some(offset) = offset {
                                    LexerCodes::MalformedCommentStart.to_diagnostic(&[SpanLabel {
                                        label: Some("Here!".to_string()),
                                        span: util::one_char_span(file, offset),
                                        style: SpanStyle::Primary,
                                    }])
                                } else {
                                    LexerCodes::ExpectedCommentStartGotEOF.to_diagnostic(&[
                                        SpanLabel {
                                            label: Some("Here!".to_string()),
                                            span: eof_span.clone(),
                                            style: SpanStyle::Primary,
                                        },
                                    ])
                                }]
                            })?;

                            let content_start =
                                util::inc_char_offset(exclamation_start + 2, file.source());

                            let end;

                            let mut history: [Option<(u64, char)>; 4] = [
                                None, // At the end, must point to '>'
                                None, // At the end, must point to '-'
                                None, // At the end, must point to '-'
                                None, // At the end, must point to comment content or None if empty
                            ];

                            loop {
                                let (offset, c) = iterator.next().ok_or_else(|| {
                                    vec![LexerCodes::UnterminatedComment.to_diagnostic(&[
                                        SpanLabel {
                                            label: Some("Here!".to_string()),
                                            span: eof_span,
                                            style: SpanStyle::Primary,
                                        },
                                        SpanLabel {
                                            label: Some("Was iniated here!".to_string()),
                                            span: util::one_char_span(file, start),
                                            style: SpanStyle::Secondary,
                                        },
                                    ])]
                                })?;

                                history[3] = history[2];
                                history[2] = history[1];
                                history[1] = history[0];
                                history[0] = Some((offset, c));

                                // Check for ending
                                match (history[2], history[1], history[0]) {
                                    (Some((_, '-')), Some((_, '-')), Some((_, '>'))) => {
                                        // Found the ending -->
                                        end = util::inc_char_offset(offset, file.source());
                                        break;
                                    }
                                    _ => (),
                                }
                            }

                            let content_end = history[3].map(|x| x.0).unwrap_or(content_start);

                            let this_span = file.span.subspan(start, end);
                            token = WhatToDoWithText::DontSaveOnlyPushTokenAndIgnoreCurrent((
                                this_span,
                                Token::Comment(Comment {
                                    content: file.span.subspan(content_start, content_end),
                                }),
                            ));
                        } else {
                            token = WhatToDoWithText::DoNothing;
                            // False start, start saving text instead
                            if text_start.is_none() {
                                text_start = Some(offset);
                            }
                        }
                    } else {
                        token = WhatToDoWithText::DoNothing;
                        // False start, start saving text instead
                        if text_start.is_none() {
                            text_start = Some(offset);
                        }
                    }
                }
            }

            '>' => {
                if tag_started_since.is_some() {
                    token = WhatToDoWithText::SaveThenPushTokenIgnoreCurrent((
                        current_char_span,
                        Token::GreaterThan,
                    ));
                    tag_started_since = None;
                } else {
                    token = WhatToDoWithText::DoNothing;
                }
            }

            '=' if tag_started_since.is_some() => {
                token = WhatToDoWithText::SaveThenPushTokenIgnoreCurrent((
                    current_char_span,
                    Token::Equal,
                ));
            }

            '/' if tag_started_since.is_some() => {
                token = WhatToDoWithText::SaveThenPushTokenIgnoreCurrent((
                    current_char_span,
                    Token::Slash,
                ));
            }

            '$' => {
                token = WhatToDoWithText::SaveThenPushTokenIgnoreCurrent(
                    parse_replacer(file, offset, None, &mut iterator, false)
                        .map(|(x, y)| (x, Token::Replacer(y)))?,
                );
            }

            '"' | '\'' if let Some((_, context_diag)) = &tag_started_since => {
                let closing_char = c;

                // Holds the start of string the first quote character
                let start = offset;
                let end;

                // Holds the content of string
                let content_start = util::inc_char_offset(offset, file.source());
                let content_end;
                loop {
                    let (offset, c) = iterator.next().ok_or_else(|| {
                        vec![
                            LexerCodes::UnterminatedString.to_diagnostic(&[
                                SpanLabel {
                                    label: Some("Here!".to_string()),
                                    span: eof_span,
                                    style: SpanStyle::Primary,
                                },
                                SpanLabel {
                                    label: Some("String was opened here!".to_string()),
                                    span: util::one_char_span(file, start),
                                    style: SpanStyle::Secondary,
                                },
                            ]),
                            context_diag.clone(),
                        ]
                    })?;

                    if c == closing_char {
                        end = util::inc_char_offset(offset, file.source());
                        content_end = offset;
                        break;
                    }

                    if c == '&' {
                        parse_html_escape(file, offset, &mut iterator).map_err(|mut x| {
                            x.push(context_diag.clone());
                            x
                        })?;
                    }
                }

                token = WhatToDoWithText::SaveThenPushTokenIgnoreCurrent((
                    file.span.subspan(start, end),
                    Token::QuotedString(QuotedString {
                        is_double_quote: closing_char == '"',
                        content: Either::Left(file.span.subspan(content_start, content_end)),
                    }),
                ));
            }

            c => {
                if let Some((_, context_diag)) = &tag_started_since {
                    let start = offset;
                    let mut end = offset;

                    let mut c = c;
                    // Probably an identifier try parse for it
                    loop {
                        if is_identifier(c) {
                            // Its identifier character, continue look for it
                            if let Some(x) = iterator.next() {
                                (end, c) = x;
                            } else {
                                return Err(vec![
                                    LexerCodes::ExpectingMoreIdentifierOrNotCharacter
                                        .to_diagnostic(&[SpanLabel {
                                            label: Some("Here".to_string()),
                                            span: util::one_char_span(file, end),
                                            style: SpanStyle::Primary,
                                        }]),
                                    context_diag.clone(),
                                ]);
                            }
                        } else {
                            // White space is terminator too
                            if c.is_whitespace() {
                                iterator.push_back((end, c));
                                break;
                            }

                            // Check if its terminators, mainly copy of everything from outer loop
                            match c {
                                '/' | '"' | '\'' | '>' | '<' | '=' | ' ' => {
                                    // Repush it so outer can handle
                                    iterator.push_back((end, c));
                                    break;
                                }
                                _ => (),
                            }

                            return Err(vec![
                                LexerCodes::ExpectingContentOfTagOrClosing.to_diagnostic(&[
                                    SpanLabel {
                                        label: Some("Here".to_string()),
                                        span: util::one_char_span(file, end),
                                        style: SpanStyle::Primary,
                                    },
                                ]),
                            ]);
                        }
                    }

                    if end > start {
                        // There atleast one character to be the identifier!
                        token = WhatToDoWithText::DontSaveOnlyPushTokenAndIgnoreCurrent((
                            file.span.subspan(start, end),
                            Token::Identifier,
                        ))
                    } else {
                        token = WhatToDoWithText::DoNothing;
                    }
                } else {
                    // Save the text
                    token = WhatToDoWithText::DoNothing;
                    if text_start.is_none() {
                        text_start = Some(offset);
                    }

                    // Parse HTML escape, just check if its correct or not. After the entire file
                    // is lexed it is guarantee only valid html entities in texts
                    if c == '&' {
                        parse_html_escape(file, offset, &mut iterator).map_err(|mut x| {
                            x.push(Diagnostic {
                                code: None,
                                level: Level::Note,
                                message: "Parsing HTML escape outside of tag".to_string(),
                                spans: vec![SpanLabel {
                                    label: Some("Here!".to_string()),
                                    span: current_char_span,
                                    style: SpanStyle::Primary,
                                }],
                            });
                            x
                        })?;
                    }
                }
            }
        }

        match token {
            WhatToDoWithText::DontSaveOnlyPushTokenAndIgnoreCurrent(token) => {
                text_start = None;
                tokens.push(token);
            }
            WhatToDoWithText::SaveThenPushTokenIgnoreCurrent(token) => {
                if let Some(text_start) = text_start.take() {
                    if offset > text_start {
                        tokens.push((file.span.subspan(text_start, offset), Token::Text));
                    }
                }
                tokens.push(token);
            }
            WhatToDoWithText::DoNothing => (),
        }
    }

    if let Some(offset) = text_start {
        tokens.push((
            file.span.subspan(offset, source.len().try_into().unwrap()),
            Token::Text,
        ));
    }

    Ok(tokens)
}
