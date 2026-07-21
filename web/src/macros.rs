macro_rules! html {
    ($path:expr) => {
        (
            $path,
            Resource::PreprocessAndIncludeHtml(Cow::Borrowed(
                &include_bytes!(concat!("../data/", $path))[..],
            )),
        )
    };
}

macro_rules! css {
    ($path:expr) => {
        (
            $path,
            Resource::Css(Cow::Borrowed(
                &include_bytes!(concat!("../data/", $path))[..],
            )),
        )
    };
}

macro_rules! raw {
    ($path:expr) => {
        (
            $path,
            Resource::RawBytes(Cow::Borrowed(
                &include_bytes!(concat!("../data/", $path))[..],
            )),
        )
    };
}

macro_rules! html_dep {
    ($path:expr) => {
        (
            $path,
            Resource::HtmlBuildResource(Cow::Borrowed(
                &include_bytes!(concat!("../data/", $path))[..],
            )),
        )
    };
}

pub(crate) use css;
pub(crate) use html;
pub(crate) use html_dep;
pub(crate) use raw;
