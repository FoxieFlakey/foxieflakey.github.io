#![feature(map_try_insert)]

use std::{fs::File, io::Read, path::Path};

use codemap_diagnostic::{ColorConfig, Emitter};
use path_jail::JailError;

use crate::html::Preprocessor;

mod html;

fn main() {
    let src_dir = "./web/";
    let mut preprocessor = Preprocessor::new(|path| {
        let path = Path::new(path);
        let path = path_jail::join(src_dir, path)
            .map_err(|e| {
                match e {
                    JailError::BrokenSymlink(location) => {
                        format!("Broken symlink at '{}' when accessing '{}'", location.display(), path.display())
                    }
                    
                    JailError::EscapedRoot { attempted, root } => {
                        format!("File path at '{}' (attempting '{}') escapes '{}'", path.display(), attempted.display(), root.display())
                    }
                    
                    JailError::Io(e) => {
                        format!("Error reading file: {e}")
                    }
                    
                    JailError::InvalidPath(path) => {
                        format!("Invalid path to read: \"{}\"", path.escape_default())
                    }
                    
                    JailError::InvalidRoot { path, source: Some(source) } => {
                        format!("Cannot access source directory '{}': {source}", path.display())
                    }
                    
                    JailError::InvalidRoot { path, source: None } => {
                        if path.parent().is_none() {
                            format!("Cannot use filesystem root '{}' for source dir", path.display())
                        } else if !path.is_dir() {
                            format!("Cannot use non directory '{}' source dir", path.display())
                        } else {
                            format!("Invalid source dir '{}'", path.display())
                        }
                    }
                    
                    e => {
                        format!("Unknown JailError: {e}")
                    }
                }
            })?;
        
        let mut file = File::open(&path)
            .map_err(|e| {
                format!("Error opening '{}': {e}", path.display())
            })?;
        
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| {
                format!("Error reading '{}': {e}", path.display())
            })?;
        
        let source_code = str::from_utf8(&buf)
            .map_err(|e| {
                format!("Error reading '{}' due its invalid UTF-8: {e}", path.display())
            })?;
        
        Ok(source_code.to_string())
    });

    match preprocessor.process_file("index.html") {
        Ok(_) => println!("File parsed succesfully"),
        Err(e) => {
            println!("Failed parsing file");
            Emitter::stderr(ColorConfig::Auto, Some(preprocessor.get_codemap())).emit(&e);
        }
    }
}
