// Display entire list at once

use std::{collections::HashMap, fmt::Write, panic::Location, rc::Rc};

use chrono::Datelike;
use html_preprocess::{GeneratorArgs, create_generator};

use crate::{
    config,
    util::{self, ExpectNone},
};

mod card;

pub fn init(
    _config: &config::Config,
    generators: &mut HashMap<
        String,
        (
            &Location<'_>,
            Rc<dyn Fn(GeneratorArgs<'_>) -> Result<String, String>>,
        ),
    >,
) {
    generators
        .insert(
            "art-full-listing".to_string(),
            create_generator(|_| Ok(gen_full_listing())),
        )
        .expect_none("expecting not set");
    generators
        .insert(
            "art-card".to_string(),
            create_generator(card::generator_func),
        )
        .expect_none("expecting not set");
}

fn gen_full_listing() -> String {
    let mut listing = String::new();

    let mut last_year = 0;
    let mut last_month = 0;
    let mut has_opened = false;
    for art in &config::arts::ARTS {
        if last_month != art.posted_on.month() || last_year != art.posted_on.year() {
            last_year = art.posted_on.year();
            last_month = art.posted_on.month();
            if has_opened {
                writeln!(&mut listing, "</div>").unwrap();
            } else {
                has_opened = true;
            }
            writeln!(
                &mut listing,
                "<div class=\"art_section\"><h3 class=\"art_divider\">{}</h3>",
                art.posted_on.format("%B %Y")
            )
            .unwrap();
        }

        let year = art.posted_on.year();
        let month = art.posted_on.format("%b");
        let id = util::encode_html(art.page_id);
        let title = util::encode_html(art.title);
        let page_base = format!("$root/{}/{year}/{month}/{id}", config::arts::ARTS_BASE_DIR);

        writeln!(
            &mut listing,
            "<details>
              <summary><a href='{page_base}.html'>{title}</a></summary>
            "
        )
        .unwrap();

        card::generate(&mut listing, art, false);

        writeln!(&mut listing, "</details>").unwrap();
    }

    if has_opened {
        writeln!(&mut listing, "</div>").unwrap();
    }

    listing
}
