use std::{collections::BTreeMap, sync::LazyLock};

include!(concat!(env!("OUT_DIR"), "/translations.rs"));

pub static TRANSLATOR_INST: LazyLock<Translator> = LazyLock::new(Translator::new);

pub struct Translator {}

impl Translator {
    fn new() -> Self {
        Self {}
    }

    pub fn translate(&self, key: &str) -> String {
        match TRANSLATIONS.get(key) {
            Some(translation) => translation.to_string(),
            None => key.to_string(),
        }
    }

    pub fn translate_and_replace<T>(&self, key: &str, replacements: &[T]) -> String
    where
        T: AsRef<str>,
    {
        match TRANSLATIONS.get(key) {
            Some(translation) => {
                let mut translation = translation.to_string();
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
    let mut res = BTreeMap::new();

    for (key, value) in &TRANSLATIONS {
        res.insert(key.to_string(), value.to_string());
    }

    res
}
