use std::{
    collections::HashMap,
    env,
    fmt::Write as FmtWrite,
    fs::{self, File},
    io::Write,
    process::Command,
};

#[derive(Debug, Clone, Copy)]
pub enum BuildKind {
    WasmRust,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").expect("Cant get out dir");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Cant get 'web' crate source dir'");
    let inside_projects_build_dir = format!("{manifest_dir}/../target/inside_build_dir_uwu");
    let web_data_dir = format!("{manifest_dir}/../web_data");
    let is_release = env::var("PROFILE").expect("Cannot get cargo's profile") == "release";

    println!("cargo:rerun-if-changed={manifest_dir}/src/config.rs");
    let config_file = fs::read_to_string(format!("{manifest_dir}/src/config.rs"))
        .expect("Cannot open web/src/config.rs");

    let mut things_need_to_be_built = vec![];

    const START_LINE: &str = "// Do not remove this comment, its for scanning stuffs";
    const END_LINE: &str = "// Do not remove this comment too, its for scanning stuffs";

    let mut can_start = false;
    for line in config_file.lines().map(|x| x.trim()) {
        if line == START_LINE {
            if can_start {
                panic!("Double start line detected for config.rs please fix")
            }

            can_start = true;
        } else if line == END_LINE {
            break;
        }

        let mut tokens_iter = line.split(&['(', ')']);
        let kind = tokens_iter
            .next()
            .expect("Expecting html!, html_dep!, etc resource macro");

        match kind {
            "wasm!" => {
                let path = tokens_iter
                    .next()
                    .expect("Expecting getting web_data/ inside path to the WASM crate")
                    .trim_matches('"');

                // OO we're interested in WASM, to trigger
                // build first
                things_need_to_be_built.push((BuildKind::WasmRust, path));
            }

            // not interresting with other
            _ => continue,
        }
    }

    let mut wasm_to_build_map = HashMap::new();

    // Now can build stuffs needed
    for (kind, path) in things_need_to_be_built {
        println!("Building {kind:?} at {path}");
        match kind {
            BuildKind::WasmRust => {
                let crate_name = path.split('/').last().expect("Cannot get last component of crate path (needed ot be same as 'crate-name' so hello_world crate is just hello_word)");

                let mut cmd = Command::new("cargo");
                let src_dir = format!("{web_data_dir}/{path}");
                let target_dir = format!("{inside_projects_build_dir}/{path}");
                let artifacts_dir = format!("{target_dir}/meow_artifacts");
                cmd.current_dir(&src_dir);
                cmd.arg("build");
                cmd.arg("--target-dir");
                cmd.arg(&target_dir);
                cmd.arg("-Z");
                cmd.arg("unstable-options");
                cmd.arg("--artifact-dir");
                cmd.arg(&artifacts_dir);

                if is_release {
                    cmd.arg("--release");
                }

                let status = cmd
                    .status()
                    .expect(&format!("Cannot wait for Rust build completion of {path}"))
                    .code()
                    .expect(&format!(
                        "Cannot build WASM crate at {path} because was signalled"
                    ));

                assert!(
                    status == 0,
                    "Cargo build for {path} WASM exited with {status}"
                );

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
                        "Cannot wait for wasm-bindgen to complete for {path}"
                    ))
                    .code()
                    .expect(&format!(
                        "Cannot run wasm-bindgen at {path} because was signalled"
                    ));

                assert!(
                    status == 0,
                    "wasm-bindgen for {path} WASM exited with {status}"
                );

                assert!(
                    wasm_to_build_map
                        .insert(
                            format!("{}_bg.wasm", path),
                            (kind, format!("{artifacts_dir}/{crate_name}_bg.wasm"))
                        )
                        .is_none(),
                    "There is duplicate '{path}'!"
                );

                assert!(
                    wasm_to_build_map
                        .insert(
                            format!("{}.js", path),
                            (kind, format!("{artifacts_dir}/{crate_name}.js"))
                        )
                        .is_none(),
                    "There is duplicate '{path}'!"
                );
            }
        };
    }

    let mut opts = String::new();
    for (path, (kind, artifact_path)) in wasm_to_build_map {
        let kind = match kind {
            BuildKind::WasmRust => "WasmType::Rust",
        };

        writeln!(
            &mut opts,
            "(\"{path}\", {}, &include_bytes!(\"{artifact_path}\")[..]),",
            kind
        )
        .unwrap();
    }

    let output = format!(
        r#"
#[allow(unused)]
pub static INITIAL_WASM_BLOBS: LazyLock<HashMap<&'static str, (WasmType, &'static [u8])>> =
    LazyLock::new(|| {{
        let entries = [
            {opts}
        ];

        let mut map = HashMap::new();
        map.reserve(entries.len());

        let iter = entries.into_iter();

        for (path, kind, blob) in iter {{
            map.insert(path, (kind, blob));
        }}

        map
    }});
"#
    );

    File::create(format!("{out_dir}/wasm_blobs.rs"))
        .expect("Cannot create wasm_blobs.rs")
        .write_all(output.as_bytes())
        .expect("Cannot write to wasm_blobs.rs");
}
