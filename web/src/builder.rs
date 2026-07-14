use std::{borrow::Cow, collections::HashMap, str::FromStr};

use codemap::CodeMap;
use codemap_diagnostic::Diagnostic;
use html_preprocess::Preprocessor;
use mime::Mime;

use crate::{config, util};

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

    preprocessor.set_env("root", &config.root);

    /////////////////////////////////////////

    for (&path, resource) in config::RESOURCES.iter() {
        let data;
        let mime;

        if resource.do_preprocess {
            let result = preprocessor.process_file(path);
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
            mime = infer::get(resource.data)
                .map(|ty| Mime::from_str(ty.mime_type()).ok())
                .flatten();
        }

        if resource.do_include {
            map.insert(path, (data, mime));
        }
    }

    Ok(map)
}
