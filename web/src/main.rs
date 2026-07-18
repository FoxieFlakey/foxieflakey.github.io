#[forbid(unsafe_code)]
use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    process::ExitCode,
};

use clap::{Parser, Subcommand};
use codemap_diagnostic::{ColorConfig, Emitter};
use human_bytes::human_bytes;
use mime::Mime;
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::Path,
    time::Instant,
};

use crate::{builder::BuildError, config::Config, server::ServerConfig};

mod builder;
mod config;
mod macros;
mod server;
mod util;

#[derive(Subcommand)]
#[command(verbatim_doc_comment)]
enum Cmd {
    /// Dump the compiled website to this directory. NOTE:
    /// THIS WILL OVERWRITE everything if there same name
    Dump {
        /// Represent an URL pointing to website like
        /// https://foxieflakey.github.io/ when hosting
        /// on github pages. So the site can find the
        /// resources via absolute URLs
        root: String,
        /// Where to place the final preprocessed HTML and
        /// resources. After this the directory is ready
        /// to be hosted
        output_directory: PathBuf,
    },
    /// Run a server, to serve the website
    Serve {
        /// The IP address where the website will be accesible
        /// has to be one IP, binding to multiple IPs are explicitly
        /// not allowed.
        #[arg(default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST))]
        ip: IpAddr,
        #[arg(default_value_t = 8080)]
        port: u16,
    },
}

#[derive(Parser)]
#[command(flatten_help = true)]
struct Args {
    #[command(subcommand)]
    command: Cmd,
}

fn main() -> Result<ExitCode, ExitCode> {
    let cmd = Args::parse().command;

    let config;

    match &cmd {
        Cmd::Dump { root, .. } => config = Config { root: root.clone() },

        Cmd::Serve { ip, port } => {
            config = Config {
                root: format!("http://{ip}:{port}"),
            }
        }
    }

    println!("Building the website");
    let start = Instant::now();
    let data;
    config::arts::init();
    match builder::build(&config) {
        Ok(ok) => data = ok,
        Err(BuildError::PreprocessFailed(file, codemap, diags)) => {
            eprintln!("Failed to preprocess file: '{}'", file.escape_default());
            Emitter::stderr(ColorConfig::Auto, Some(&codemap)).emit(&diags);
            return Err(ExitCode::FAILURE);
        }
        Err(BuildError::LoadCSSNonUtf8(file, err)) => {
            eprintln!("Failed to minify CSS for: '{}'", file.escape_default());
            eprintln!("Error is: {err}");
            return Err(ExitCode::FAILURE);
        }

        Err(BuildError::ParseCSSFailed(file, err)) => {
            eprintln!("Failed to parse CSS for: '{}'", file.escape_default());
            eprintln!("Error is: {err}");
            return Err(ExitCode::FAILURE);
        }

        Err(BuildError::EncodeCSSFailed(file, err)) => {
            eprintln!("Failed to encode CSS for: '{}'", file.escape_default());
            eprintln!("Error is: {err}");
            return Err(ExitCode::FAILURE);
        }

        Err(BuildError::MinifyCSSFailed(file, err)) => {
            eprintln!("Failed to minify CSS for: '{}'", file.escape_default());
            eprintln!("Error is: {err}");
            return Err(ExitCode::FAILURE);
        }
    }
    let time = util::round_duration_to_ms(Instant::now() - start);
    println!(
        "Built the website. Took {:.2}",
        humantime::format_duration(time)
    );

    match cmd {
        Cmd::Dump {
            output_directory, ..
        } => dump(data, &output_directory),

        Cmd::Serve { ip, port } => {
            let server_config = ServerConfig { ip, port };
            server::serve(&server_config, data)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn dump(
    data: HashMap<String, (Cow<'_, [u8]>, Option<Mime>)>,
    output: &Path,
) -> Result<ExitCode, ExitCode> {
    println!("Dumping files to '{}'", output.display());
    let mut written_bytes = 0;
    let start = Instant::now();
    for (path, (bytes, _)) in data {
        let path_raw = util::sanify_path(&path);
        let path = output.join(format!("./{path_raw}"));

        if let Some(parent) = path.parent() {
            // Create parent directories, if not exist
            if !parent.as_os_str().is_empty() && !path.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    eprintln!(
                        "Error: Cannot create parent directory for '{path_raw}' at '{}': {e}",
                        path.display()
                    );
                    ExitCode::FAILURE
                })?;
            }
        }

        let mut file = File::create(&path).map_err(|e| {
            eprintln!(
                "Error: Cannot create file for '{path_raw}' at '{}': {e}",
                path.display()
            );
            ExitCode::FAILURE
        })?;
        file.write_all(&bytes).map_err(|e| {
            eprintln!(
                "Error: Cannot dump file '{path_raw}' to '{}': {e}",
                path.display()
            );
            ExitCode::FAILURE
        })?;
        written_bytes += bytes.len();
    }
    let time = util::round_duration_to_ms(Instant::now() - start);

    println!(
        "Done dumping files. Written {} in {}",
        human_bytes(written_bytes as f64),
        humantime::format_duration(time)
    );
    Ok(ExitCode::SUCCESS)
}
