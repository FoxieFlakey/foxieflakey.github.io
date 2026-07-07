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

use char_positions::{CharPositions, CharPositionsExt, LineColByteRange};
use pushback_iter::PushBackIterator;

use crate::html_display;

#[derive(Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

impl From<LineColByteRange> for Location {
    fn from(value: LineColByteRange) -> Self {
        Self {
            line: value.0,
            column: value.1,
            byte_offset: value.2.start
        }
    }
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
    char_iter: PushBackIterator<CharPositions<'a, Location>>,
    location_stack: Vec<Location>,
    cur_location: Location,
    eof_met: bool,
}

impl<'a> Identifier<'a> {
    pub fn is_same_identifier(&self, other: &Self) -> bool {
        match (self, other) {
            (Identifier::Parsed(_, x), Identifier::Parsed(_, y)) => x == y,
            (Identifier::Replacer(x), Identifier::Replacer(y)) => x.is_same(y),
            _ => false
        }
    }
}

impl<'a> State<'a> {
    fn peek(&mut self) -> Option<(Location, char)> {
        self.char_iter.peek().copied()
    }

    // This does not skip whitespace, manually skip it before
    fn next_char(&mut self) -> Option<(Location, char)> {
        let Some((pos, chr)) = self.char_iter.next() else {
            if !self.eof_met {
                self.cur_location.column += 1;
                self.eof_met = true;
            }
            return None;
        };

        self.cur_location = pos;

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

    fn unnext_char(&mut self, location: Location, chr: char) {
        self.char_iter.push_back((
            location,
            chr
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

    fn parse_replacer(&mut self) -> Result<Replacer<'a>, ParseError<'a>> {
        self.push_position();
        self.check_char('$').map_err(|x| x.context(self, "Parsing replacer"))?;
        
        let (location, chr) = self.next_char().ok_or_else(|| ParseError::new(self, "Expected { or identifier character while parsing replacer"))?;
        
        if chr == '{' {
            let start = self.cur_location;
            loop {
                let (pos, char) = self.next_char().ok_or_else(|| {
                    ParseError::new(self, "expected more character for complex replacer got EOF")
                })?;
                
                if char == '}' {
                    // End of complex replacer, quit
                    return Ok(Replacer::Complex(self.pop_position(), &self.source[start.byte_offset + 1..pos.byte_offset]))
                }
            }
        } else {
            self.unnext_char(location, chr);
            let identifier = self.parse_identifier()?;
            Ok(Replacer::Simple(self.pop_position(), identifier.1))
        }
    }

    fn parse_identifier_or_replacer(&mut self) -> Result<Identifier<'a>, ParseError<'a>> {
        let chr = self.peek().ok_or_else(|| ParseError::new(self, "Expected $ or identifier character while parsing identifier or replacer"))?.1;
        if chr == '$' {
            // Replacer
            Ok(Identifier::Replacer(self.parse_replacer()?))
        } else {
            // Plain identifier
            let result = self.parse_identifier()?;
            Ok(Identifier::Parsed(result.0, result.1))
        }
    }
    
    // This strictly ignores whitespaces, and terminates when its not valid identifier character
    fn parse_identifier(&mut self) -> Result<(Span<'a>, &'a str), ParseError<'a>> {
        let mut start = None;
        loop {
            let (pos, char) = self.next_char().ok_or_else(|| {
                ParseError::new_with_location(self, self.cur_location, "expected more character got EOF")
            })?;
            
            if start.is_none() {
                start = Some(self.push_position());
            }

            match char {
                '0'..='9' | 'A'..='Z' | 'a'..='z' | '-' | '_' | ':' => {
                    continue;
                }
                _ => {
                    self.unnext_char(pos, char);
                    if let Some(start) = start {
                        return Ok((self.pop_position(), &self.source[start.byte_offset..pos.byte_offset]));
                    } else {
                        return Err(ParseError::new_with_location(self, self.cur_location, "Expected atleast one character for identifier got none"))
                    }
                }
            }
        }
    }

    fn parse_comment(&mut self) -> Result<(Span<'a>, &'a str), ParseError<'a>> {
        self.push_position();
        self.check_char('<')?;
        self.check_char('!')?;
        self.check_char('-')?;
        self.check_char('-')?;
        
        let mut start = None;
        
        let mut history: [Option<(Location, char)>; 4] = [
            None, // At the end, must point to '>'
            None, // At the end, must point to '-'
            None, // At the end, must point to '-'
            None  // At the end, must point to comment content or None if empty
        ];
        
        loop {
            let (pos, char) = self.next_char().ok_or_else(|| ParseError::new(self, "expecting comment closing or any character got EOF"))?;
            if start.is_none() {
                start = Some(self.cur_location);
            }
            
            history[3] = history[2];
            history[2] = history[1];
            history[1] = history[0];
            history[0] = Some((pos, char));
            
            // Check for ending
            match (history[2], history[1], history[0]) {
                (Some((_, '-')), Some((_, '-')), Some((_, '>'))) => {
                    // Found the ending -->
                    break;
                }
                _ => ()
            }
        }
        
        if let Some((last_pos, _)) = history[3] {
            if let Some(start) = start {
                Ok((self.pop_position(), &self.source[start.byte_offset..=last_pos.byte_offset]))
            } else {
                panic!("There last position but not start")
            }
        } else {
            Ok((self.pop_position(), &""))
        }
    }

    fn parse_element(&mut self) -> Result<Element<'a>, ParseError<'a>> {
        self.skip_whitespace();
        self.push_position();

        // Parsing the openign tag
        ////////////////////////////////
        self.check_char('<')?;
        self.skip_whitespace();
        let identifier = self
            .parse_identifier_or_replacer()
            .map_err(|x| x.context(self, "Reading identifier"))?;
        self.skip_whitespace();
        
        if self.peek().ok_or_else(|| ParseError::new(self, "Expected /> for void tag or > for normal tag got EOF"))?.1 == '/' {
            self.check_char('/')?;
            self.check_char('>')?;
            
            // Void tag like <img />
            return Ok(Element {
                attributes: Vec::new(),
                content: Vec::new(),
                name: identifier,
                this_span: self.pop_position()
            })
        }
        
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
            
            if chr == '<' || chr == '$' {
                let child_location = self.cur_location;
                
                // Current text portions are ended, save it, if exists
                if let Some(start_text) = start_text {
                    let text_str = &self.source[start_text.byte_offset..pos.byte_offset];
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
                
                if chr == '$' {
                    // Parse replacer
                    // put '$' back
                    self.unnext_char(pos, chr);
                    
                    content.push(ElementContent::Replacer(self.parse_replacer().map_err(|x| {
                        x.context_with_location(
                            self,
                            child_location,
                            format!("Parsing replacer in {}", html_display::DisplayIdentifier(&identifier)),
                        )
                    })?));
                } else {
                    // Lets see if its ending
                    let (pos, chr) = self.peek().ok_or_else(|| {
                        ParseError::new(self, "Expected identifier chars or '/' got EOF")
                    })?;
                    if chr == '/' {
                        break;
                    }
                    
                    match chr {
                        '/' => break,
                        '!' => {
                            // Detected comment :3
                            // carefuly unnext the '<' which positioned at 'child_location'
                            self.unnext_char(child_location, '<');
                            
                            let (span, comment) = self.parse_comment()
                                .map_err(|x| x.context_with_location(self, child_location, format!("Parsing comment in {}", html_display::DisplayIdentifier(&identifier))))?;
                            
                            content.push(ElementContent::Comment(span, comment));
                        }
                        _ => {
                            // Parse child element
                            // put the '<' back
                            self.unnext_char(pos, '<');

                            content.push(ElementContent::Element(self.parse_element().map_err(
                                |x| {
                                    x.context_with_location(
                                        self,
                                        child_location,
                                        format!("Parsing child element in {}", html_display::DisplayIdentifier(&identifier)),
                                    )
                                },
                            )?));
                        }
                    }
                }
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
        self.skip_whitespace();
        
        let closing_position = self.peek()
            .ok_or_else(|| ParseError::new(self, "expecting closing tag, got EOF"))
            .map(|x| x.0)?;
        
        let closing = self
            .parse_identifier_or_replacer()
            .map_err(|x| x.context(self, "Reading identifier"))?;
        self.skip_whitespace();
        self.check_char('>')?;
        ////////////////////////////////

        if !identifier.is_same_identifier(&closing) {
            return Err(ParseError::new_with_location(self, closing_position, format!("Closing tag has different name than opening ('{}')!", html_display::DisplayIdentifier(&identifier))))
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
