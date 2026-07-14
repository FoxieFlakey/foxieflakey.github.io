macro_rules! html {
    ($path:expr) => {
        (
            $path,
            Resource {
                data: &include_bytes!(concat!("../", $path))[..],
                do_preprocess: true,
                do_include: true,
            },
        )
    };
}

macro_rules! raw {
    ($path:expr) => {
        (
            $path,
            Resource {
                data: &include_bytes!(concat!("../", $path))[..],
                do_preprocess: false,
                do_include: true,
            },
        )
    };
}

macro_rules! html_dep {
    ($path:expr) => {
        (
            $path,
            Resource {
                data: &include_bytes!(concat!("../", $path))[..],
                do_preprocess: false,
                do_include: false,
            },
        )
    };
}

pub(crate) use html;
pub(crate) use html_dep;
pub(crate) use raw;
