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

use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use codemap::{CodeMap, File, Span};
use codemap_diagnostic::{Diagnostic, Level, SpanLabel, SpanStyle};
use either::Either;

use crate::html::{lexer::Token, parser::ElementContent};

mod import_resolver;
mod lexer;
mod parser;
mod replacer_resolver;
mod util;
mod template_resolver;
mod encoder;

pub struct Preprocessor<'a> {
    cached_files: HashMap<String, Arc<(Arc<File>, Vec<(Span, ElementContent)>)>>,
    code_map: CodeMap,
    fetcher: Box<dyn FnMut(&str) -> Result<String, String> + 'a>,
    environment: HashMap<String, String>
}

struct FileContext<'a, 'env> {
    preprocessor: &'a mut Preprocessor<'env>,
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
            cached_files: HashMap::new(),
            fetcher: Box::new(file_fetcher),
            environment: HashMap::new()
        }
    }

    // Same return value as HashMap::get
    pub fn get_env<K: AsRef<str> + ?Sized>(&self, key: &K) -> Option<&String> {
        self.environment.get(key.as_ref())
    }

    // Same return value as HashMap::insert
    pub fn set_env<S1: Into<String>, S2: Into<String>>(&mut self, key: S1, value: S2) -> Option<String> {
        self.environment.insert(key.into(), value.into())
    }
    
    // Same return value as HashMap::remove
    pub fn unset_env<S: AsRef<str>>(&mut self, key: S) -> Option<String> {
        self.environment.remove(key.as_ref())
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

    fn resolve_span_to_string(&self, span: Span) -> &str {
        util::resolve_span_to_string(&self.code_map, span)
    }

    // Merely only loads file, parses it and caches it.
    fn load_file(
        &mut self,
        path: &str,
    ) -> Result<Arc<(Arc<File>, Vec<(Span, ElementContent)>)>, Either<Vec<Diagnostic>, String>>
    {
        match self.cached_files.entry(path.to_string()) {
            Entry::Occupied(cached) => return Ok(cached.get().clone()),

            Entry::Vacant(vacant) => {
                // Slow path load the file and parse
                let source = (self.fetcher)(path).map_err(Either::Right)?;
                let file = self.code_map.add_file(path.to_string(), source.to_string());
                let parsed = parser::run(file.clone(), lexer::run(&file).map_err(Either::Left)?)
                    .map_err(Either::Left)?;

                Ok(vacant.insert(Arc::new((file, parsed))).clone())
            }
        }
    }

    pub fn process_file(&mut self, path: &str) -> Result<String, Vec<Diagnostic>> {
        let loaded = self.load_file(path).map_err(|x| match x {
            Either::Left(diag) => diag,
            Either::Right(err) => {
                vec![Diagnostic {
                    code: None,
                    level: Level::Error,
                    message: err,
                    spans: Vec::new(),
                }]
            }
        })?;
        let file = &loaded.0;
        let mut tree = loaded.1.clone();

        let mut ctx = FileContext { preprocessor: self };

        let max_iter = 512;
        let mut current_iter = 0;
        loop {
            if current_iter >= max_iter {
                return Err(vec![
                    PreprocessorCodes::IterationExceededLimit.to_diagnostic(&[SpanLabel {
                        label: None,
                        span: util::one_char_span(&file, 0),
                        style: SpanStyle::Primary,
                    }]),
                ]);
            }

            // Resolve imports
            tree = import_resolver::run(&mut ctx, tree)?;

            // Resolve replacers
            tree = replacer_resolver::run(&mut ctx, tree)?;

            // Resolve templates
            tree = template_resolver::run(&mut ctx, tree)?;

            // Check if next iteration is not needed
            current_iter += 1;
            
            let mut need_reiter = false;
            util::iter_tree_mut(&mut tree, |element| {
                match &element.1 {
                    ElementContent::Replacer(_) => {
                        need_reiter = true;
                        false
                    }
                    
                    ElementContent::Element(elem) => {
                        if let Either::Right(a) = &elem.name {
                            let _: &lexer::Replacer = a;
                            need_reiter = true;
                            false
                        } else {
                            true
                        }
                    }
                    
                    _ => true
                }
            });
            
            if !need_reiter {
                // Nothing changes, quit iterating
                break;
            }
        }

        let mut buf = Vec::new();
        encoder::encode(&mut ctx, &mut buf, &tree)
            .map_err(|e| {
                let error_span = file.span.subspan(0, util::inc_char_offset(0, file.source()));
                let msg = match e {
                    encoder::EncodeError::IO(e) => {
                        format!("Cannot write to output: {e}")
                    }
                    
                    encoder::EncodeError::Message(e) => {
                        format!("Encoder met an error: {e}")
                    }
                };
                
                return vec![
                    Diagnostic {
                        code: None,
                        level: Level::Error,
                        message: msg,
                        spans: vec![
                            SpanLabel {
                                label: None,
                                span: error_span,
                                style: SpanStyle::Primary
                            }
                        ]
                    }
                ]
            })?;
        Ok(String::from_utf8(buf).expect("HTML encoder written non valid UTF-8 bytes!"))
    }
}
