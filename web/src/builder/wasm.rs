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
                    wasm_dirs.insert(path.to_string(), WasmType::Rust);
                }
            }
        }

        Self {
            web_data_dir_and_build_dir: None,
            refetch_from_disk_set: HashSet::new(),
            wasm_dirs,
        }
    }

    pub fn always_fetch_fresh(&mut self) {
        for (path, ..) in self.wasm_dirs.iter() {
            self.refetch_from_disk_set.insert(path.clone());
        }
    }

    pub fn set_dirs(&mut self, dir: Option<(&str, &str)>) {
        self.web_data_dir_and_build_dir = dir.map(|(x, y)| (x.to_string(), y.to_string()));
    }

    pub fn get(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>, String> {
        if self.refetch_from_disk_set.contains(path) {
            let (web_data_dir, wasm_build_dir) = self
                .web_data_dir_and_build_dir
                .as_ref()
                .ok_or_else(|| format!("Cannot fetch fresh from disk without setting data dir"))?;

            // Run cargo build first
            let mut cmd = Command::new("cargo");
            let src_dir = format!("{web_data_dir}/{path}");
            let target_dir = format!("{wasm_build_dir}/{path}");
            let artifacts_dir = format!("{target_dir}/meow_artifacts");
            cmd.current_dir(src_dir);
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
                .expect(&format!("Cannot wait for Rust build completion of {path}"))
                .code()
                .expect(&format!(
                    "Cannot build WASM crate at {path} because was signalled"
                ));

            if status != 0 {
                return Err(format!("Cargo build for {path} WASM exited with {status}"));
            }

            // Then read new one
            match self.wasm_dirs.get(path).unwrap() {
                WasmType::Rust => {
                    let filename = path.split('/').last().ok_or_else(|| "Cannot get last component of crate path (needed ot be same as 'crate-name' + '.wasm' so hello_world crate is hello_word.wasm)".to_string())?;

                    // Explicitly fetch from disk
                    let mut file = match File::open(format!(
                        "{web_data_dir}/{path}/target/meow_artifacts/{filename}"
                    )) {
                        Ok(x) => x,
                        Err(e) => {
                            if e.kind() == ErrorKind::NotFound {
                                return Ok(None);
                            }

                            return Err(format!(
                                "Cannot fetch fresh from disk for WASM rust crate at {path} while opening: {e}"
                            ));
                        }
                    };

                    let mut buf = vec![];
                    file.read_to_end(&mut buf).map_err(|e| {
                        format!("Cannot fetch fresh from disk for WASM rust crate at {path} while reading: {e}")
                    })?;
                    Ok(Some(Cow::Owned(buf)))
                }
            }
        } else {
            Ok(config::INITIAL_WASM_BLOBS
                .get(path)
                .map(|x| Cow::Borrowed(x.1)))
        }
    }
}
