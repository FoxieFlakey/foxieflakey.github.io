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
    borrow::Cow,
    collections::{HashMap, hash_map::Entry},
    panic::Location,
    path::Path,
    rc::Rc,
    sync::Arc,
};

use codemap::{CodeMap, File, Span};
use codemap_diagnostic::{Diagnostic, Level, SpanLabel, SpanStyle};
use either::Either;

use crate::html::parser::ElementContent;

mod encoder;
mod import_resolver;
pub mod lexer;
pub mod parser;
mod replacer_resolver;
mod template_resolver;
mod util;

pub struct Preprocessor<'a> {
    cached_files: HashMap<String, Arc<(Arc<File>, Vec<(Span, ElementContent)>)>>,
    code_map: CodeMap,
    fetcher: Box<dyn FnMut(&str) -> Result<String, String> + 'a>,
    environment: HashMap<String, String>,
    minify: bool,
}

pub struct GeneratorArgs<'a> {
    pub childs: &'a Vec<(Span, parser::ElementContent)>,
    pub attributes: &'a Vec<parser::Attribute>,
}

pub enum Template<'a> {
    FromSource(Span, Cow<'a, Vec<(Span, parser::ElementContent)>>),
    Generator(
        // Where the generator defined
        &'static Location<'static>,
        // Return a string of encoded HTML. Yes this is somewhat inefficient
        // but saner than reconstructing the AST tree manually in code. and
        // verifying data from there is not bogus/invalid
        Rc<dyn Fn(GeneratorArgs) -> Result<String, String> + 'static>,
    ),
}

struct FileContext<'a, 'env> {
    preprocessor: &'a mut Preprocessor<'env>,
    known_templates: HashMap<String, Template<'static>>,
}

impl<'env> FileContext<'_, 'env> {
    // Same return value as HashMap::get
    pub fn get_env<K: AsRef<str> + ?Sized>(&self, key: &K) -> Option<&String> {
        self.preprocessor.get_env(key)
    }

    pub fn find_src_file(&self, span: Span) -> &Arc<File> {
        self.preprocessor.code_map.find_file(span.low())
    }

    pub fn import_file(
        &mut self,
        importer: &File,
        path: &str,
    ) -> Result<Arc<(Arc<File>, Vec<(Span, ElementContent)>)>, Either<Vec<Diagnostic>, String>>
    {
        if path.starts_with('/') {
            self.preprocessor.load_file(path)
        } else {
            // Special: code got from generator. It does not have
            // path for relative imports
            if importer.name().starts_with("generator:") {
                return Err(Either::Right(format!(
                    "Generated code ('{}'), cannot relatively import",
                    importer.name()
                )));
            }

            let importer_dir = Path::new(importer.name())
                .parent()
                .unwrap()
                .to_str()
                .unwrap();
            self.preprocessor
                .load_file(&format!("./{importer_dir}/{path}"))
        }
    }

    pub fn resolve_span_to_string(&self, span: Span) -> &str {
        self.preprocessor.resolve_span_to_string(span)
    }
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
    pub fn new<F>(file_fetcher: F, can_minify: bool) -> Self
    where
        F: FnMut(&str) -> Result<String, String> + 'a,
    {
        Self {
            code_map: CodeMap::new(),
            cached_files: HashMap::new(),
            fetcher: Box::new(file_fetcher),
            environment: HashMap::new(),
            minify: can_minify,
        }
    }

    // Same return value as HashMap::get
    pub fn get_env<K: AsRef<str> + ?Sized>(&self, key: &K) -> Option<&String> {
        self.environment.get(key.as_ref())
    }

    // Same return value as HashMap::insert
    pub fn set_env<S1: Into<String>, S2: Into<String>>(
        &mut self,
        key: S1,
        value: S2,
    ) -> Option<String> {
        self.environment.insert(key.into(), value.into())
    }

    // Same return value as HashMap::remove
    pub fn unset_env<S: AsRef<str>>(&mut self, key: S) -> Option<String> {
        self.environment.remove(key.as_ref())
    }

    pub fn get_codemap(&self) -> &CodeMap {
        &self.code_map
    }

    fn resolve_span_to_string(&self, span: Span) -> &str {
        util::resolve_span_to_string(&self.code_map, span)
    }

    // Parses the src code, and return AST
    // does not cache it
    fn add_generated_code(
        &mut self,
        location: &Location,
        src: String,
    ) -> Result<Vec<(Span, ElementContent)>, Vec<Diagnostic>> {
        let file = self.code_map.add_file(
            format!(
                "generator:{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            ),
            src,
        );
        Ok(parser::run(file.clone(), lexer::run(&file)?)?)
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

    pub fn process_file(
        &mut self,
        path: &str,
        generators: &HashMap<
            String,
            (
                &'static Location<'static>,
                Rc<dyn Fn(GeneratorArgs) -> Result<String, String> + 'static>,
            ),
        >,
    ) -> Result<String, Vec<Diagnostic>> {
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

        let mut ctx = FileContext {
            preprocessor: self,
            known_templates: HashMap::new(),
        };

        for (key, (location, func)) in generators {
            ctx.known_templates
                .insert(format!("x-{key}"), Template::Generator(location, func.clone()));
        }

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
            util::iter_tree_mut(&mut tree, |element| match &element.1 {
                ElementContent::Replacer(_) => {
                    need_reiter = true;
                    false
                }

                ElementContent::Element(elem) => match &elem.name {
                    Either::Left(name) => {
                        if name.starts_with("x-") {
                            need_reiter = true;
                            false
                        } else {
                            true
                        }
                    }
                    Either::Right(_) => {
                        need_reiter = true;
                        false
                    }
                },

                _ => true,
            });

            if !need_reiter {
                // Nothing changes, quit iterating
                break;
            }
        }

        let mut buf = Vec::new();
        encoder::encode(&mut ctx, &mut buf, &tree).map_err(|e| {
            let msg = match e {
                encoder::EncodeError::IO(e) => {
                    format!("Cannot write to output: {e}")
                }

                encoder::EncodeError::Message(e) => {
                    format!("Encoder met an error: {e}")
                }
            };

            return vec![Diagnostic {
                code: None,
                level: Level::Error,
                message: msg,
                spans: vec![],
            }];
        })?;

        if self.minify {
            let cfg = simple_minify_html::Cfg {
                keep_html_and_head_opening_tags: true,
                ..Default::default()
            };

            let input = &buf;
            str::from_utf8(input).expect("HTML encoder written non valid UTF-8 bytes!");

            let output = simple_minify_html::minify(&input, Some(cfg));
            return Ok(
                String::from_utf8(output).expect("HTML minifier written non valid UTF-8 bytes!")
            );
        }

        Ok(String::from_utf8(buf).expect("HTML encoder written non valid UTF-8 bytes!"))
    }

    // This one way convert the Preprocess into CodeMap for diagnostics
    pub fn to_codemap(self) -> CodeMap {
        self.code_map
    }
}
