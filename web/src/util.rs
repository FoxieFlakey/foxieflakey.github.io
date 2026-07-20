use std::{borrow::Cow, str::FromStr, sync::LazyLock, time::Duration};

use infer::Infer;
use mime::Mime;

pub fn sanify_path(path: &str) -> String {
    let mut segments = Vec::new();

    for segment in path.split('/') {
        if segment == "." {
            continue;
        } else if segment == ".." {
            segments.pop();
        } else if !segment.is_empty() {
            segments.push(segment);
        }
    }

    let joined = segments.join("/");
    format!(
        "{}{}{}",
        if joined.starts_with('/') { "" } else { "/" },
        joined,
        if path.ends_with('/') && joined.len() > 1 {
            "/"
        } else {
            ""
        }
    )
}

pub fn round_duration_to_ms(duration: Duration) -> Duration {
    Duration::new(duration.as_secs(), duration.subsec_millis() * 1_000_000)
}

pub trait ExpectNone {
    fn expect_none(self, msg: &str);
}

impl<T> ExpectNone for Option<T> {
    #[track_caller]
    fn expect_none(self, msg: &str) {
        if self.is_some() {
            panic!("Expecting None got Some: {msg}")
        }
    }
}

pub fn infer(filename: Option<&str>, data: &[u8]) -> Option<mime::Mime> {
    INFERRER
        .get(data)
        .map(|ty| {
            Mime::from_str(ty.mime_type())
                .inspect_err(|e| {
                    println!("[WARN] mime crate cannot parse mime from infer crate (the Mime was '{}'): {e}", ty.mime_type().escape_default());
                })
                .ok()
        })
        .flatten()
        .or_else(|| {
            filename.map(|name| {
                println!("[WARN] Cannot infer based on file content. Falling back to extension detection for '{}'", name.escape_default());
                mime_guess::from_path(name)
                    .first()
            })
            .flatten()
        })
}

static INFERRER: LazyLock<Infer> = LazyLock::new(|| {
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

    inferrer
});

pub fn encode_html<S>(text: &S) -> Cow<'_, str>
where
    S: ?Sized + AsRef<str>
{
    Cow::Owned(html_escape::encode_safe(text).replace('$', "&dollar;"))
}
