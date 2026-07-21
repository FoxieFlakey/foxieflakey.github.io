use std::{io::Cursor, sync::OnceLock};

use chrono::{Datelike, NaiveDate};

use crate::{config::Resource, util};

mod data;

// NOTE: This contains unescaped HTML characters
#[derive(Clone, Default)]
pub struct Art {
    pub posted_on: NaiveDate,
    pub title: &'static str,
    pub page_id: &'static str,
    pub data: &'static [u8],
    pub description_short: Option<&'static str>,
    pub description_long: &'static str,
    render_width: Option<u32>,
    render_height: Option<u32>,
    mime: OnceLock<Option<mime::Mime>>,
    
    // Lazily initialized, if render_width and height previously is None
    // then its generated from parsing data.
    //
    // If only render_width Some but  not other, or vice verrsa.
    // it is properly scaled based on aspect ratio
    actual_render_size: OnceLock<(Option<u32>, Option<u32>)>,
    
    // None if can't be scanned/fetched
    actual_size: OnceLock<Option<(u32, u32)>>
}

impl Art {
    pub fn mime(&self) -> &Option<mime::Mime> {
        self.mime.get_or_init(move || util::infer(None, self.data))
    }
    
    pub fn actual_size(&self) -> Option<(u32, u32)> {
        *self.actual_size
            .get_or_init(|| {
                match self.mime().clone()?.type_() {
                    mime::IMAGE => {
                        let reader = image::ImageReader::new(Cursor::new(self.data))
                            .with_guessed_format()
                            .unwrap();
                        
                        match reader.into_dimensions() {
                            Ok(x) => Some(x),
                            Err(e) => {
                                println!("[ERROR] Art module: Cannot parse art file as image: {e}");
                                None
                            }
                        }
                    }
                    _ => None
                }
            })
    }
    
    fn calc_render_size(&self) -> (Option<u32>, Option<u32>) {
        match (self.render_width, self.render_height) {
            (Some(width), Some(height)) => (Some(width), Some(height)),
            (None, None) => {
                let Some((w, h)) = self.actual_size() else {
                    return (None, None)
                };
                
                (Some(w), Some(h))
            },
            (Some(width), None) => {
                let Some((w, h)) = self.actual_size() else {
                    return (None, None)
                };
                
                let ratio = f64::from(h) / f64::from(w);
                let calculated_height = f64::from(width) * ratio;
                
                (Some(w), Some(calculated_height as u32))
            },
            (None, Some(height)) => {
                let Some((w, h)) = self.actual_size() else {
                    return (None, None)
                };
                
                let ratio = f64::from(w) / f64::from(h);
                let calculated_width = f64::from(height) * ratio;
                
                (Some(calculated_width as u32), Some(h))
            }
        }
    }
    
    pub fn render_width(&self) -> Option<u32> {
        self.actual_render_size
            .get_or_init(|| self.calc_render_size())
            .0
    }
    
    pub fn render_height(&self) -> Option<u32> {
        self.actual_render_size
            .get_or_init(|| self.calc_render_size())
            .1
    }

    pub fn path_to_data(&self) -> String {
        let year = self.posted_on.year();
        let month = self.posted_on.format("%b");
        let id = util::encode_html(self.page_id);
        let page_base = format!("{}/{year}/{month}/{id}", ARTS_BASE_DIR);

        let ext = self
            .mime()
            .as_ref()
            .map(|x| x.as_ref())
            .map(mime2ext::mime2ext)
            .flatten()
            .unwrap_or(".bin");
        format!("{page_base}.{ext}")
    }
}

pub const ARTS_BASE_DIR: &'static str = "/arts";
pub use data::ARTS;

pub fn init() {
    // Check if ARTS sorted chronologically
    // start from index 0 mean recent one, to LEN-1 is oldest one

    let mut current = &NaiveDate::MAX;
    let mut current_idx = 0;

    while current_idx < ARTS.len() {
        // NOTE: Couldn't compare directly because traits
        // dont work, and chrono dont have const for the trait necessary
        // it doesnt look as good as i want because cant acces year/month/data
        // because trait of DateLike is not const.
        if ARTS[current_idx].posted_on.to_epoch_days() > current.to_epoch_days() {
            panic!(
                "Arts are not sorted descending due one at index {current_idx} titled {}",
                ARTS[current_idx].title
            );
        }
        current = &ARTS[current_idx].posted_on;

        current_idx += 1;
    }
}

pub fn gen_resources_list() -> Vec<(String, Resource)> {
    ARTS.iter()
        .map(|x| (x.path_to_data(), Resource::RawBytes(x.data)))
        .collect()
}
