use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    tauri_build::build();
    generate_translations_file();
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
    content.push_str("}");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("translation_keys.rs");

    fs::write(&dest_path, content).unwrap();
}
