macro_rules! html {
    ($path:expr) => {
        (
            $path,
            Resource::PreprocessAndIncludeHtml(&include_bytes!(concat!("../data/", $path))[..]),
        )
    };
}

macro_rules! css {
    ($path:expr) => {
        (
            $path,
            Resource::Css(&include_bytes!(concat!("../data/", $path))[..]),
        )
    };
}

macro_rules! raw {
    ($path:expr) => {
        (
            $path,
            Resource::RawBytes(&include_bytes!(concat!("../data/", $path))[..]),
        )
    };
}

macro_rules! html_dep {
    ($path:expr) => {
        (
            $path,
            Resource::HtmlBuildResource(&include_bytes!(concat!("../data/", $path))[..]),
        )
    };
}

pub(crate) use css;
pub(crate) use html;
pub(crate) use html_dep;
pub(crate) use raw;
