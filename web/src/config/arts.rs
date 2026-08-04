use std::{borrow::Cow, fmt::Write, sync::LazyLock};

pub use art_data::{Art, ARTS, ID_TO_ART};
use chrono::{Datelike, NaiveDate};

use crate::{config::Resource, util};

mod sealed {
    pub trait Sealed {}
}

pub trait ArtExt: sealed::Sealed {
    fn mime(&self) -> &Option<mime::Mime>;
    fn path_to_data(&self) -> String;
    fn path_to_art(&self) -> String;
    fn actual_size(&self) -> Option<(u32, u32)>;
    fn calc_render_size(&self) -> (Option<u32>, Option<u32>);
    fn render_width(&self) -> Option<u32>;
    fn render_height(&self) -> Option<u32>;
    fn get_short_desc(&self) -> String;
}

impl sealed::Sealed for Art {}
impl ArtExt for Art {
    fn mime(&self) -> &Option<mime::Mime> {
        self.mime.get_or_init(move || util::infer(None, self.data))
    }

    fn path_to_data(&self) -> String {
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

    fn path_to_art(&self) -> String {
        let year = self.posted_on.year();
        let month = self.posted_on.format("%b");
        let id = util::encode_html(self.page_id);

        format!("{}/{year}/{month}/{id}.html", ARTS_BASE_DIR)
    }
    
    fn actual_size(&self) -> Option<(u32, u32)> {
        *self
            .actual_size
            .get_or_init(|| match self.mime().clone()?.type_() {
                mime::IMAGE => {
                    match imagesize::blob_size(self.data) {
                        Ok(size) => Some((size.width.try_into().unwrap(), size.height.try_into().unwrap())),
                        Err(e) => {
                            println!("[ERROR] Art module: Cannot parse art file as image: {e}");
                            None
                        }
                    }
                }
                _ => None,
            })
    }

    fn calc_render_size(&self) -> (Option<u32>, Option<u32>) {
        match (self.render_width, self.render_height) {
            (Some(width), Some(height)) => (Some(width), Some(height)),
            (None, None) => {
                let Some((w, h)) = self.actual_size() else {
                    return (None, None);
                };

                (Some(w), Some(h))
            }
            (Some(width), None) => {
                let Some((w, h)) = self.actual_size() else {
                    return (None, None);
                };

                let ratio = f64::from(h) / f64::from(w);
                let calculated_height = f64::from(width) * ratio;

                (Some(w), Some(calculated_height as u32))
            }
            (None, Some(height)) => {
                let Some((w, h)) = self.actual_size() else {
                    return (None, None);
                };

                let ratio = f64::from(w) / f64::from(h);
                let calculated_width = f64::from(height) * ratio;

                (Some(calculated_width as u32), Some(h))
            }
        }
    }

    fn render_width(&self) -> Option<u32> {
        self.actual_render_size
            .get_or_init(|| self.calc_render_size())
            .0
    }

    fn render_height(&self) -> Option<u32> {
        self.actual_render_size
            .get_or_init(|| self.calc_render_size())
            .1
    }

    fn get_short_desc(&self) -> String {
        self.description_short
            .map(|x| x.to_string())
            .unwrap_or_else(|| {
                format!(
                    "{}...",
                    String::from_iter(self.description_long.chars().take(60))
                )
            })
    }
}

pub const ARTS_BASE_DIR: &'static str = "/arts";

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
    
    LazyLock::force(&ID_TO_ART);
}

pub fn gen_resources_list() -> Vec<(String, Resource)> {
    let mut resources: Vec<(String, Resource)> = ARTS
        .iter()
        .map(|x| (x.path_to_data(), Resource::RawBytes(Cow::Borrowed(x.data))))
        .collect();

    // Generate per art individual page
    let individual_pages = ARTS.iter().map(|x| {
        let title = html_escape::encode_safe(x.title);
        let short_desc = x.get_short_desc();
        let short_desc = html_escape::encode_safe(&short_desc);
        let path_to_data = x.path_to_data();

        let mut opengraph_data = String::new();

        if let Some(mime) = x.mime() {
            match mime.type_() {
                mime::IMAGE => {
                    writeln!(
                        &mut opengraph_data,
                        r#"
                        <x-metadata-image content="$root/{path_to_data}" />
                        <meta property="twitter:card" content="summary_large_image" />
                        <meta property="og:image:type" content="{}" />
                        "#, mime.to_string()
                    )
                    .unwrap();
                    if let Some((width, height)) = x.actual_size() {
                        writeln!(
                            &mut opengraph_data,
                            r#"<meta property="og:image:width" content="{width}" />"#
                        )
                        .unwrap();
                        writeln!(
                            &mut opengraph_data,
                            r#"<meta property="og:image:height" content="{height}" />"#
                        )
                        .unwrap();
                    }
                }
                mime::VIDEO => {
                    writeln!(
                        &mut opengraph_data,
                        r#"
                        <x-metadata-video content="$root/{path_to_data}" />
                        <meta property="og:video:type" content="{}" />
                        <meta property="twitter:card" content="summary_large_image" />
                        "#,
                        mime.to_string()
                    )
                    .unwrap();
                    if let Some((width, height)) = x.actual_size() {
                        writeln!(
                            &mut opengraph_data,
                            r#"
                            <meta property="og:video:width" content="{width}" />
                            <meta property="og:video:height" content="{height}" />
                            <meta property="twitter:player:width" content="{width}" />
                            <meta property="twitter:player:height" content="{height}" />
                            "#
                        )
                        .unwrap();
                    }
                }
                _ => (),
            }
        }

        let page = format!(
            r#"
                <import src="/components/page.html" />
                <import src="/components/opengraph.html" />
                <x-navbar-set-Gallery />

                <html lang="en">
                    <head>
                        <title>{title}</title>
                        <x-base-metadata />
                        <link href="$root/css/pages/arts.css" rel="stylesheet" />
                        <x-metadata-title content="{title}" />
                        <meta property="og:type" content="website" />
                        <x-metadata-url content="$root/$current_file" />
                        <x-metadata-description content="{short_desc}" />
                        {opengraph_data}
                    </head>
                    
                    <body>
                        <x-page>
                            <x-art-card id="{}" with_title />
                        </x-page>
                    </body>
                </html>
            "#,
            x.page_id
        )
        .into_bytes();
        (
            x.path_to_art(),
            Resource::PreprocessAndIncludeHtml(Cow::Owned(page)),
        )
    });

    resources.extend(individual_pages);
    resources
}
