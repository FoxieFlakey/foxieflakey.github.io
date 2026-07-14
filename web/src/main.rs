#[forbid(unsafe_code)]
use std::{
    net::{IpAddr, Ipv4Addr},
    process::ExitCode,
};

use codemap_diagnostic::{ColorConfig, Emitter};

use crate::{builder::BuildError, config::Config, server::ServerConfig};

mod builder;
mod config;
mod macros;
mod server;
mod util;

fn main() -> Result<ExitCode, ExitCode> {
    let config = Config {
        root: "http://localhost:8080".to_string(),
    };

    let server_config = ServerConfig {
        ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: 8080,
    };

    let data;
    match builder::build(&config) {
        Ok(ok) => data = ok,
        Err(BuildError::PreprocessFailed(file, codemap, diags)) => {
            eprintln!("Failed to preprocess file: '{}'", file.escape_default());
            Emitter::stderr(ColorConfig::Auto, Some(&codemap)).emit(&diags);
            return Err(ExitCode::FAILURE);
        }
    }
    println!("Built the website, now serving");

    server::serve(&server_config, data)?;

    Ok(ExitCode::SUCCESS)
}
