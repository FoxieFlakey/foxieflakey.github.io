#[forbid(unsafe_code)]
use std::{path::PathBuf, process::ExitCode};

#[cfg(feature = "built_in_server")]
use std::net::{IpAddr, Ipv4Addr};

use clap::{Parser, Subcommand};
use codemap_diagnostic::{ColorConfig, Emitter};
use human_bytes::human_bytes;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    time::Instant,
};

use crate::{
    builder::{BuildError, Builder},
    config::Config,
};

mod builder;
mod config;
#[cfg(feature = "built_in_server")]
mod hot_reloader;
mod macros;
#[cfg(feature = "built_in_server")]
use server::ServerConfig;
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
    #[cfg(feature = "built_in_server")]
    Serve {
        /// Hot rebuilding, triggered when files are modified
        /// only for non Rust files are triggered
        #[arg(long, requires = "hot_rebuild_temp_dir")]
        web_src_dir: Option<PathBuf>,
        /// This would contain temporary data from hot rebuilding
        #[arg(long, requires = "web_src_dir")]
        hot_rebuild_temp_dir: Option<PathBuf>,
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

        #[cfg(feature = "built_in_server")]
        Cmd::Serve { ip, port, .. } => {
            config = Config {
                root: format!("http://{ip}:{port}"),
            }
        }
    }

    println!("Building the website");
    let start = Instant::now();
    config::arts::init();
    let mut builder = Builder::new(config);
    builder.set_fetcher(|path| {
        // Fetch from statically saved data
        Ok(config::RESOURCES
            .get(&common_utils::sanify_path(path))
            .map(|x| x.data().clone()))
    });
    do_full_rebuild(&builder)?;

    let time = util::round_duration_to_ms(Instant::now() - start);
    println!(
        "Built the website. Took {:.2}",
        humantime::format_duration(time)
    );

    match cmd {
        Cmd::Dump {
            output_directory, ..
        } => dump(&mut builder, &output_directory),

        #[cfg(feature = "built_in_server")]
        Cmd::Serve {
            ip,
            port,
            web_src_dir: web_dir,
            hot_rebuild_temp_dir,
        } => {
            use std::io::ErrorKind;
            use std::sync::Arc;
            use std::{borrow::Cow, io::Read};

            if let Some(dir) = web_dir.clone() {
                builder.set_dirs(Some((
                    dir.to_str().expect("Web data dir path is not valid UTF-8"),
                    hot_rebuild_temp_dir
                        .unwrap()
                        .to_str()
                        .expect("Web data dir path is not valid UTF-8"),
                )));

                let old_fetcher = builder.take_fetcher();

                // If hot reload active, load file directly from there
                // and fall back to current fetcher
                builder.set_fetcher(move |path| {
                    let path = &common_utils::sanify_path(path);
                    let joined = dir.join(format!("./{path}"));
                    match File::open(&joined) {
                        Ok(mut x) => {
                            let mut buf = Vec::new();
                            x.read_to_end(&mut buf).map_err(|e| {
                                format!("Cannot read '{}' because {e}", joined.display())
                            })?;
                            Ok(Some(Cow::Owned(buf)))
                        }
                        Err(e) => {
                            if e.kind() == ErrorKind::NotFound {
                                if let Some(x) = &old_fetcher {
                                    x(path)
                                } else {
                                    Ok(None)
                                }
                            } else {
                                Err(format!("Cannot open '{}'", joined.display()))
                            }
                        }
                    }
                });
            }

            let builder = Arc::new(builder);
            if let Some(dir) = web_dir {
                use crate::hot_reloader::AutoMiss;
                use std::thread;

                let mut reloader = AutoMiss::new(dir, builder.clone());
                println!("Auto miss enabled!");
                thread::spawn(move || {
                    reloader.run();
                });
            }

            let server_config = ServerConfig { ip, port };
            server::serve(&server_config, move |path| {
                match builder.get(path).map(Some) {
                    Ok(ret) => Ok(ret),
                    Err(e) => {
                        let mut msg = Vec::new();
                        match e {
                            BuildError::PreprocessFailed(file, codemap, diags) => {
                                writeln!(
                                    &mut msg,
                                    "Failed to preprocess file: '{}'",
                                    file.escape_default()
                                )
                                .unwrap();
                                Emitter::vec(&mut msg, Some(&codemap)).emit(&diags);
                            }

                            // File not found is not fatal in server
                            BuildError::CannotFindResource(_) => return Ok(None),
                            BuildError::CannotFetchResource(file, err) => {
                                writeln!(
                                    &mut msg,
                                    "Cannot fetch '{}' for building: {err}",
                                    file.escape_default()
                                )
                                .unwrap();
                            }

                            #[cfg(feature = "minify")]
                            BuildError::LoadCSSNonUtf8(file, err) => {
                                writeln!(
                                    &mut msg,
                                    "Failed to minify CSS for: '{}'",
                                    file.escape_default()
                                )
                                .unwrap();
                                writeln!(&mut msg, "Error is: {err}").unwrap();
                            }

                            #[cfg(feature = "minify")]
                            BuildError::ParseCSSFailed(file, err) => {
                                writeln!(
                                    &mut msg,
                                    "Failed to parse CSS for: '{}'",
                                    file.escape_default()
                                )
                                .unwrap();
                                writeln!(&mut msg, "Error is: {err}").unwrap();
                            }

                            #[cfg(feature = "minify")]
                            BuildError::EncodeCSSFailed(file, err) => {
                                writeln!(
                                    &mut msg,
                                    "Failed to encode CSS for: '{}'",
                                    file.escape_default()
                                )
                                .unwrap();
                                writeln!(&mut msg, "Error is: {err}").unwrap();
                            }

                            #[cfg(feature = "minify")]
                            BuildError::MinifyCSSFailed(file, err) => {
                                writeln!(
                                    &mut msg,
                                    "Failed to minify CSS for: '{}'",
                                    file.escape_default()
                                )
                                .unwrap();
                                writeln!(&mut msg, "Error is: {err}").unwrap();
                            }
                        }
                        Err(String::from_utf8_lossy_owned(msg))
                    }
                }
            })?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn do_full_rebuild(builder: &Builder<'_>) -> Result<(), ExitCode> {
    match builder.build_all() {
        Ok(()) => (),
        Err(BuildError::PreprocessFailed(file, codemap, diags)) => {
            eprintln!("Failed to preprocess file: '{}'", file.escape_default());
            Emitter::stderr(ColorConfig::Auto, Some(&codemap)).emit(&diags);
            return Err(ExitCode::FAILURE);
        }

        Err(BuildError::CannotFindResource(file)) => {
            eprintln!("Cannot find '{}' for building", file.escape_default());
            return Err(ExitCode::FAILURE);
        }

        Err(BuildError::CannotFetchResource(file, err)) => {
            eprintln!(
                "Cannot fetch '{}' for building: {err}",
                file.escape_default()
            );
            return Err(ExitCode::FAILURE);
        }

        #[cfg(feature = "minify")]
        Err(BuildError::LoadCSSNonUtf8(file, err)) => {
            eprintln!("Failed to minify CSS for: '{}'", file.escape_default());
            eprintln!("Error is: {err}");
            return Err(ExitCode::FAILURE);
        }

        #[cfg(feature = "minify")]
        Err(BuildError::ParseCSSFailed(file, err)) => {
            eprintln!("Failed to parse CSS for: '{}'", file.escape_default());
            eprintln!("Error is: {err}");
            return Err(ExitCode::FAILURE);
        }

        #[cfg(feature = "minify")]
        Err(BuildError::EncodeCSSFailed(file, err)) => {
            eprintln!("Failed to encode CSS for: '{}'", file.escape_default());
            eprintln!("Error is: {err}");
            return Err(ExitCode::FAILURE);
        }

        #[cfg(feature = "minify")]
        Err(BuildError::MinifyCSSFailed(file, err)) => {
            eprintln!("Failed to minify CSS for: '{}'", file.escape_default());
            eprintln!("Error is: {err}");
            return Err(ExitCode::FAILURE);
        }
    }
    Ok(())
}

fn dump(builder: &mut Builder<'_>, output: &Path) -> Result<ExitCode, ExitCode> {
    println!("Dumping files to '{}'", output.display());
    let mut written_bytes = 0;
    let start = Instant::now();
    for (path, bytes, _) in builder.iter_data() {
        let path_raw = common_utils::sanify_path(&path);
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
