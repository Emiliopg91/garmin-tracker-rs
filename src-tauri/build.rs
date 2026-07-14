use std::{
    collections::BTreeMap,
    env,
    fs::{self},
    path::{Path, PathBuf},
};

fn main() {
    tauri_build::build();
    generate_ddl_file();
    generate_translations_file();
}

fn generate_translations_file() {
    let translations_file = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("resources")
        .join("translations.yaml");
    println!("cargo:rerun-if-changed={}", translations_file.display());

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("translations.rs");

    let content = fs::read_to_string(translations_file).unwrap();
    let translations_map: BTreeMap<String, BTreeMap<String, String>> =
        serde_yaml::from_str(content.as_str()).unwrap();

    let default_lang = "en";

    let mut lang_var = std::env::var("LANG").unwrap_or("C".to_string());
    if lang_var.contains(".") {
        lang_var = lang_var.split(".").next().unwrap().into();
    }
    if lang_var.contains("_") {
        lang_var = lang_var.split("_").next().unwrap().into();
    }
    lang_var = lang_var.to_lowercase();
    if lang_var == "C" {
        lang_var = "en".into();
    }

    let mut builder = phf_codegen::Map::new();
    for (key, values) in &translations_map {
        builder.entry(
            key.as_str(),
            format!(
                "\"{}\"",
                values
                    .get(&lang_var)
                    .unwrap_or(values.get(default_lang).unwrap_or(&key.to_string()))
                    .clone(),
            ),
        );
    }

    let code = format!(
        "pub static TRANSLATIONS: phf::Map<&'static str, &'static str> = {};\n",
        builder.build()
    );
    fs::write(&dest_path, code).unwrap();
}

fn generate_ddl_file() {
    let ddls_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("resources")
        .join("ddl");
    println!("cargo:rerun-if-changed={}", ddls_dir.display());

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("database_versions.rs");

    let files = fs::read_dir(ddls_dir)
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

    fs::write(&dest_path, output).unwrap();
}
