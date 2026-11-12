use anyhow::Context;
use chrono::{Local, NaiveDate};
use regex::regex;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

use crate::ui;

#[derive(Debug)]
pub struct Parameters {
    pub title: String,
    #[expect(unused)]
    pub id: String,
    pub filename: String,
    pub file: Vec<u8>,
    #[expect(unused)]
    pub post_date: NaiveDate,
    pub keywords: Vec<String>,
    pub description: String,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub meta_version: u32,
    pub id: String,
    pub post_date: NaiveDate,
}

impl Parameters {
    pub async fn ask() -> anyhow::Result<Self> {
        let title = ui::ask_question("Title of art", false, false, None, None, None)
            .await
            .context("Asking title of art")?
            .context("Title is required")?;

        let default_id = regex!(r"[^a-zA-Z0-9_]")
            .replace_all(&stringcase::snake_case(&title), "_")
            .to_string();

        let id = ui::ask_question(
            "Art ID",
            false,
            false,
            Some(&default_id),
            Some((
                regex!("^[a-zA-Z0-9_]*$"),
                "Invalid ID, only a-z, A-Z, 0-9 or underscore allowed for ID",
            )),
            None,
        )
        .await
        .context("Asking art ID")?
        .context("Art ID is required")?;

        let filename = ui::ask_question(
            "Path to art's file'",
            false,
            true,
            None,
            None,
            Some(("./art_data/src/data/", "")),
        )
        .await
        .context("Asking path to art")?
        .context("File path to art is required")?;

        let post_date = ui::ask_question(
            "Post date",
            false,
            false,
            Some(&Local::now().date_naive().format("%Y-%b-%d").to_string()),
            Some((regex!(r"^[+-]?\d{4,}-(?:\d{1,2}|[Jj][Aa][Nn]|[Ff][Ee][Bb]|[Mm][Aa][Rr]|[Aa][Pp][Rr]|[Mm][Aa][Yy]|[Jj][Uu][Nn]|[Jj][Uu][Ll]|[Aa][Uu][Gg]|[Ss][Ee][Pp]|[Oo][Cc][Tt]|[Nn][Oo][Vv]|[Dd][Ee][Cc])-\d{1,2}$"), "Invalid date, the form should be yyyy-mmm-dd where mmm is short name like Jun, Jul, etc caps dont matter")),
            None
        )
        .await
        .context("Asking date of posting")?
        .context("Date of art upload is required")?;

        // First check with numeric month and second with english abbreviation
        let post_date = post_date.trim();
        let post_date = NaiveDate::parse_from_str(&post_date, "%Y-%m-%d")
            .or_else(|_| NaiveDate::parse_from_str(&post_date, "%Y-%b-%d"))
            .context("Invalid post date")?;

        // Create vec of nonempty keywords only
        let keywords = ui::ask_question(
            "Keywords (seperated by newlines, empties are ignored)",
            true,
            false,
            Some(""),
            Some((
                regex!(r"^[-a-zA-Z0-9_\n]*$"),
                "Only a-z, A-Z, 0-9, _, newline, and - is allowed",
            )),
            None,
        )
        .await
        .context("Asking keyword of art")?
        .context("Keywords are required")?
        .split('\n')
        .filter(|x| !x.trim().is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

        let description = ui::ask_question("Description of art", true, false, Some(""), None, None)
            .await
            .context("Asking description of art")?
            .context("Description is required")?;

        let mut file = File::open(&filename).await.context("Opening art file")?;
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)
            .await
            .context("Reading art file content")?;

        Ok(Parameters {
            filename,
            metadata: Metadata {
                meta_version: 1,
                id: id.clone(),
                post_date,
            },
            title,
            id,
            file: file_bytes,
            post_date,
            keywords,
            description,
        })
    }
}
