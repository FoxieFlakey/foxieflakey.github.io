use std::{collections::HashMap, sync::LazyLock};

use crate::macros::{html, html_dep, raw};

pub static RESOURCES: LazyLock<HashMap<&str, Resource>> = LazyLock::new(|| {
    let entries = [
        html!("/index.html"),
        html_dep!("/components/page.html"),
        raw!("/img/profile.gif"),
        raw!("/css/global.css"),
        raw!("/css/pages/home.css"),
        raw!("/favicon.ico"),
        raw!("/favicon_for_opengraph.png"),
        raw!("/img/Gallery_Icon.png"),
        raw!("/img/Home_Icon.png"),
    ];

    let mut map = HashMap::new();
    map.reserve(entries.len());

    for (path, resource) in entries {
        map.insert(path, resource);
    }

    map
});

pub struct Config {
    pub root: String,
}

pub struct Resource {
    pub data: &'static [u8],

    // Whether this resource be preprocessed
    pub do_preprocess: bool,

    // Whether this resource be included in
    // final output
    pub do_include: bool,
}
