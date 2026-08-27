use phf_codegen::Map as PhfMap;
use std::fmt::Write as _;
use std::path::Path;
use std::{
    collections::HashMap,
    env,
    fs::{self},
    path::PathBuf,
};

fn main() {
    tauri_build::build();
    generate_translations_file();
}

fn generate_translations_file() {
    let translations_file = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("resources")
        .join("translations.yaml");
    println!("cargo:rerun-if-changed={}", translations_file.display());

    let yaml_str = fs::read_to_string(&translations_file)
        .unwrap_or_else(|e| panic!("Could not read {}: {}", translations_file.display(), e));

    let translation_map =
        serde_yaml::from_str::<HashMap<String, HashMap<String, String>>>(&yaml_str)
            .unwrap_or_else(|e| panic!("Error parsing translations file: {}", e));

    let mut inner_codes: Vec<(String, String)> = Vec::new();
    for (key, translations) in &translation_map {
        let mut inner_map: PhfMap<&str> = PhfMap::new();
        for (lang, text) in translations {
            inner_map.entry(lang.as_str(), &format!("{:?}", text));
        }
        inner_codes.push((key.clone(), inner_map.build().to_string()));
    }
    let mut outer_map: PhfMap<&str> = PhfMap::new();
    for (key, inner_code) in &inner_codes {
        outer_map.entry(key.as_str(), inner_code);
    }

    let mut out = String::new();
    out.push_str("// Automatically generated file, don't edit manually.\n");
    writeln!(
        out,
        "pub static TRANSLATIONS: phf::Map<&'static str, phf::Map<&'static str, &'static str>> = {};",
        outer_map.build()
    )
    .unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("translations_map.rs");
    fs::write(&dest_path, out).unwrap_or_else(|e| panic!("Could not write generated code: {}", e));
}
