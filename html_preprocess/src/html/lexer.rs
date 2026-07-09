use std::sync::Arc;

use codemap::{File, Span};
use miette::{Diagnostic, NamedSource};
use thiserror::Error;

use crate::html::util::MietteFile;

struct Lexer {
    file: Arc<File>
}

// The entire <!-- ... -->
#[derive(Clone)]
pub struct Comment {
    pub this: Span,
    pub content: Span
}

// Just the '<' symbol
#[derive(Clone)]
pub struct SmallerThan {
    pub this: Span
}

// The '>' symbol
#[derive(Clone)]
pub struct GreaterThan {
    pub this: Span
}

// Identifier, specifically matchs this regex
// [-0-9A-Za-z_:@.]+
#[derive(Clone)]
pub struct Identifier {
    pub this: Span
}

// The '/' symbol
#[derive(Clone)]
pub struct Slash {
    pub this: Span
}

// The "=" symbol
#[derive(Clone)]
pub struct Equal {
    pub this: Span
}

#[derive(Clone)]
pub struct QuotedString {
    pub this: Span,
    pub content: Span,
    
    // The unescaped version after HTML's
    // unescapting the only known &..; which for now
    // only &amp; then &lt; then &gt; then &quot; then
    // &apos; &nbsp;
    pub unescaped: Arc<String>
}

// Bare text in HTML elements
#[derive(Clone)]
pub struct Text {
    pub this: Span,
    
    // Same rule as QuotedString's unescaped
    pub unescaped: Arc<String>
}

#[derive(Clone)]
pub struct Replacer {
    pub this: Span,
    pub content: Span
}

#[derive(Clone)]
pub enum Token {
    Comment(Comment),
    SmallerThan(SmallerThan),
    GreaterThan(GreaterThan),
    Identifier(Identifier),
    Equal(Equal),
    Text(Text),
    
    // My extensions, which is "replacers"
    // it does not get unescaped
    Replacer(Replacer)
}

#[derive(Error, Diagnostic, Debug)]
#[diagnostic(code(lexer::error))]
pub enum LexerError {
    #[error("Expecting --> to close comment got EOF")]
    ExpectingClosingCommentGotEof {
        #[source_code]
        src: MietteFile,
        
        #[label("here!")]
        location: miette::SourceSpan
    }
}

pub fn run()  {
    
}

