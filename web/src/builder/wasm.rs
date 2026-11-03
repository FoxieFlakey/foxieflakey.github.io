use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fs::File,
    io::{ErrorKind, Read},
    process::Command,
};

use crate::config::{self, WasmType};

pub struct Wasm {
    web_data_dir_and_build_dir: Option<(String, String)>,
    wasm_dirs: HashMap<String, WasmType>,
    refetch_from_disk_set: HashSet<String>,
}

impl Wasm {
    pub fn new() -> Self {
        let mut wasm_dirs = HashMap::new();
        for (path, resource) in config::RESOURCES.iter() {
            match resource {
                config::Resource::RawBytes(_)
                | config::Resource::HtmlBuildResource(_)
                | config::Resource::PreprocessAndIncludeHtml(_)
                | config::Resource::Css(_) => continue,
                config::Resource::WasmRust => {
                    let (crate_path, ext) = path
                        .rsplit_once('.')
                        .expect("Resource::WasmRust's must have .wasm extension");
                    assert_eq!(
                        ext, "wasm",
                        "Resource::WasmRust's must have .wasm extension"
                    );
                    wasm_dirs.insert(
                        crate_path
                            .strip_suffix("_bg")
                            .expect("Resource::WasmRust must ends with _bg.wasm not just .wasm")
                            .to_string(),
                        WasmType::Rust,
                    );
                }
                config::Resource::WasmRustLoader => continue,
            }
        }

        Self {
            web_data_dir_and_build_dir: None,
            refetch_from_disk_set: HashSet::new(),
            wasm_dirs,
        }
    }

    pub fn always_fetch_fresh(&mut self) {
        for (crate_path, ..) in self.wasm_dirs.iter() {
            self.refetch_from_disk_set.insert(crate_path.clone());
        }
    }

    pub fn set_dirs(&mut self, dir: Option<(&str, &str)>) {
        self.web_data_dir_and_build_dir = dir.map(|(x, y)| (x.to_string(), y.to_string()));
    }

    pub fn get(&self, full_path: &str) -> Result<Option<Cow<'static, [u8]>>, String> {
        let Some((path_component, ext)) = full_path.rsplit_once('.') else {
            // We do not own this 'path' obviously, WASM builder never have standalone extension less fle
            return Ok(None);
        };

        let crate_path;
        if let Some(actual_crate_path) = path_component.strip_suffix("_bg") {
            crate_path = actual_crate_path;
        } else {
            crate_path = path_component;
        }

        if !self.wasm_dirs.contains_key(crate_path) {
            // Looked the table, we dont know it
            return Ok(None);
        }

        if ext != "wasm" && ext != "js" {
            // Don't know whats its asking
            return Ok(None);
        }

        if ext == "wasm" && !path_component.ends_with("_bg") {
            // Don't know this one if it ends with .wasm
            // it has to be <crate path>_bg.wasm
            return Ok(None);
        }

        if self.refetch_from_disk_set.contains(crate_path) {
            let (web_data_dir, wasm_build_dir) = self
                .web_data_dir_and_build_dir
                .as_ref()
                .ok_or_else(|| format!("Cannot fetch fresh from disk without setting data dir"))?;

            // Run cargo build first
            let mut cmd = Command::new("cargo");
            let src_dir = format!("{web_data_dir}/{crate_path}");
            let crate_name = crate_path.split('/').last().ok_or_else(|| "Cannot get last component of crate path (needed ot be same as 'crate-name' + '.wasm' so hello_world crate is hello_word.wasm)".to_string())?;
            let target_dir = format!("{wasm_build_dir}/{crate_path}");
            let artifacts_dir = format!("{target_dir}/meow_artifacts");
            cmd.current_dir(&src_dir);
            cmd.arg("build");
            cmd.arg("--target-dir");
            cmd.arg(&target_dir);
            cmd.arg("-Z");
            cmd.arg("unstable-options");
            cmd.arg("--artifact-dir");
            cmd.arg(&artifacts_dir);

            #[cfg(debug_assertions)]
            cmd.arg("--release");

            let status = cmd
                .status()
                .expect(&format!(
                    "Cannot wait for Rust build completion of {crate_path}"
                ))
                .code()
                .expect(&format!(
                    "Cannot build WASM crate at {crate_path} because was signalled"
                ));

            if status != 0 {
                return Err(format!(
                    "Cargo build for {crate_path} WASM exited with {status}"
                ));
            }

            // Run the wasm bindgen
            Command::new("wasm-bindgen")
                .current_dir(src_dir)
                .arg("--target")
                .arg("web")
                .arg("--no-typescript")
                .arg("--out-dir")
                .arg(&artifacts_dir)
                .arg(format!("{artifacts_dir}/{crate_name}.wasm"))
                .status()
                .expect(&format!(
                    "Cannot wait for wasm-bindgen to complete for {crate_path}"
                ))
                .code()
                .expect(&format!(
                    "Cannot run wasm-bindgen at {crate_path} because was signalled"
                ));

            if status != 0 {
                return Err(format!(
                    "wasm-bindgen for {crate_path} WASM exited with {status}"
                ));
            }

            // Then read new one
            match self.wasm_dirs.get(crate_path).unwrap() {
                WasmType::Rust => {
                    let read_path = match ext {
                        "wasm" => format!("{artifacts_dir}/{crate_name}_bg.wasm"),
                        "js" => format!("{artifacts_dir}/{crate_name}.js"),
                        _ => unreachable!(),
                    };

                    // Explicitly fetch from disk
                    let mut file = match File::open(read_path) {
                        Ok(x) => x,
                        Err(e) => {
                            if e.kind() == ErrorKind::NotFound {
                                panic!("Artifact is anymore where it isnt");
                            }

                            return Err(format!(
                                "Cannot fetch fresh from disk for WASM rust crate at {crate_path} while opening: {e}"
                            ));
                        }
                    };

                    let mut buf = vec![];
                    file.read_to_end(&mut buf).map_err(|e| {
                        format!("Cannot fetch fresh from disk for WASM rust crate at {crate_path} while reading: {e}")
                    })?;
                    Ok(Some(Cow::Owned(buf)))
                }
            }
        } else {
            Ok(Some(
                config::INITIAL_WASM_BLOBS
                    .get(full_path)
                    .map(|x| Cow::Borrowed(x.1)),
            )
            .expect("INITIAL_WASM_BLOBS compiled table must have the wasm resources already!"))
        }
    }
}
