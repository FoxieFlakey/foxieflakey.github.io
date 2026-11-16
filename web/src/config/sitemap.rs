use std::{collections::HashMap, io::Cursor};

use common_utils::sanify_path;
use quick_xml::events::{BytesDecl, BytesText, Event};

use crate::config::Resource;

const MAX_URL_COUNT: usize = 50000;
const WARN_URL_COUNT: usize = 45000;

const MAX_OUTPUT_SIZE: usize = 50 * 1024 * 1024;
const WARN_OUTPUT_SIZE: usize = 45 * 1024 * 1024;

const URL_BASE: &str = "https://foxieflakey.github.io";

// NOTE: This panics if result is over limit
pub fn generate_monolithic(all_resources: &HashMap<String, Resource<'_>>) -> Vec<u8> {
    let mut url_count = 0;
    let mut output = Vec::new();

    let mut xml_writer = quick_xml::Writer::new_with_indent(Cursor::new(output), b' ', 2);
    xml_writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .expect("Should not failed");

    xml_writer
        .create_element("urlset")
        .with_attribute(("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9"))
        .write_inner_content(|writer| {
            for (path, output) in all_resources {
                let url = &format!("{URL_BASE}{}", sanify_path(path));
                match output {
                    // Those shouldn't be crawled and not accessible
                    Resource::WasmRustLoader
                    | Resource::WasmRust
                    | Resource::Css(_)
                    | Resource::HtmlBuildResource(_)
                    | Resource::RawBytes(_) => continue,
                    Resource::PreprocessAndIncludeHtml(_) => {
                        url_count += 1;
                        // This is actual HTML to be crawlable
                        writer.create_element("url").write_inner_content(|writer| {
                            writer
                                .create_element("loc")
                                .write_text_content(BytesText::new(&url))?;
                            Ok(())
                        })?;
                    }
                }
            }
            Ok(())
        })
        .unwrap();

    output = xml_writer.into_inner().into_inner();

    if output.len() > MAX_OUTPUT_SIZE {
        panic!(
            "Exceeded limit of {MAX_OUTPUT_SIZE} bytes! Generated {}",
            output.len()
        )
    }

    if url_count > MAX_URL_COUNT {
        panic!("Exceeded limit of {MAX_URL_COUNT} URLs! Generated {url_count} urls",)
    }

    if output.len() >= WARN_OUTPUT_SIZE {
        println!(
            "[WARN] Sitemap.xml gets dangerously close to limit at {MAX_OUTPUT_SIZE} bytes currently {} bytes",
            output.len()
        );
    }

    if url_count >= WARN_URL_COUNT {
        println!(
            "[WARN] Sitemap.xml gets dangerously close to URL count limit at {MAX_URL_COUNT} URLs currently {url_count} URls",
        );
    }

    output
}
