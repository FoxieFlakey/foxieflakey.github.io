use std::{
    io::{ErrorKind, SeekFrom},
    path::Path,
};

use anyhow::{Context, bail};
use chrono::Datelike;
use regex::regex;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncBufReadExt, AsyncSeekExt, AsyncWriteExt, BufReader},
};

use crate::interface::Parameters;

pub async fn upload(
    _data: &mut crate::Data,
    // JSON encoded which then base64 encoded content of params.metadata
    _metadata: &str,
    params: &Parameters,
) -> anyhow::Result<()> {
    let filename = match params.full_path.rsplit_once('/') {
        Some((_, filename)) => filename,
        None => &params.full_path,
    };
    let mut data_file = OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(false)
        .create(false)
        .truncate(false)
        .append(false)
        .open("./art_data/src/data.rs")
        .await
        .context("Opening data file")
        .map(BufReader::new)?;

    let mut untransformed = Vec::new();
    let mut data_file_lines = (&mut data_file).lines();
    while let Some(line) = data_file_lines
        .next_line()
        .await
        .context("Cannot read line in data file")?
    {
        untransformed.push(line);
    }

    let untransformed = untransformed;
    let mut transformed = Vec::new();

    let mut found = false;
    for (line_num, line) in untransformed.iter().enumerate().map(|(x, y)| (x + 1, y)) {
        if found {
            // Push the rest of data
            transformed.push(line.clone());
            continue;
        }

        let regex = regex!(r"pub static ARTS: \[Art; ([0-9]+)\] = \[");
        let Some(caps) = regex.captures(line) else {
            transformed.push(line.clone());
            continue;
        };

        let count = caps
            .get(1)
            .expect("There must be capture group #1")
            .as_str()
            .parse::<u32>()
            .with_context(|| format!("Cannot parse art count on line {line_num} in data file"))?;
        let new_count = count + 1;

        let year = params.post_date.year();
        let month = params.post_date.month();
        let day = params.post_date.day();

        // TODO: Make guarantee that these strings converts
        // to valid rust string syntactically. This is "coincident"
        // not seen any contract
        let title = params.title.escape_debug();
        let page_id = params.id.escape_debug();
        let description = params.description.escape_debug();
        let keywords = params
            .keywords
            .iter()
            .map(|x| format!("\"{}\"", x.escape_debug()))
            .collect::<Vec<_>>()
            .join(", ");

        transformed.push(format!("pub static ARTS: [Art; {new_count}] = ["));
        transformed.push(format!(
            r#"    art! {{
        posted_on: NaiveDate::from_ymd_opt({year}, {month}, {day}).unwrap(),
        data: include_bytes!("data/{filename}"),
        title: "{title}",
        page_id: "{page_id}",
        description_long: "{description}",
        keywords: &[{keywords}]
    }},"#
        ));

        found = true;
    }

    // Keep the last line
    transformed.push("\n".to_string());

    if !found {
        bail!("Cannot found line to be edited in data file")
    }

    let mut data_file = data_file.into_inner();
    data_file
        .seek(SeekFrom::Start(0))
        .await
        .context("Cannot seek data file")?;
    data_file
        .set_len(0)
        .await
        .context("Cannot truncate data file")?;

    // then save the file
    let dest = format!("./art_data/src/data/{filename}");
    let dest = Path::new(&dest);

    let mut file = match File::create_new(dest).await {
        Ok(file) => file,
        Err(e) => {
            if e.kind() == ErrorKind::AlreadyExists {
                bail!(
                    "Art named '{filename}' already exists in {}, failing",
                    dest.display()
                );
            }

            bail!("Cannot create new file in {}", dest.display());
        }
    };

    file.write_all(&params.file)
        .await
        .inspect_err(|_| {
            // Lets delete incomplete file
            if let Err(e) = std::fs::remove_file(dest) {
                println!(
                    "[WARN] Cannot delete art file on error '{}': {e}",
                    dest.display()
                );
            }
        })
        .context("Writing art file")?;

    data_file
        .write_all(transformed.join("\n").as_bytes())
        .await
        .inspect_err(|_| {
            // Lets delete incomplete file
            if let Err(e) = std::fs::remove_file(dest) {
                println!(
                    "[WARN] Cannot delete art file on error '{}': {e}",
                    dest.display()
                );
            }
        })
        .context("Cannot write to data file")?;

    Ok(())
}
