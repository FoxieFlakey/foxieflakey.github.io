use std::{borrow::Cow, fmt::Write};

use html_preprocess::GeneratorArgs;

use crate::{
    config::{self, arts::Art},
    util,
};

pub fn generate<W>(output: &mut W, art: &Art, with_title: bool)
where
    W: Write,
{
    let title = util::encode_html(art.title);
    let short_desc = art.description_short.as_ref().map(util::encode_html);
    let long_desc = util::encode_html(&art.description_long);
    let file_type_desc = art
        .mime()
        .as_ref()
        .map(|x| x.as_ref())
        .map(|x| util::encode_html(x))
        .unwrap_or(Cow::Borrowed("&lt;unknown file type&gt;"));
    let data_url = util::sanify_path_unrooted(&format!("$root/{}", art.path_to_data()));

    writeln!(output, "  <div class=\"art_card\">").unwrap();
    if with_title {
        writeln!(output, "<h1>{title}</h1>").unwrap();
    }

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
                writeln!(output, "<img class=\"art_preview\" loading=\"lazy\" src=\"{data_url}\" {width} {height} />").unwrap();
            }
            mime::VIDEO => {
                writeln!(output, "<video class=\"art_preview\" controls loading=\"lazy\" src=\"{data_url}\" {width} {height} />").unwrap();
            }
            _ => writeln!(output, "<strong>&lt;Unknwon type for preview. <a href=\"{data_url}\">Download first</a> to see&gt;</strong><br />").unwrap()
        }
    } else {
        writeln!(output, r#"&lt;Cannot determine file type for art preview. <a href="{data_url}">Download first</a> to see&gt;"#).unwrap();
    }

    writeln!(output, "    <div class=\"art_content\">").unwrap();
    writeln!(
        output,
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
    "#,
        util::encode_html(&art.posted_on.format("%a, %d %B %Y").to_string()),
        file_type_desc
    )
    .unwrap();

    if let Some(short_desc) = short_desc {
        writeln!(
            output,
            "<tr>
                <th>Short description</th>
                <td>{short_desc}</td>
            </tr>"
        )
        .unwrap();
    }
    writeln!(output, "</table>").unwrap();
    writeln!(output, "<p>{long_desc}</p>").unwrap();

    writeln!(output, "    </div>").unwrap();
    writeln!(output, "  </div>").unwrap();
}

pub fn generator_func(args: GeneratorArgs) -> Result<String, String> {
    // Whether wanted with the title or not
    let Some(id) = util::find_attribute(args.attributes, args.preprocessor, "id") else {
        return Err(format!("ID is not set when its required for x-art-card"));
    };

    let with_title =
        util::find_attribute(args.attributes, args.preprocessor, "with_title").is_some();

    // TODO: maybe optimize the layout of ARTS array, or add another one but hashmap
    // so does not perform linear search here

    let Some(art) = config::arts::ARTS.iter().find(|x| x.page_id == id) else {
        return Err(format!("Cannot find art with ID of '{id}'"));
    };

    let mut output = String::new();
    generate(&mut output, art, with_title);
    Ok(output)
}
