use std::{
    collections::BTreeMap,
    env,
    fs::{self},
    path::{Path, PathBuf},
};

fn main() {
    tauri_build::build();
    generate_translations_file();
    generate_ddl_file();
}

fn generate_translations_file() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("resources")
        .join("translations.yaml");
    println!("cargo:rerun-if-changed={}", manifest_dir.display());

    let raw = fs::read_to_string(manifest_dir).unwrap();
    let data: BTreeMap<String, BTreeMap<String, String>> =
        serde_yaml::from_str(&raw).expect("yaml inválido");

    let mut content = "".to_string();
    content.push_str("pub struct TranslationKeys {}\n\n");
    content.push_str("#[allow(dead_code, unused)]\nimpl TranslationKeys {\n");
    data.keys().for_each(|key| {
        content.push_str(&format!(
            "    pub const {} : &str = \"{}\";\n",
            key.to_uppercase(),
            key
        ))
    });
    content.push('}');

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("translation_keys.rs");

    fs::write(&dest_path, content).unwrap();
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

    let mut output = format!("pub static DDLS: [(u16, &str, &str); {}] = [", files.len());
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
            "({}, \"{}\", include_str!(\"{}\")),\n",
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
