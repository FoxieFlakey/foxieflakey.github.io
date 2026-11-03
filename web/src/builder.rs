use std::{borrow::Cow, collections::HashMap, panic::Location, sync::Arc};

use chrono::Utc;
use codemap::CodeMap;
use codemap_diagnostic::Diagnostic;
use html_preprocess::{GeneratorArgs, Preprocessor};
use mime::Mime;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    builder::wasm::Wasm,
    config::{self, ResourceType},
    util,
};

#[cfg(feature = "minify")]
use std::str::Utf8Error;

#[cfg(feature = "minify")]
use lightningcss::{
    error::{MinifyErrorKind, ParserError, PrinterErrorKind},
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
};

mod arts;
mod navbar;
mod wasm;

pub struct Builder<'fetcher> {
    config: config::Config,
    state: RwLock<State<'fetcher>>,
    build_time: String,
}

struct State<'fetcher> {
    cached: HashMap<String, Arc<(Cow<'static, [u8]>, Option<Mime>)>>,
    fetcher: Arc<
        RwLock<
            Option<
                Box<
                    dyn Fn(&str) -> Result<Option<Cow<'static, [u8]>>, String>
                        + Send
                        + Sync
                        + 'fetcher,
                >,
            >,
        >,
    >,
    preprocessor: Preprocessor<'fetcher>,
    wasm_builder: Wasm,
}

pub enum BuildError<'resources> {
    PreprocessFailed(
        String,
        MappedRwLockReadGuard<'resources, CodeMap>,
        Vec<Diagnostic>,
    ),
    CannotFindResource(String),
    CannotFetchResource(String, String),

    #[cfg(feature = "minify")]
    LoadCSSNonUtf8(String, Utf8Error),

    #[cfg(feature = "minify")]
    ParseCSSFailed(String, String),

    #[cfg(feature = "minify")]
    MinifyCSSFailed(String, lightningcss::error::Error<MinifyErrorKind>),

    #[cfg(feature = "minify")]
    EncodeCSSFailed(String, lightningcss::error::Error<PrinterErrorKind>),
}

impl State<'_> {
    fn call_fetcher(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>, String> {
        match self.fetcher.read().as_ref() {
            Some(fetcher) => fetcher(path),
            None => Err(format!("There no fetcher installed")),
        }
    }
}

impl<'fetcher> Builder<'fetcher> {
    pub fn new(config: config::Config) -> Self {
        let fetcher: Arc<
            parking_lot::lock_api::RwLock<
                parking_lot::RawRwLock,
                Option<
                    Box<dyn Fn(&str) -> Result<Option<Cow<'static, [u8]>>, String> + Send + Sync>,
                >,
            >,
        > = Arc::new(RwLock::new(None));
        let fetcher_cloned = fetcher.clone();

        let mut preprocessor = Preprocessor::new(
            move |path| {
                fetcher_cloned
                    .read()
                    .as_ref()
                    .map(|x| x(path))
                    .ok_or(format!("There no fetcher installed"))
                    .flatten()
                    .map_err(|e| format!("Cannot fetch file '{path}': {e}"))
                    .map(|x| x.ok_or_else(|| format!("Cannot find '{path}'")))?
                    .map(|x| String::from_utf8(x.to_vec()))?
                    .map_err(|e| format!("File '{path}' has invalid UTF8 format: {e}"))
            },
            // TODO: Minify but not extreme. Social media bot crawler, struggle to find
            // or parse the heavily minified HTML.
            false,
        );
        preprocessor.set_env("root", &config.root);

        Self {
            config,
            build_time: Utc::now().to_rfc2822(),
            state: RwLock::new(State {
                cached: HashMap::new(),
                fetcher: fetcher.clone(),
                wasm_builder: Wasm::new(),
                preprocessor,
            }),
        }
    }

    pub fn set_fetcher<F>(
        &self,
        fetcher: F,
    ) -> Option<
        Box<dyn Fn(&str) -> Result<Option<Cow<'static, [u8]>>, String> + Send + Sync + 'fetcher>,
    >
    where
        F: Fn(&str) -> Result<Option<Cow<'static, [u8]>>, String> + Send + Sync + 'fetcher,
    {
        self.state
            .write()
            .fetcher
            .write()
            .replace(Box::new(fetcher))
    }

    pub fn take_fetcher(
        &self,
    ) -> Option<
        Box<dyn Fn(&str) -> Result<Option<Cow<'static, [u8]>>, String> + Send + Sync + 'fetcher>,
    > {
        self.state.write().fetcher.write().take()
    }

    pub fn build_all<'this>(&'this self) -> Result<(), BuildError<'this>> {
        for path in config::RESOURCES.keys() {
            self.get(path)?;
        }
        Ok(())
    }

    pub fn clear_cache(&self) {
        let mut state = self.state.write();
        state.cached.clear();
        state.preprocessor.clear_cache();
        state.wasm_builder.always_fetch_fresh();
    }

    // .0 is web data dir and .1 is build dir (for miscellanous resources)
    pub fn set_dirs(&self, dir: Option<(&str, &str)>) {
        self.state.write().wasm_builder.set_dirs(dir);
    }

    pub fn iter_data(
        &mut self,
    ) -> impl Iterator<Item = (&String, &Cow<'static, [u8]>, &Option<Mime>)> {
        self.state
            .get_mut()
            .cached
            .iter()
            .map(|x| (x.0, &x.1.0, &x.1.1))
    }

    pub fn get<'this>(
        &'this self,
        path: &str,
    ) -> Result<Arc<(Cow<'static, [u8]>, Option<Mime>)>, BuildError<'this>> {
        let path = common_utils::sanify_path(path);

        // Check if its in cache
        if let Some(cached) = self.state.read().cached.get(&path) {
            // Cache hit
            return Ok(cached.clone());
        }

        // Try find hint in global list
        let hint = config::RESOURCES.get(&path).map(|x| x.type_());
        let state = self.state.write();

        let Some(data) = (match hint {
            Some(ResourceType::WasmRust) | Some(ResourceType::WasmRustLoader) => {
                // WasmRust has no "data" attached to it, so fetcher will fails
                // because it cant fetch data. So ask wasm_builder to do it
                // NOTE: This bypasses 'fetcher' mechanism. Caller will handle
                // wasm_builder seperately

                match state.wasm_builder.get(&path) {
                    Ok(x) => x,
                    Err(e) => return Err(BuildError::CannotFetchResource(path, e)),
                }
            }

            None
            | Some(
                ResourceType::Css
                | ResourceType::HtmlBuildResource
                | ResourceType::PreprocessAndIncludeHtml
                | ResourceType::RawBytes,
            ) => {
                // These resources has the data embededded directly onto 'Resource'
                match state.call_fetcher(&path) {
                    Ok(x) => x,
                    Err(e) => return Err(BuildError::CannotFetchResource(path, e)),
                }
            }
        }) else {
            assert!(
                hint.is_none(),
                "Global resource table has the path, it cannot be gone for {path}"
            );
            return Err(BuildError::CannotFindResource(path));
        };

        // Check if its in cache again under exclusive lock, just incase someone
        // cached it while we're waiting for write lock
        if let Some(cached) = state.cached.get(&path) {
            // Cache hit
            return Ok(cached.clone());
        }

        let (mime, result, mut state) = self.get_one(&path, state, data, hint)?;

        let ret = Arc::new((result, mime));
        state.cached.insert(path, ret.clone());
        Ok(ret)
    }

    fn get_one<'this, 'data>(
        &'this self,
        path: &str,
        mut state: RwLockWriteGuard<'this, State<'fetcher>>,
        data: Cow<'data, [u8]>,
        hint: Option<ResourceType>,
    ) -> Result<
        (
            Option<Mime>,
            Cow<'data, [u8]>,
            RwLockWriteGuard<'this, State<'fetcher>>,
        ),
        BuildError<'this>,
    > {
        let inferred;
        let ty;
        match hint {
            Some(res_type) => match res_type {
                ResourceType::RawBytes => {
                    inferred = util::infer(Some(path), &data);
                    match &inferred {
                        Some(mime) => {
                            if mime == &mime::TEXT_HTML {
                                ty = ResourceType::PreprocessAndIncludeHtml
                            } else if mime == &mime::TEXT_CSS {
                                ty = ResourceType::Css
                            } else {
                                ty = ResourceType::RawBytes
                            }
                        }
                        None => {
                            println!("[WARN] Cannot infer mime type for '{path}'");
                            ty = ResourceType::RawBytes;
                        }
                    }
                }
                ResourceType::HtmlBuildResource => {
                    inferred = Some(mime::TEXT_HTML);
                    ty = res_type;
                }
                ResourceType::PreprocessAndIncludeHtml => {
                    inferred = Some(mime::TEXT_HTML);
                    ty = res_type;
                }
                ResourceType::Css => {
                    inferred = Some(mime::TEXT_CSS);
                    ty = res_type;
                }
                ResourceType::WasmRust => {
                    inferred = Some("application/wasm".parse().unwrap());
                    ty = res_type;
                }
                ResourceType::WasmRustLoader => {
                    inferred = Some(mime::TEXT_JAVASCRIPT);
                    ty = res_type;
                }
            },
            None => {
                inferred = util::infer(Some(path), &data);
                match &inferred {
                    Some(mime) => {
                        if mime == &mime::TEXT_HTML {
                            ty = ResourceType::PreprocessAndIncludeHtml
                        } else if mime == &mime::TEXT_CSS {
                            ty = ResourceType::Css
                        } else {
                            ty = ResourceType::RawBytes
                        }
                    }
                    None => {
                        println!("[WARN] Cannot infer mime type for '{path}'");
                        ty = ResourceType::RawBytes;
                    }
                }
            }
        };

        match ty {
            ResourceType::WasmRust
            | ResourceType::WasmRustLoader
            | ResourceType::HtmlBuildResource
            | ResourceType::RawBytes => Ok((inferred, data, state)),
            ResourceType::PreprocessAndIncludeHtml => {
                let mut generators = HashMap::new();
                self.init_generators(&mut generators);

                let result = state.preprocessor.process_file(&path, &generators);
                let data = match result {
                    Ok(data) => Cow::<'_, [u8]>::Owned(data.into_bytes()),
                    Err(diags) => {
                        return Err(BuildError::PreprocessFailed(
                            path.to_string(),
                            RwLockReadGuard::map(RwLockWriteGuard::downgrade(state), |x| {
                                x.preprocessor.get_codemap()
                            }),
                            diags,
                        ));
                    }
                };
                Ok((inferred, data, state))
            }

            #[cfg(not(feature = "minify"))]
            ResourceType::Css => Ok((inferred, data, state)),

            #[cfg(feature = "minify")]
            ResourceType::Css => {
                let source = str::from_utf8(&data)
                    .map_err(|e| BuildError::LoadCSSNonUtf8(path.to_string(), e))?;

                let warnings = Arc::new(std::sync::RwLock::new(Vec::new()));

                // Parse CSS first
                let mut css = StyleSheet::parse(
                    source,
                    ParserOptions {
                        filename: path.to_string(),
                        warnings: Some(warnings.clone()),
                        ..Default::default()
                    },
                )
                .map_err(|e| BuildError::ParseCSSFailed(path.to_string(), e.to_string()))?;

                // Minify CSS
                css.minify(MinifyOptions::default())
                    .map_err(|e| BuildError::MinifyCSSFailed(path.to_string(), e))?;

                // Encode back to CSS code
                let minified = css
                    .to_css(PrinterOptions {
                        minify: true,
                        ..Default::default()
                    })
                    .map_err(|e| BuildError::EncodeCSSFailed(path.to_string(), e))?;
                Ok((
                    Some(mime::TEXT_CSS_UTF_8),
                    Cow::Owned(minified.code.into_bytes()),
                    state,
                ))
            }
        }
    }

    fn init_generators(
        &self,
        generators: &mut HashMap<
            String,
            (
                &Location<'_>,
                Arc<dyn Fn(GeneratorArgs<'_>) -> Result<String, String>>,
            ),
        >,
    ) {
        let build_time = self.build_time.clone();
        generators.insert(
            "build-time".to_string(),
            html_preprocess::create_generator(move |_| Ok(format!("<p>Built on {build_time}</p>"))),
        );
        // Relating to navbars
        navbar::init(&self.config, generators);
        arts::init(&self.config, generators);
    }
}
