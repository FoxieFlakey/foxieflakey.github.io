#[forbid(unsafe_code)]

use clap::Parser;
use std::{
    collections::HashSet, env, fs::File, io::{self, Read, Write}, path::{Path, PathBuf}, process::ExitCode, str::FromStr
};

use codemap_diagnostic::{ColorConfig, Emitter};
use path_jail::JailError;

use html_preprocess::Preprocessor;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Set environment values which replacer can find
    /// -Dbweh=23 lets ${bweh} or $bweh in HTML turned
    /// into 23. To unset use -D-bweh to remove
    #[arg(short = 'D', action = clap::ArgAction::Append, verbatim_doc_comment)]
    env_values: Vec<EnvValue>,

    /// Tells where the base directory for looking up imports
    /// from and containing the input HTML. If omitted
    /// current directory is used
    #[arg(short, long, verbatim_doc_comment)]
    source_dir: Option<String>,

    /// The HTML to be preprocessed. This has to be in source_dir
    input: String,

    /// The output where preprocessed HTML be outputted
    /// or stdout if '-' given
    output: String,

    /// Whether to minify the output HTML or not
    #[arg(short, long)]
    minify: bool,
    
    /// Dependency file, whether to generate makefile depedency or not
    /// On stdout, this no-op
    #[arg(long)]
    makefile_depedency: Option<PathBuf>
}

#[derive(Clone)]
struct EnvValue {
    key: String,

    // none, mean the 'key' is removed
    value: Option<String>,
}

impl FromStr for EnvValue {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('-') {
            let key = &s[1..];
            if key.len() == 0 {
                return Err("Key must be non empty".to_string());
            }

            // Special syntax to undefine
            return Ok(EnvValue {
                key: key.to_string(),
                value: None,
            });
        }

        let (key, val) = s.split_once('=')
            .ok_or_else(|| format!("Environment must have both key and value '{}', to define empty omit the value keep the '='", s.escape_default()))?;

        if key.len() == 0 {
            return Err("Key must be non empty".to_string());
        }

        Ok(EnvValue {
            key: key.to_string(),
            value: Some(val.to_string()),
        })
    }
}

fn main() -> Result<ExitCode, ExitCode> {
    let args = Args::parse();
    let cwd;
    match env::current_dir() {
        Ok(x) => cwd = x,
        Err(e) => {
            eprintln!("Cannot find current directory: {e}");
            return Err(ExitCode::FAILURE);
        }
    };

    let src_dir = args
        .source_dir
        .map(|x| Path::new(&x).to_path_buf())
        .unwrap_or(cwd);

    let mut dependencies = HashSet::new();

    let mut preprocessor = Preprocessor::new(
        |path| {
            let path = Path::new(path);
            let path = path_jail::join(&src_dir, path).map_err(|e| match e {
                JailError::BrokenSymlink(location) => {
                    format!(
                        "Broken symlink at '{}' when accessing '{}'",
                        location.display(),
                        path.display()
                    )
                }

                JailError::EscapedRoot { attempted, root } => {
                    format!(
                        "File path at '{}' (attempting '{}') escapes '{}'",
                        path.display(),
                        attempted.display(),
                        root.display()
                    )
                }

                JailError::Io(e) => {
                    format!("Error reading file: {e}")
                }

                JailError::InvalidPath(err) => {
                    format!("Invalid path to open for '{}': {err}", path.display())
                }

                JailError::InvalidRoot {
                    path,
                    source: Some(source),
                } => {
                    format!(
                        "Cannot access source directory '{}': {source}",
                        path.display()
                    )
                }

                JailError::InvalidRoot { path, source: None } => {
                    if path.parent().is_none() {
                        format!(
                            "Cannot use filesystem root '{}' for source dir",
                            path.display()
                        )
                    } else if !path.is_dir() {
                        format!("Cannot use non directory '{}' source dir", path.display())
                    } else {
                        format!("Invalid source dir '{}'", path.display())
                    }
                }

                e => {
                    format!("Unknown JailError: {e}")
                }
            })?;

            let mut file = File::open(&path)
                .map_err(|e| format!("Error opening '{}': {e}", path.display()))?;

            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| format!("Error reading '{}': {e}", path.display()))?;

            let source_code = str::from_utf8(&buf).map_err(|e| {
                format!(
                    "Error reading '{}' due its invalid UTF-8: {e}",
                    path.display()
                )
            })?;

            dependencies.insert(path);
            Ok(source_code.to_string())
        },
        args.minify,
    );

    // Process environments
    for env_val in &args.env_values {
        match &env_val.value {
            Some(val) => {
                preprocessor.set_env(&env_val.key, val);
            }

            None => {
                preprocessor.unset_env(&env_val.key);
            }
        }
    }

    match preprocessor.process_file(&args.input) {
        Ok(x) => {
            let mut output_file;
            if args.output != "-" {
                match File::create(Path::new(&args.output)) {
                    Ok(x) => output_file = Some(x),
                    Err(e) => {
                        eprintln!("Cannot open output: {e}");
                        return Err(ExitCode::FAILURE);
                    }
                }
            } else {
                // Writes to stdout
                output_file = None;
            }
            
            if let Some(file) = &mut output_file {
                if let Err(e) = file.write_all(&x.as_bytes()) {
                    eprintln!("Error writing to output file: {e}");
                    return Err(ExitCode::FAILURE);
                }
            } else {
                println!("{}", x);
            }
            
            if output_file.is_some() {
                if let Some(path) = args.makefile_depedency {
                    let mut dep_file;
                    match File::create(&path) {
                        Ok(x) => dep_file = x,
                        Err(e) => {
                            eprintln!("warning: Cannot open dependency file: {e}");
                            return Err(ExitCode::FAILURE);
                        }
                    }
                    
                    fn handle_err(err: io::Error) -> ExitCode {
                        eprintln!("Cannot write to makefile dependency file: {err}");
                        ExitCode::FAILURE
                    }
                    
                    writeln!(dep_file, "{}: \\", Path::new(&args.output).display()).map_err(handle_err)?;
                    
                    drop(preprocessor);
                    for (idx, dep) in dependencies.iter().enumerate() {
                        write!(dep_file, "\t{} ", dep.display()).map_err(handle_err)?;
                        
                        // All except last entry needs \
                        if idx < dependencies.len() - 1 {
                            writeln!(dep_file, "\\").map_err(handle_err)?;
                        } else {
                            writeln!(dep_file).map_err(handle_err)?;
                        }
                    }
                    
                    for dep in dependencies {
                        writeln!(dep_file, "{}:", dep.display()).map_err(handle_err)?;
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            eprintln!("Failed preprocessing file");
            Emitter::stderr(ColorConfig::Auto, Some(preprocessor.get_codemap())).emit(&e);
            Err(ExitCode::FAILURE)
        }
    }
}
