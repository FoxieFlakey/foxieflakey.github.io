// My HTML parser :3
// Its not fully compliant to HTML at "deep" levels
// just works for majority and sane HTMLs not including
// old baggaes
//
// There also tweaks like for sanification
// 1. case sensitive
// 2. stuffs like ${...} where ... can be anything
//    placed in text spaces, property, tag names, etc
//    The ${...} named replacer
// 3. Unquoted value is disallowed
// 4. Nested comment don't work (they make 0 sense)
// 5. Comment only works as children of element or
//    attributes, it doesnt work in tag name so
//    < <!-- --> abc></abc>
//    is invalid
// 6. "Identifier" names are smaller list than actual HTML
//    Valid chars are 0-9-_A-Za-z:
// 7. Closing tag must have exact same byte to byte value as
//    the opening after trimming whitespaces

use std::cmp;

use char_positions::{CharPositions, CharPositionsExt, LineColByte};
use pushback_iter::PushBackIterator;

#[derive(Clone, Copy)]
pub struct Location {
    pub line: u32,
    pub column: u32,
    pub byte_offset: usize,
}

// NOTE: Spans are inclusive on both ends
#[derive(Clone, Copy)]
pub struct Span<'a> {
    pub start: Location,
    pub end: Location,

    // Original &str, not corresponding to current Span
    source: &'a str,
}

pub enum Identifier<'a> {
    Replacer(Replacer<'a>),
    Parsed(Span<'a>, &'a str),
}

pub enum Attribute<'a> {
    Replacer(Replacer<'a>),
    Comment(Span<'a>, &'a str),
    Parsed {
        this_span: Span<'a>,
        key_span: Span<'a>,
        value_span: Span<'a>,
    },
}

pub enum Replacer<'a> {
    // ${text} syntax
    Complex(Span<'a>, &'a str),
    
    // $variable syntax
    Simple(Span<'a>, &'a str)
}

impl Replacer<'_> {
    pub fn is_same(&self, other: &Self) -> bool {
        match (self, other) {
            (Replacer::Complex(_, a), Replacer::Complex(_, b)) => a == b,
            (Replacer::Simple(_, a), Replacer::Simple(_, b)) => a == b,
            _ => false
        }
    }
    
    pub fn debug_name(&self) -> &str {
        match self {
            Replacer::Complex(_, a) => *a,
            Replacer::Simple(_, a) => *a,
        }
    }
}

pub enum ElementContent<'a> {
    Replacer(Replacer<'a>),
    Text(Span<'a>, &'a str),
    Comment(Span<'a>, &'a str),
    Element(Element<'a>),
}

pub struct Element<'a> {
    pub this_span: Span<'a>,
    pub name: Identifier<'a>,

    // There can be duplicate key/value pair here too
    // up to caller to handle it
    pub attributes: Vec<Attribute<'a>>,

    // List of fragment/section of content
    // like the span of these come one after another
    pub content: Vec<ElementContent<'a>>,
}

struct State<'a> {
    source: &'a str,
    // Iterator of chars
    char_iter: PushBackIterator<CharPositions<'a, LineColByte>>,
    location_stack: Vec<Location>,
    cur_location: Location,
    cur_char: Option<char>,
    eof_met: bool,
}

impl<'a> Identifier<'a> {
    pub fn debug_name(&self) -> &str {
        match self {
            Identifier::Parsed(_, x) => x,
            Identifier::Replacer(x) => x.debug_name(),
        }
    }
    
    pub fn is_same_identifier(&self, other: &Self) -> bool {
        match (self, other) {
            (Identifier::Parsed(_, x), Identifier::Parsed(_, y)) => x == y,
            (Identifier::Replacer(x), Identifier::Replacer(y)) => x.is_same(y),
            _ => false
        }
    }
}

impl<'a> State<'a> {
    fn peek(&mut self) -> Option<(LineColByte, char)> {
        self.char_iter.peek().copied()
    }

    // This does not skip whitespace, manually skip it before
    fn next_char(&mut self) -> Option<(LineColByte, char)> {
        let Some((pos, chr)) = self.char_iter.next() else {
            if !self.eof_met {
                self.cur_location.column += 1;
                self.eof_met = true;
            }
            return None;
        };

        self.cur_location.line = u32::try_from(pos.0).unwrap();
        self.cur_location.column = u32::try_from(pos.1).unwrap();
        self.cur_location.byte_offset = pos.2;
        self.cur_char = Some(chr);

        Some((pos, chr))
    }

    fn push_position(&mut self) -> Location {
        self.location_stack.push(self.cur_location);
        self.cur_location
    }

    fn pop_position(&mut self) -> Span<'a> {
        Span {
            start: self.location_stack.pop().unwrap(),
            end: self.cur_location,
            source: self.source,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some((_, char)) = self.peek() {
            if !char.is_whitespace() {
                return;
            }

            self.next_char().unwrap();
        }
    }

    fn unnext_char(&mut self) {
        let chr = self.cur_char.take().expect("cant unnext char (because its either first char/already unnext)");
        self.char_iter.push_back((
            LineColByte(
                self.cur_location.line.try_into().unwrap(),
                self.cur_location.column.try_into().unwrap(),
                self.cur_location.byte_offset,
            ),
            chr,
        ));
    }

    fn check_char(&mut self, expected: char) -> Result<(), ParseError<'a>> {
        let chr = self
            .next_char()
            .ok_or_else(|| {
                ParseError::new(
                    self,
                    format!("Expected '{expected}' to begin element got EOF"),
                )
            })?
            .1;
        if chr != expected {
            return Err(ParseError::new(
                self,
                format!(
                    "Expected character '{}' got '{}'",
                    expected.escape_default(),
                    chr.escape_default()
                ),
            ));
        }
        Ok(())
    }

    fn parse_identifier_or_replacer(&mut self) -> Result<Identifier<'a>, ParseError<'a>> {
        let result = self.parse_identifier()?;
        Ok(Identifier::Parsed(result.0, result.1))
    }
    
    // This strictly ignores whitespaces, and terminates when its not valid identifier character
    fn parse_identifier(&mut self) -> Result<(Span<'a>, &'a str), ParseError<'a>> {
        let start = self.push_position();
        loop {
            let (pos, char) = self.next_char().ok_or_else(|| {
                ParseError::new(self, "expected more character got EOF")
            })?;

            match char {
                '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '_' | ':' => {
                    continue;
                }
                _ => {
                    self.unnext_char();
                    return Ok((self.pop_position(), &self.source[start.byte_offset + 1..pos.2]));
                }
            }
        }
    }

    fn parse_element(&mut self) -> Result<Element<'a>, ParseError<'a>> {
        self.skip_whitespace();
        self.push_position();

        // Parsing the openign tag
        ////////////////////////////////
        self.check_char('<')?;
        let identifier = self
            .parse_identifier_or_replacer()
            .map_err(|x| x.context(self, "Reading identifier"))?;
        self.check_char('>')?;
        ////////////////////////////////

        // Parsing the text content, may be nest of other elements
        ////////////////////////////////
        let mut content = Vec::new();
        let mut start_text: Option<Location> = None;
        loop {
            let (pos, chr) = self
                .next_char()
                .ok_or_else(|| ParseError::new(self, "Expected any character or '<' got EOF"))?;
            
            if chr == '<' {
                let child_location = self.cur_location;
                
                // Current text portions are ended, save it, if exists
                if let Some(start_text) = start_text {
                    let text_str = &self.source[start_text.byte_offset..pos.2];
                    content.push(ElementContent::Text(
                        Span {
                            start: start_text,
                            end: self.cur_location,
                            source: text_str,
                        },
                        text_str,
                    ));
                }
                start_text = None;

                // Lets see if its ending
                let (_, chr) = self.peek().ok_or_else(|| {
                    ParseError::new(self, "Expected identifier chars or '/' got EOF")
                })?;
                if chr == '/' {
                    break;
                }

                // Parse child element
                // put the '<' back
                self.unnext_char();

                content.push(ElementContent::Element(self.parse_element().map_err(
                    |x| {
                        x.context_with_location(
                            self,
                            child_location,
                            format!("Parsing child element of {}", identifier.debug_name()),
                        )
                    },
                )?));
            } else {
                // Try begin new span of text, if it havent
                if start_text.is_none() {
                    start_text = Some(self.cur_location);
                }
            }
        }
        ////////////////////////////////

        // Parsing the closing tag, at this
        // previous loop encountered < and peeked
        // to be '/'
        ////////////////////////////////
        self.check_char('/')?;
        let closing_position = self.cur_location;
        let closing = self
            .parse_identifier_or_replacer()
            .map_err(|x| x.context(self, "Reading identifier"))?;
        self.check_char('>')?;
        ////////////////////////////////

        if !identifier.is_same_identifier(&closing) {
            return Err(ParseError::new_with_location(self, closing_position, format!("Closing tag has different name than opening ('{}')!", identifier.debug_name())))
        }

        Ok(Element {
            attributes: Vec::new(),
            content,
            name: identifier,
            this_span: self.pop_position(),
        })
    }
}

pub fn parse<'a>(string: &'a str) -> Result<Element<'a>, ParseError<'a>> {
    State {
        char_iter: PushBackIterator::from(string.char_positions()),
        cur_location: Location {
            byte_offset: 0,
            column: 1,
            line: 1,
        },
        location_stack: Vec::new(),
        source: string,
        cur_char: None,
        eof_met: false,
    }
    .parse_element()
}

pub struct ParseError<'a> {
    at: Location,
    span: Span<'a>,
    message: String,

    // If an error happen inside other element
    context_chain: Option<Box<ParseError<'a>>>,
}

impl<'a> ParseError<'a> {
    // NOTE: this pops current position
    // create this in .ok_or_else, map_err, etc
    fn new<T: Into<String>>(state: &mut State<'a>, message: T) -> Self {
        Self {
            at: state.cur_location,
            message: message.into(),
            span: state.pop_position(),
            context_chain: None,
        }
    }
    
    fn new_with_location<T: Into<String>>(state: &mut State<'a>, location: Location, message: T) -> Self {
        Self {
            at: location,
            message: message.into(),
            span: state.pop_position(),
            context_chain: None,
        }
    }

    fn context<T: Into<String>>(mut self, state: &mut State<'a>, message: T) -> Self {
        let mut cur = &mut self;
        loop {
            if cur.context_chain.is_none() {
                cur.context_chain = Some(Box::new(ParseError::new(state, message)));
                break;
            }

            cur = cur.context_chain.as_mut().unwrap();
        }

        self
    }

    fn context_with_location<T: Into<String>>(
        mut self,
        state: &mut State<'a>,
        location: Location,
        message: T,
    ) -> Self {
        let mut cur = &mut self;
        loop {
            if cur.context_chain.is_none() {
                cur.context_chain = Some(Box::new(Self::new_with_location(state, location, message)));
                break;
            }

            cur = cur.context_chain.as_mut().unwrap();
        }

        self
    }

    pub fn print_error(&self) {
        println!("Cannot parse: {}", self.message);
        let all_lines = self.span.source.lines().collect::<Vec<_>>();
        let context_length = 3;

        let first_line_to_print = usize::try_from(self.span.start.line)
            .unwrap()
            .saturating_sub(context_length + 2);
        let last_line_to_print = cmp::min(
            usize::try_from(self.span.end.line).unwrap() + context_length + 1,
            all_lines.len(),
        );

        for (mut line_index, &line) in all_lines[first_line_to_print..last_line_to_print]
            .iter()
            .enumerate()
        {
            line_index += first_line_to_print;

            println!("{:4} | {line}", line_index + 1);

            if line_index == usize::try_from(self.at.line - 1).unwrap() {
                // Show the pointer of where error occured
                println!(
                    "      {}^ here!",
                    " ".repeat(usize::try_from(self.at.column).unwrap())
                );
            }
        }

        if let Some(ctx) = &self.context_chain {
            println!("Context:");
            ctx.print_error();
        }
    }
}
