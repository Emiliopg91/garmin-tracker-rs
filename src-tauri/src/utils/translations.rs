use serde::{Deserialize, Serialize};
use tauri_plugin_log::log::error;

use crate::{logic::app::SETTINGS_INST, utils::constants};

include!(concat!(env!("OUT_DIR"), "/translations_map.rs"));

pub struct Language(pub &'static str);
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Languages {
    Spanish,
    English,
}
impl std::fmt::Display for Languages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code().0)
    }
}
impl Languages {
    pub fn code(&self) -> Language {
        match self {
            Languages::Spanish => Language("es"),
            Languages::English => Language("en"),
        }
    }
    pub fn from_name(literal: &str) -> Self {
        match literal.trim() {
            "Spanish" => Languages::Spanish,
            "English" => Languages::English,
            _ => unreachable!(),
        }
    }
    pub fn from(literal: &str) -> Self {
        match literal.trim() {
            "es" => Languages::Spanish,
            "en" => Languages::English,
            other => {
                error!(
                    "Language '{}' not supported, fallback to '{}'",
                    other,
                    Languages::English
                );
                Languages::English
            }
        }
    }
}

pub fn translate(key: &str) -> String {
    match TRANSLATIONS.get(key) {
        Some(langs) => match langs.get(
            SETTINGS_INST
                .get()
                .unwrap()
                .read()
                .unwrap()
                .language
                .code()
                .0,
        ) {
            Some(translation) => translation,
            None => match langs.get(constants::DEFAULT_LANGUAGE.code().0) {
                Some(translation) => translation,
                None => key,
            },
        },
        None => key,
    }
    .to_string()
}

pub fn translate_and_replace(key: &str, replacements: &[&str]) -> String {
    let mut literal = translate(key);

    for replacement in replacements {
        literal = literal.replace("{}", replacement)
    }

    literal
}
