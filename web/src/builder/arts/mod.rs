// Display entire list at once

use std::{borrow::Cow, collections::HashMap, fmt::Write, panic::Location, rc::Rc};

use chrono::Datelike;
use html_preprocess::{GeneratorArgs, create_generator};

use crate::{config, util::{self, ExpectNone}};

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
}

fn gen_full_listing() -> String {
    let mut listing = String::new();

    for art in &config::arts::ARTS {
        let year = art.posted_on.year();
        let month = art.posted_on.format("%b");
        let id = util::encode_html(art.page_id);
        let title = util::encode_html(art.title);
        let page_base = format!("$root/{}/{year}/{month}/{id}", config::arts::ARTS_BASE_DIR);
        let short_desc = art.description_short.as_ref().map(util::encode_html);
        let long_desc = util::encode_html(&art.description_long);
        let file_type_desc = art
            .mime()
            .as_ref()
            .map(|x| x.as_ref())
            .map(|x| util::encode_html(x))
            .unwrap_or(Cow::Borrowed("&lt;unknown file type&gt;"));
        let data_url = format!("$root/{}", art.path_to_data());

        writeln!(&mut listing, "<details>").unwrap();
        writeln!(
            &mut listing,
            "  <summary><a href='{page_base}.html'>{title}</a></summary>"
        )
        .unwrap();
        writeln!(&mut listing, "  <div class=\"art_card\">").unwrap();

        // The art preview
        if let Some(mime) = art.mime() {
            let width = art
                .render_width()
                .map(|x| format!("width='{x}'"))
                .unwrap_or("".to_string());
            let height = art
                .render_height()
                .map(|x| format!("height='{x}'"))
                .unwrap_or("".to_string());

            match mime.type_() {
                mime::IMAGE => {
                    writeln!(&mut listing, "<img class=\"art_preview\" loading=\"lazy\" src=\"{data_url}\" {width} {height} />").unwrap();
                }
                mime::VIDEO => {
                    writeln!(&mut listing, "<video class=\"art_preview\" controls loading=\"lazy\" src=\"{data_url}\" {width} {height} />").unwrap();
                }
                _ => writeln!(&mut listing, "<strong>&lt;Unknwon type for preview. <a href=\"{data_url}\">Download first</a> to see&gt;</strong><br />").unwrap()
            }
        } else {
            writeln!(&mut listing, r#"&lt;Cannot determine file type for art preview. <a href="{data_url}">Download first</a> to see&gt;"#).unwrap();
        }

        writeln!(&mut listing, "    <div class=\"art_content\">").unwrap();
        writeln!(
            &mut listing,
            r#"
            <table>
                <tr>
                    <th>Title</th>
                    <td>{title}</td>
                </tr>
                <tr>
                    <th>Posted On</th>
                    <td>{}</td>
                </tr>
                <tr>
                    <th>By</th>
                    <td>Foxie Flakey</td>
                </tr>
                <tr>
                    <th>File Type</th>
                    <td>{}</td>
                </tr>
            </table>
        "#,
            util::encode_html(&art.posted_on.format("%a, %d %B %Y").to_string()),
            file_type_desc
        )
        .unwrap();

        if let Some(short_desc) = short_desc {
            writeln!(
                &mut listing,
                "<h3>Short description</h3><p>{short_desc}</p>"
            )
            .unwrap();
        }
        writeln!(&mut listing, "<h3>Description</h3><p>{long_desc}</p>").unwrap();

        writeln!(&mut listing, "    </div>").unwrap();
        writeln!(&mut listing, "  </div>").unwrap();
        writeln!(&mut listing, "</details>").unwrap();
    }

    listing
}
