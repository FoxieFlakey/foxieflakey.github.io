use std::{borrow::Cow, collections::HashMap, sync::LazyLock};

use crate::macros::{css, html, html_dep, raw, wasm};

pub mod arts;

// NOTE: There is no "raw" type here
// opaque wasm blob can be use raw! macro
#[allow(unused)]
#[derive(Clone, Copy)]
pub enum WasmType {
    Rust,
}

mod wasm_blobs {
    use super::WasmType;
    use std::{collections::HashMap, sync::LazyLock};

    include!(concat!(env!("OUT_DIR"), "/wasm_blobs.rs"));
}

pub use wasm_blobs::INITIAL_WASM_BLOBS;

pub static RESOURCES: LazyLock<HashMap<String, Resource>> = LazyLock::new(|| {
    let entries = [
        // Do not remove this comment, its for scanning stuffs
        html!("/index.html"),
        html!("/404.html"),
        html_dep!("/components/page.html"),
        raw!("/img/profile.gif"),
        css!("/css/global.css"),
        css!("/css/pages/home.css"),
        css!("/css/pages/arts.css"),
        raw!("/favicon.ico"),
        raw!("/favicon.png"),
        raw!("/favicon_for_opengraph.png"),
        raw!("/img/Gallery_Icon.png"),
        raw!("/img/Home_Icon.png"),
        html!("/creations/index.html"),
        html!("/creations/RPGGameCollege/index.html"),
        raw!("/creations/RPGGameCollege/rpg-college-0.1.0.jar"),
        html!("/creations/InteraksiManusiaKomputer_ServerDashboard/index.html"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/favicon.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/red_button.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/green_button.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/save_icon.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/plus_icon.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/checkmark_inactive.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/checkmark_active.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/foxie_icon.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/question_icon.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/x_icon.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/ServiceStatus_Warning.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerRack.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerRack_Warning.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerRack_Failed.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerCluster_Warning.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerCluster_Failed.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/ServerCluster.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/ServiceStatus_Failed.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/CPU_Chart.svg"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/Network_Chart.svg"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/Restart.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/Shutdown.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/AdminPfp.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/Edit.png"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/img/Placeholder.png"),
        css!("/creations/InteraksiManusiaKomputer_ServerDashboard/css/global.css"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/css/font.ttf"),
        html!("/creations/CSS_animation/index.html"),
        css!("/creations/CSS_animation/popup.css"),
        raw!("/creations/CSS_animation/left_arrow.png"),
        raw!("/creations/CSS_animation/right_arrow.png"),
        css!("/css/popup.css"),
        html_dep!("/components/popup.html"),
        html_dep!("/creations/popup.html"),
        raw!("/img/popup/left_arrow.png"),
        raw!("/img/popup/right_arrow.png"),
        html!("/arts/index.html"),
        html_dep!("/components/opengraph.html"),
        raw!("/googlece00914e0cd14977.html"),
        raw!("/robots.txt"),
        wasm!("/creations/wasm/hello_world"),
        html!("/creations/wasm/index.html"),
        wasm!("/creations/college_sap/saps_emulators"),
        html!("/creations/college_sap/index.html"),
        // Do not remove this comment too, its for scanning stuffs
    ];

    let mut map = HashMap::new();
    map.reserve(entries.len());

    let iter = entries
        .into_iter()
        .map(|x| (x.0.to_string(), x.1))
        .chain(arts::gen_resources_list().into_iter());

    for (path, resource) in iter {
        match resource {
            Resource::RawBytes(_)
            | Resource::HtmlBuildResource(_)
            | Resource::PreprocessAndIncludeHtml(_)
            | Resource::Css(_) => {
                map.insert(path, resource);
            }
            Resource::WasmRust => {
                // the WasmRustLoader is second resource needed to so wasm can properly
                // loaded this uses wasm-bindgen
                map.insert(format!("{}_bg.wasm", path), resource);
                map.insert(format!("{}.js", path), Resource::WasmRustLoader);
            }
            Resource::WasmRustLoader => panic!("DO NOT manually put wasm rust loader manually"),
        }
    }

    map
});

pub const FAVICON_ALT_TEXT: &'static str = "Pixel art picture of a fox named Foxie with right eye colored brown and left colored blue. She wears a blue collar, and red bow on left ear. With heart emote on floating top of the head.";

pub struct Config {
    pub root: String,
}

pub enum Resource<'bytes> {
    RawBytes(Cow<'bytes, [u8]>),
    HtmlBuildResource(Cow<'bytes, [u8]>),
    PreprocessAndIncludeHtml(Cow<'bytes, [u8]>),
    Css(Cow<'bytes, [u8]>),

    // A wasm component
    WasmRust,

    // It is special .js loader for wasm. The companion
    // .js wasm-bindgen generates
    WasmRustLoader,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ResourceType {
    RawBytes,
    HtmlBuildResource,
    PreprocessAndIncludeHtml,
    Css,
    WasmRust,
    WasmRustLoader,
}

impl<'data> Resource<'data> {
    pub fn type_(&self) -> ResourceType {
        match self {
            Resource::RawBytes(_) => ResourceType::RawBytes,
            Resource::HtmlBuildResource(_) => ResourceType::HtmlBuildResource,
            Resource::PreprocessAndIncludeHtml(_) => ResourceType::PreprocessAndIncludeHtml,
            Resource::Css(_) => ResourceType::Css,
            Resource::WasmRust => ResourceType::WasmRust,
            Resource::WasmRustLoader => ResourceType::WasmRustLoader,
        }
    }

    pub fn data<'a>(&'a self) -> &'a Cow<'data, [u8]> {
        match self {
            Resource::RawBytes(items)
            | Resource::HtmlBuildResource(items)
            | Resource::Css(items)
            | Resource::PreprocessAndIncludeHtml(items) => items,
            Resource::WasmRust | Resource::WasmRustLoader => {
                panic!("WasmRust resource DOES NOT have data in it")
            }
        }
    }
}
