use std::{fs, path::PathBuf, str::FromStr, sync::LazyLock};

use semver::Version;
use tauri_plugin_log::{RotationStrategy, log::LevelFilter};

use crate::utils::translations::Languages;

// App block
pub static APP_TITLE: &str = "Garmin Tracker";
pub static APP_NAME: LazyLock<String> = LazyLock::new(|| env!("CARGO_PKG_NAME").to_string());
pub static APP_VERSION: LazyLock<String> = LazyLock::new(|| env!("CARGO_PKG_VERSION").to_string());
pub static APP_SEM_VERSION: LazyLock<Version> =
    LazyLock::new(|| Version::parse(&APP_VERSION).unwrap());
pub static LIB_NAME: LazyLock<String> =
    LazyLock::new(|| format!("{}_lib", APP_NAME.replace('-', "_")));
pub static PID: LazyLock<u32> = LazyLock::new(std::process::id);
pub static LOCK_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let run_dir = std::env::var("XDG_RUNTIME_DIR").expect("Could not get runtime dir");
    PathBuf::from(run_dir).join(format!("{}.lock", *APP_NAME))
});
pub static URL: &str = "https://api.github.com/repos/Emiliopg91/garmin-tracker-rs/releases/latest";

// Languages block
pub static DEFAULT_LANGUAGE: Languages = Languages::English;
pub static SYSTEM_LANGUAGE: LazyLock<Languages> = LazyLock::new(|| {
    let mut lang_var = std::env::var("LANG").unwrap_or("C".to_string());
    lang_var = lang_var.to_lowercase();
    if lang_var == "c" {
        lang_var = Languages::English.to_string();
    }
    if lang_var.contains(".") {
        lang_var = lang_var.split(".").next().unwrap().into();
    }
    if lang_var.contains("_") {
        lang_var = lang_var.split("_").next().unwrap().into();
    }

    Languages::from(&lang_var)
});

// Dir block
pub static HOME_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(std::env::var("HOME").expect("Could not get home folder")));
pub static DATA_LOCAL_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    ensure_dir(
        HOME_DIR
            .join(".local")
            .join("share")
            .join(APP_NAME.as_str()),
    )
});

// Database block
pub static DB_FILE: LazyLock<PathBuf> = LazyLock::new(|| DATA_LOCAL_DIR.join("database.db"));

// Logs block
pub static LOGS_DIR: LazyLock<PathBuf> = LazyLock::new(|| ensure_dir(DATA_LOCAL_DIR.join("logs")));

pub static LOG_LEVEL: LazyLock<LevelFilter> = LazyLock::new(|| {
    std::env::var("LOGGER_LEVEL")
        .ok()
        .and_then(|v| LevelFilter::from_str(v.trim()).ok())
        .unwrap_or(LevelFilter::Info)
});

pub const LOG_FILE_MAX_SIZE: u128 = 50 * 1_024 * 1_024;
pub const LOG_FILE_ROTATION_STRATEGY: RotationStrategy = RotationStrategy::KeepSome(3);

#[repr(i32)]
pub enum ExitCodes {
    SettingsError = 1,
    DbError = 2,
    NoMainWindow = 3,
    TauriError = 4,
}

impl From<ExitCodes> for i32 {
    fn from(val: ExitCodes) -> Self {
        val as i32
    }
}

// UI block
pub static ICON_PATH: LazyLock<String> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("../../icons/icon.png")
            .display()
            .to_string()
    }

    #[cfg(not(debug_assertions))]
    {
        "/usr/share/icons/hicolor/128x128/apps/garmin-tracker-rs.png".to_string()
    }
});

/// Creates `dir` (and parents) if it doesn't exist yet, then returns it.
fn ensure_dir(dir: PathBuf) -> PathBuf {
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("Could not create directory {}: {e}", dir.display()));
    }
    dir
}
