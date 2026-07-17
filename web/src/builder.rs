use std::{borrow::Cow, collections::HashMap, str::FromStr};

use chrono::Utc;
use codemap::CodeMap;
use codemap_diagnostic::Diagnostic;
use html_preprocess::Preprocessor;
use infer::Infer;
use mime::Mime;

use crate::{config, util};

mod navbar;

pub enum BuildError {
    PreprocessFailed(&'static str, CodeMap, Vec<Diagnostic>),
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
                .map(|x| str::from_utf8(x.data))?
                .map_err(|e| format!("File '{name}' has invalid UTF8 format: {e}"))
                .map(|x| x.to_string())
        },
        true,
    );

    // Some stuffs for the preprocessor :3 //
    let build_time = Utc::now().to_rfc2822();

    preprocessor.set_env("root", &config.root);
    let mut generators = HashMap::new();
    generators.insert(
        "build-time".to_string(),
        html_preprocess::create_generator(move |_| Ok(format!("<p>Built on {build_time}</p>"))),
    );

    // Relating to navbars
    navbar::init(config, &mut generators);

    /////////////////////////////////////////

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
        let data;
        let mime;

        if resource.do_preprocess {
            let result = preprocessor.process_file(path, &generators);
            data = match result {
                Ok(data) => Cow::<'_, [u8]>::Owned(data.into_bytes()).into(),
                Err(diags) => {
                    return Err(BuildError::PreprocessFailed(
                        path,
                        preprocessor.to_codemap(),
                        diags,
                    ));
                }
            };
            mime = Some(mime::TEXT_HTML_UTF_8);
        } else {
            
            data = Cow::Borrowed(resource.data);
            mime = inferrer.get(resource.data)
                .map(|ty| Mime::from_str(ty.mime_type()).ok())
                .flatten();
        }

        if resource.do_include {
            map.insert(path, (data, mime));
        }
    }

    Ok(map)
}
