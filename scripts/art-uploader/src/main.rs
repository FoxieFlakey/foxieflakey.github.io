use std::{
    fs,
    path::Path,
    process::ExitCode,
};

use chrono::NaiveDate;
use clap::Parser;

use crate::data::UploaderData;

mod data;
mod deviantart_uploader;

#[derive(Parser)]
#[command(flatten_help = true)]
struct Args {
    /// Title of the art
    #[arg(long)]
    title: String,

    /// Description of the art
    #[arg(long)]
    description: String,

    /// Page ID of art
    #[arg(long)]
    id: String,

    /// Filename where the art is. In the ./art_data/src/data. Directory.
    /// Why it the case? Its a script only for assisting me, not publically
    /// usable :<
    filename: String,

    /// Keywords associating with art, comma seperated
    #[arg(long, value_delimiter = ',')]
    keywords: Vec<String>,

    /// Date when the art posted
    #[arg(long, default_value_t = default_post_date())]
    post_date: NaiveDate,
}

fn default_post_date() -> NaiveDate {
    chrono::Local::now().naive_local().date()
}

fn main() -> Result<ExitCode, ExitCode> {
    let args = Args::parse();

    if args.filename.contains('/') {
        eprintln!("Filename cannot containg '/'");
        return Err(ExitCode::FAILURE);
    }

    if args.id.contains(char::is_whitespace) {
        eprintln!("Art ID cannot containg whitespace");
        return Err(ExitCode::FAILURE);
    }

    if args.title.contains('\n') {
        eprintln!("Page title cannot contains newlines");
        return Err(ExitCode::FAILURE);
    }

    for keyword in &args.keywords {
        if keyword.contains(char::is_whitespace) {
            eprintln!("Keyword cannot contains whitespace for '{keyword}'");
            return Err(ExitCode::FAILURE);
        }
    }

    let file_path = Path::new("./art_data/src/data").join(&args.filename);
    let file_data = fs::read(&file_path).map_err(|x| {
        println!("Cannot read art file at '{}': {x}", file_path.display());
        ExitCode::FAILURE
    })?;

    let data_path = Path::new("./.uploader_data.json");
    let data = fs::read_to_string(&data_path).map_err(|x| {
        println!("Cannot open data file at '{}': {x}", data_path.display());
        ExitCode::FAILURE
    })?;

    let mut data = serde_json::from_str::<UploaderData>(&data).map_err(|x| {
        println!(
            "Cannot deserialize data file at '{}': {x}",
            data_path.display()
        );
        ExitCode::FAILURE
    })?;

    println!("-- SUMMARY --");
    println!("Art title: {}", args.title);
    println!("Art ID: {}", args.id);
    println!("Art file path: {}", file_path.display());
    println!("Posted on: {}", args.post_date.format("%a, %d %B %Y"));
    println!("Key words: [{}]", args.keywords.join(", "));
    println!("Description:\n{}", args.description);
    println!("-------------");

    println!("Uploading to DeviantArt");
    deviantart_uploader::upload(&mut data, &args, &file_data).map_err(|x| {
        println!("Cannot upload art: {x}");
        ExitCode::FAILURE
    })?;

    // Saving the uploader data
    let serialized = serde_json::to_string_pretty(&data).map_err(|x| {
        println!("Cannot serialize data: {x}");
        ExitCode::FAILURE
    })?;
    fs::write(&data_path, serialized).map_err(|x| {
        println!(
            "Cannot serialize data file to '{}': {x}",
            data_path.display()
        );
        ExitCode::FAILURE
    })?;

    Ok(ExitCode::SUCCESS)
}
