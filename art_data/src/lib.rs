// This crate store bulk of ArtData

pub mod data;

use std::{collections::HashMap, sync::{LazyLock, OnceLock}};

use chrono::NaiveDate;

trait ExpectNone {
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

// NOTE: This contains unescaped HTML characters
#[derive(Clone, Default)]
pub struct Art {
    pub posted_on: NaiveDate,
    pub title: &'static str,
    pub page_id: &'static str,
    pub data: &'static [u8],
    pub description_short: Option<&'static str>,
    pub description_long: &'static str,
    pub render_width: Option<u32>,
    pub render_height: Option<u32>,
    pub mime: OnceLock<Option<mime::Mime>>,

    // Lazily initialized, if render_width and height previously is None
    // then its generated from parsing data.
    //
    // If only render_width Some but  not other, or vice verrsa.
    // it is properly scaled based on aspect ratio
    pub actual_render_size: OnceLock<(Option<u32>, Option<u32>)>,

    // None if can't be scanned/fetched
    pub actual_size: OnceLock<Option<(u32, u32)>>,
}

pub use data::ARTS;

pub static ID_TO_ART: LazyLock<HashMap<String, &'static Art>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(ARTS.len());
    
    for art in &ARTS {
        map.insert(art.page_id.to_string(), art)
            .expect_none(&format!("There duplicate arts for '{}'" ,art.page_id));
    }
    
    map
});



