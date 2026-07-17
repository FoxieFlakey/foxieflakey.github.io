use std::{borrow::Cow, collections::HashMap, panic::Location, rc::Rc, str::FromStr};

use chrono::Utc;
use codemap::CodeMap;
use codemap_diagnostic::Diagnostic;
use html_preprocess::{GeneratorArgs, Preprocessor};
use infer::Infer;
use mime::Mime;

use crate::{config, util};

mod navbar;

pub enum BuildError {
    PreprocessFailed(&'static str, CodeMap, Vec<Diagnostic>),
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
}

pub fn build(
    config: &config::Config,
) -> Result<HashMap<&'static str, (Cow<'static, [u8]>, Option<Mime>)>, BuildError> {
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

    let mut inferrer = Infer::new();
    inferrer.add("image/svg+xml", "svg", |buf| {
        // From https://github.com/bojand/infer/pull/119
        // hasnt been merged yet
        pub fn is_svg(buf: &[u8]) -> bool {
            if buf.starts_with(b"<svg") {
                return true;
            }

            // Avoid conflicts with other XML types while detecting SVGs.
            if buf.starts_with(b"<?xml") {
                return buf
                    .get(..256)
                    .unwrap_or(buf)
                    .windows(4)
                    .any(|w| w == b"<svg");
            }

            false
        }

        is_svg(buf)
    });

    for (&path, resource) in config::RESOURCES.iter() {
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

                let result = preprocessor.process_file(path, &generators);
                let data = match result {
                    Ok(data) => Cow::<'_, [u8]>::Owned(data.into_bytes()),
                    Err(diags) => {
                        return Err(BuildError::PreprocessFailed(
                            path,
                            preprocessor.to_codemap(),
                            diags,
                        ));
                    }
                };
                Some((Some(mime::TEXT_HTML_UTF_8), data))
            }
            
            config::Resource::RawBytes(data) => {
                let mime = inferrer
                    .get(data)
                    .map(|ty| Mime::from_str(ty.mime_type()).ok())
                    .flatten();
                Some((mime, Cow::Borrowed(data)))
            }
            
            // The HTML is dependency needed by other preprocessable html
            config::Resource::HtmlBuildResource(_) => None
        };

        if let Some((mime, data)) = resource_result {
            map.insert(path, (data, mime));
        }
    }

    Ok(map)
}
