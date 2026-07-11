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
// 9. Like normal HTML, first attribute win, if there duplicate
// 10. Unlike normal HTML, you can put </> to close any elements
//    will be useful when using replacers for tag names but <>
//    cannot be used to open a tag

use std::sync::Arc;

use codemap::{CodeMap, File, Span};
use codemap_diagnostic::{Diagnostic, Level, SpanLabel, SpanStyle};
use either::Either;

use crate::html::{lexer::Token, parser::ElementContent};

mod import_resolver;
mod lexer;
mod parser;
mod replacer_resolver;
mod util;

pub struct Preprocessor<'a> {
    code_map: CodeMap,
    fetcher: Box<dyn FnMut(&str) -> Result<String, String> + 'a>,
}

struct FileContext<'a, 'env> {
    preprocessor: &'a mut Preprocessor<'env>,
    file: &'a Arc<File>,

    // Tell is preprocess need to repeat
    // 1. Replace
    // 2. Handle imports
    // again
    reiterate: bool,
}

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum PreprocessorCodes {
    IterationExceededLimit = 0000,
}

impl PreprocessorCodes {
    pub fn description(&self) -> &'static str {
        match self {
            PreprocessorCodes::IterationExceededLimit => {
                "Limit reached when processing file (too deep of import or runaway replacer + import loop). Preprocessor can't find stable state"
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

        format!("Preprocessor:{letter}{:04}", *self as u16)
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

impl<'a> Preprocessor<'a> {
    pub fn new<F>(file_fetcher: F) -> Self
    where
        F: FnMut(&str) -> Result<String, String> + 'a,
    {
        Self {
            code_map: CodeMap::new(),
            fetcher: Box::new(file_fetcher),
        }
    }

    pub fn get_codemap(&self) -> &CodeMap {
        &self.code_map
    }

    #[expect(unused)]
    fn dump_tokens(&self, file: &File, tokens: &[(Span, Token)]) {
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
                    println!(
                        "Comment('{}')",
                        file.source_slice(comment.content).escape_default()
                    )
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
    }

    fn dump_element(&mut self, file: &File, depth: usize, elements: &[(Span, ElementContent)]) {
        let indent = "  ".repeat(depth);
        for (idx, element) in elements.iter().enumerate() {
            let element_slice = file.source_slice(element.0);
            print!("{indent}[{idx:#3}] = ");

            match &element.1 {
                parser::ElementContent::Comment(comment) => println!(
                    "Comment('{}')",
                    file.source_slice(comment.content).escape_default()
                ),

                parser::ElementContent::Replacer(replacer) => println!(
                    "Replacer('{}', isSimple = {})",
                    file.source_slice(replacer.content).escape_default(),
                    replacer.is_simple
                ),

                parser::ElementContent::Element(element) => {
                    match &element.name {
                        Either::Left(name) => {
                            println!("Element tag name: '{}'", name.escape_default());
                        }

                        Either::Right(replacer) => {
                            println!(
                                "Element tag name: Replacer('{}', is_simple = {})",
                                file.source_slice(replacer.content).escape_default(),
                                replacer.is_simple
                            );
                        }
                    }

                    println!("{indent}  Attribute (count {}):", element.attributes.len());
                    for (idx, attribute) in element.attributes.iter().enumerate() {
                        print!("{indent}    [{idx:#3}] = ");
                        match attribute {
                            parser::Attribute::EmptyAttribute(span) => {
                                println!(
                                    "EmptyAttribute('{}')",
                                    file.source_slice(*span).escape_default()
                                );
                            }
                            parser::Attribute::Attribute(_, data) => {
                                println!(
                                    "Attribute(key = '{}', value = QuotedString('{}', is_double_quote = {}))",
                                    file.source_slice(data.key_span).escape_default(),
                                    file.source_slice(data.value.content).escape_default(),
                                    data.value.is_double_quote
                                );
                            }
                            parser::Attribute::Replacer(_, replacer) => {
                                println!(
                                    "Replacer('{}', is_simple = {})",
                                    file.source_slice(replacer.content),
                                    replacer.is_simple
                                );
                            }
                        }
                    }

                    println!("{indent}  Child (count {}):", element.childs.len());
                    self.dump_element(file, depth + 2, &element.childs);
                }

                parser::ElementContent::Text => {
                    println!("Text('{}')", element_slice.escape_default())
                }
            }
        }
    }

    fn load_file(&mut self, path: &str) -> Result<Arc<File>, String> {
        let source = (self.fetcher)(path)?;
        Ok(self.code_map.add_file(path.to_string(), source))
    }

    pub fn parse_file(&mut self, path: &str) -> Result<(), Vec<Diagnostic>> {
        let file = self.load_file(path).map_err(|err| {
            vec![Diagnostic {
                code: None,
                level: Level::Error,
                message: err,
                spans: Vec::new(),
            }]
        })?;

        let tokens = lexer::run(&file)?;
        let mut tree = parser::run(file.clone(), tokens)?;

        let mut ctx = FileContext {
            file: &file,
            reiterate: true,
            preprocessor: self,
        };

        let max_iter = 512;
        let mut current_iter = 0;
        while ctx.reiterate {
            if current_iter >= max_iter {
                return Err(vec![
                    PreprocessorCodes::IterationExceededLimit.to_diagnostic(&[SpanLabel {
                        label: None,
                        span: util::one_char_span(&file, 0),
                        style: SpanStyle::Primary,
                    }]),
                ]);
            }

            ctx.reiterate = false;

            // Resolve imports
            tree = import_resolver::run(&mut ctx, tree)?;

            // Resolve replacers
            tree = replacer_resolver::run(&mut ctx, tree)?;

            current_iter += 1;
        }

        self.dump_element(&file, 0, &tree);
        Ok(())
    }
}
