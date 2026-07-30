use std::{fs, path::PathBuf, str::FromStr, sync::LazyLock};

use tauri_plugin_log::{RotationStrategy, log::LevelFilter};

// App block
pub static APP_TITLE: &str = "Garmin Tracker";
pub static APP_NAME: LazyLock<String> = LazyLock::new(|| env!("CARGO_PKG_NAME").to_string());
pub static APP_VERSION: LazyLock<String> = LazyLock::new(|| env!("CARGO_PKG_VERSION").to_string());
pub static LIB_NAME: LazyLock<String> =
    LazyLock::new(|| format!("{}_lib", APP_NAME.replace('-', "_")));

// Dir block
pub static DATA_LOCAL_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let home = std::env::var("HOME").expect("Could not get local data folder");
    ensure_dir(
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join(APP_NAME.as_str()), // sin clone()
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

pub const LOG_FILE_MAX_SIZE: u128 = 50 * 1_024;
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

fn ensure_dir(dir: PathBuf) -> PathBuf {
    if !dir.exists() {
        fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("Could not create directory {}: {e}", dir.display()));
    }
    dir
}
