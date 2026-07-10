// My HTML preprocessor :3
//
// Its not fully compliant to HTML at "deep" levels
// just works for majority and sane HTMLs not including
// old baggaes
//
// There also tweaks like for sanification
// 1. case sensitive
// 2. stuffs like ${...} where ... can be anything
//    placed in text spaces, attribute value, etc
//    ofc validity depends on context you cant put
//    html tag in attribute value. But you can put
//    string in either attribute value or in text
//    The ${...} named replacer
// 3. Unquoted value is disallowed
// 4. Nested comment works as long as <!-- and --> are
//    balanced
// 5. Comment only works as children of element or
//    attributes, it doesnt work in tag name so
//    < <!-- --> abc></abc>
//    is invalid
// 6. "Identifier" names are smaller list than actual HTML
//    Valid chars are 0-9-_A-Za-z:
// 7. Closing tag must have exact same byte to byte value as
//    the opening after trimming whitespaces
// 8. You dont need to escape most of time inside attribute values
//    only needs &quot; (for " inside "...") &apos; (for ' inside '...')
//    and &amp; (for typing & literally to not get confused)

use codemap::{CodeMap, File, Span};
use codemap_diagnostic::Diagnostic;

use crate::html::lexer::Token;

mod lexer;
mod util;

pub struct Preprocessor<'a> {
    code_map: CodeMap,
    #[expect(unused)]
    fetcher: Box<dyn FnMut(&str) -> Result<String, String> + 'a>
}

impl<'a> Preprocessor<'a> {
    pub fn new<F>(file_fetcher: F) -> Self
        where F: FnMut(&str) -> Result<String, String> + 'a
    {
        Self {
            code_map: CodeMap::new(),
            fetcher: Box::new(file_fetcher)
        }
    }

    fn perform_lex(&self, file: &File) -> Result<Vec<(Span, Token)>, Vec<Diagnostic>> {
        lexer::run(file)
    }

    pub fn get_codemap(&self) -> &CodeMap {
        &self.code_map
    }

    pub fn parse_file(&mut self, name: &str, source: String) -> Result<(), Vec<Diagnostic>> {
        let file = self.code_map.add_file(name.to_string(), source);
        let tokens = self.perform_lex(&file)?;

        for (idx, token) in tokens.iter().enumerate() {
            let token_slice = file.source_slice(token.0);
            print!("Token[{idx}] (raw: '{}') = ", token_slice.escape_default());

            match &token.1 {
                Token::Equal => println!("Equal"),
                Token::LessThan => println!("LessThan"),
                Token::GreaterThan => println!("GreaterThan"),
                Token::Identifier => println!("Identifer('{}')", token_slice.escape_default()),
                Token::Slash => println!("Slash"),
                Token::Text => println!("Text('{}')", token_slice.escape_default()),
                Token::Comment(comment) => {
                    println!("Comment('{}')", file.source_slice(comment.content).escape_default())
                }
                Token::QuotedString(quoted_string) => println!(
                    "QuotedString('{}', is_double_quote = {})",
                    file.source_slice(quoted_string.content).escape_default(),
                    quoted_string.is_double_quote
                ),
                Token::Replacer(replacer) => println!(
                    "Replacer('{}', isSimple = {})",
                    file.source_slice(replacer.content).escape_default(),
                    replacer.is_simple
                ),
            }
        }

        Ok(())
    }
}
