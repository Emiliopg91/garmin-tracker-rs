use std::{collections::BTreeMap, sync::LazyLock};

pub mod translation_keys {
    include!(concat!(env!("OUT_DIR"), "/translation_keys.rs"));
}

pub static TRANSLATOR_INST: LazyLock<Translator> = LazyLock::new(Translator::new);

pub struct Translator {
    translations: BTreeMap<String, String>,
}

impl Translator {
    const DEFAULT_LANG: &str = "en";

    fn new() -> Self {
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

        let yaml_str = include_str!("../../../resources/translations.yaml");
        let yaml_obj: BTreeMap<String, BTreeMap<String, String>> =
            serde_yaml::from_str(yaml_str).unwrap();

        let mut translations = BTreeMap::new();
        for (key, values) in yaml_obj {
            translations.insert(
                key.clone(),
                values
                    .get(&lang_var)
                    .unwrap_or(values.get(Self::DEFAULT_LANG).unwrap_or(&key.to_string()))
                    .clone(),
            );
        }

        Self { translations }
    }

    pub fn translate(&self, key: &str) -> String {
        match self.translations.get(key) {
            Some(translation) => translation.clone(),
            None => key.to_string(),
        }
    }

    pub fn translate_and_replace<T>(&self, key: &str, replacements: &[T]) -> String
    where
        T: AsRef<str>,
    {
        match self.translations.get(key) {
            Some(translation) => {
                let mut translation = translation.clone();
                for rep in replacements {
                    translation = translation.replacen("{}", rep.as_ref(), 1);
                }
                translation
            }
            None => key.to_string(),
        }
    }
}

#[tauri::command]
pub fn get_translations() -> BTreeMap<String, String> {
    TRANSLATOR_INST.translations.clone()
}
