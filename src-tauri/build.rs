use std::{
    env,
    fs::{self},
    path::{Path, PathBuf},
};

fn main() {
    tauri_build::build();
    generate_ddl_file();
}

fn generate_ddl_file() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("resources")
        .join("ddl");
    println!("cargo:rerun-if-changed={}", manifest_dir.display());

    let files = fs::read_dir(manifest_dir)
        .unwrap()
        .filter_map(|e| {
            if let Ok(val) = e {
                Some(val.path())
            } else {
                None
            }
        })
        .collect::<Vec<PathBuf>>();

    let mut output = "#[derive(Clone)]\npub struct DdlVersion {\n    pub version: u16,\n    pub description: &'static str,\n    pub sql: &'static str\n}".to_string();
    output.push_str(&format!(
        "pub static DDLS: [DdlVersion; {}] = [",
        files.len()
    ));
    for path in files {
        let name = path.file_name().unwrap().display().to_string();
        let version: u16 = name.split("_").next().unwrap().parse().unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let description = match content.find('\n') {
            Some(pos) => {
                let comment = content[..pos].to_string();
                comment.strip_prefix("--").unwrap().trim().to_string()
            }
            None => unreachable!(),
        };

        output.push_str(&format!(
            "DdlVersion{{version: {}, description:\"{}\", sql:include_str!(\"{}\")}},\n",
            version,
            description,
            path.display()
        ));
    }
    output.push_str("];");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("database_versions.rs");

    fs::write(&dest_path, output).unwrap();
}
