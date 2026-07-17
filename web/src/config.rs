use std::{collections::HashMap, sync::LazyLock};

use crate::macros::{html, html_dep, raw};

pub static RESOURCES: LazyLock<HashMap<&str, Resource>> = LazyLock::new(|| {
    let entries = [
        html!("/index.html"),
        html!("/404.html"),
        html_dep!("/components/page.html"),
        raw!("/img/profile.gif"),
        raw!("/css/global.css"),
        raw!("/css/pages/home.css"),
        raw!("/favicon.ico"),
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
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/css/global.css"),
        raw!("/creations/InteraksiManusiaKomputer_ServerDashboard/css/font.ttf"),
        html!("/creations/CSS_animation/index.html"),
        raw!("/creations/CSS_animation/popup.css"),
        raw!("/creations/CSS_animation/left_arrow.png"),
        raw!("/creations/CSS_animation/right_arrow.png"),
        raw!("/css/popup.css"),
        html_dep!("/components/popup.html"),
        html_dep!("/creations/popup.html"),
        raw!("/img/popup/left_arrow.png"),
        raw!("/img/popup/right_arrow.png"),
        html!("/arts/index.html"),
        html_dep!("/components/opengraph.html"),
    ];

    let mut map = HashMap::new();
    map.reserve(entries.len());

    for (path, resource) in entries {
        map.insert(path, resource);
    }

    map
});

pub const FAVICON_ALT_TEXT: &'static str = "Pixel art picture of a fox named Foxie with right eye colored brown and left colored blue. She wears a blue collar, and red bow on left ear. With heart emote on floating top of the head.";

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
