use std::{borrow::Cow, collections::HashMap, panic::Location, rc::Rc, str::Utf8Error, sync::{Arc, RwLock}};

use chrono::Utc;
use codemap::CodeMap;
use codemap_diagnostic::Diagnostic;
use html_preprocess::{GeneratorArgs, Preprocessor};
use lightningcss::{error::{MinifyErrorKind, ParserError, PrinterErrorKind}, printer::PrinterOptions, stylesheet::{MinifyOptions, ParserOptions, StyleSheet}};
use mime::Mime;

use crate::{config, util};

mod navbar;
mod arts;

pub enum BuildError {
    PreprocessFailed(String, CodeMap, Vec<Diagnostic>),
    LoadCSSNonUtf8(String, Utf8Error),
    ParseCSSFailed(String, lightningcss::error::Error<ParserError<'static>>),
    MinifyCSSFailed(String, lightningcss::error::Error<MinifyErrorKind>),
    EncodeCSSFailed(String, lightningcss::error::Error<PrinterErrorKind>),
}

fn init_generators(
    config: &config::Config,
    generators: &mut HashMap<
        String,
        (
            &Location<'_>,
            Rc<dyn Fn(GeneratorArgs<'_>) -> Result<String, String>>,
        ),
    >,
) {
    // Relating to navbars
    navbar::init(config, generators);
    arts::init(config, generators);
}

pub fn build(
    config: &config::Config,
) -> Result<HashMap<String, (Cow<'static, [u8]>, Option<Mime>)>, BuildError> {
    let mut map = HashMap::new();

    let mut preprocessor = Preprocessor::new(
        |name| {
            config::RESOURCES
                .get(util::sanify_path(name).as_str())
                .ok_or(())
                .map_err(|_| format!("Cannot find file '{name}'"))
                .map(|x| str::from_utf8(x.data()))?
                .map_err(|e| format!("File '{name}' has invalid UTF8 format: {e}"))
                .map(|x| x.to_string())
        },
        true,
    );

    let build_time = Utc::now().to_rfc2822();
    preprocessor.set_env("root", &config.root);
    let mut generators = HashMap::new();
    let build_time2 = build_time.clone();
    generators.insert(
        "build-time".to_string(),
        html_preprocess::create_generator(move |_| Ok(format!("<p>Built on {build_time2}</p>"))),
    );
    init_generators(config, &mut generators);

    for (path, resource) in config::RESOURCES.iter() {
        let resource_result: Option<(Option<Mime>, Cow<'static, [u8]>)> = match resource {
            config::Resource::PreprocessAndIncludeHtml(_) => {
                // Each instance of generator, store state
                // relating to each file
                generators.clear();
                let build_time = build_time.clone();
                generators.insert(
                    "build-time".to_string(),
                    html_preprocess::create_generator(move |_| {
                        Ok(format!("<p>Built on {build_time}</p>"))
                    }),
                );
                init_generators(config, &mut generators);

                let result = preprocessor.process_file(&path, &generators);
                let data = match result {
                    Ok(data) => Cow::<'_, [u8]>::Owned(data.into_bytes()),
                    Err(diags) => {
                        return Err(BuildError::PreprocessFailed(
                            path.clone(),
                            preprocessor.to_codemap(),
                            diags,
                        ));
                    }
                };
                Some((Some(mime::TEXT_HTML_UTF_8), data))
            }
            
            config::Resource::Css(data) => {
                let source = str::from_utf8(data)
                    .map_err(|e| BuildError::LoadCSSNonUtf8(path.clone(), e))?;
                
                let warnings = Arc::new(RwLock::new(Vec::new()));
                
                // Parse CSS first
                let mut css = StyleSheet::parse(source, ParserOptions {
                    filename: path.to_string(),
                    warnings: Some(warnings.clone()),
                    ..Default::default()
                }).map_err(|e| BuildError::ParseCSSFailed(path.clone(), e))?;
                
                // Minify CSS
                css.minify(MinifyOptions::default())
                    .map_err(|e| BuildError::MinifyCSSFailed(path.clone(), e))?;
                
                // Encode back to CSS code
                let minified = css.to_css(PrinterOptions {
                    minify: true,
                    ..Default::default()
                }).map_err(|e| BuildError::EncodeCSSFailed(path.clone(), e))?;
                Some((Some(mime::TEXT_CSS_UTF_8), Cow::Owned(minified.code.into_bytes())))
            }
            
            config::Resource::RawBytes(data) => {
                Some((util::infer(data), Cow::Borrowed(data)))
            }
            
            // The HTML is dependency needed by other preprocessable html
            config::Resource::HtmlBuildResource(_) => None
        };

        if let Some((mime, data)) = resource_result {
            map.insert(path.clone(), (data, mime));
        }
    }

    Ok(map)
}
