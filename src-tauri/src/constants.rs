use std::{fs, path::PathBuf, str::FromStr, sync::LazyLock};

use tauri_plugin_log::{RotationStrategy, log::LevelFilter};

// App block
pub static APP_NAME: LazyLock<String> = LazyLock::new(|| env!("CARGO_PKG_NAME").to_string());
pub static APP_VERSION: LazyLock<String> = LazyLock::new(|| env!("CARGO_PKG_VERSION").to_string());

// Dir block
pub static DATA_LOCAL_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = dirs::data_local_dir()
        .expect("Could not get local data folder")
        .join(APP_NAME.clone());

    if !fs::exists(&dir).expect("IO error") {
        fs::create_dir_all(&dir).expect("Could not create local data folder");
    }

    dir
});

// Database block
pub static DB_FILE: LazyLock<PathBuf> = LazyLock::new(|| DATA_LOCAL_DIR.join("database.db"));

// Logs block
pub static LOGS_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let dir = DATA_LOCAL_DIR.join("logs");

    if !fs::exists(&dir).expect("IO error") {
        fs::create_dir_all(&dir).expect("Could not create logs folder");
    }

    dir
});
pub static LOG_LEVEL: LazyLock<LevelFilter> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    let mut level = LevelFilter::Debug;
    #[cfg(not(debug_assertions))]
    let mut level = LevelFilter::Info;

    if let Ok(level_var) = std::env::var("LOGGER_LEVEL")
        && let Ok(level_filter) = LevelFilter::from_str(level_var.trim())
    {
        level = level_filter
    }

    level
});
pub const LOG_FILE_MAX_SIZE: u128 = 1_024 * 1_024;
pub const LOG_FILE_ROTATION_STRATEGY: RotationStrategy = RotationStrategy::KeepSome(3);

pub enum ExitCodes {
    DbError,
    NoMainWindow,
    TauriError,
}

impl From<ExitCodes> for i32 {
    fn from(val: ExitCodes) -> Self {
        match val {
            ExitCodes::DbError => 1,
            ExitCodes::NoMainWindow => 2,
            ExitCodes::TauriError => 3,
        }
    }
}

// UI block
pub static ICON_PATH: LazyLock<String> = LazyLock::new(|| {
    #[cfg(debug_assertions)]
    return std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
        .join("../../icons/icon.png")
        .display()
        .to_string();

    #[cfg(not(debug_assertions))]
    return "/usr/share/icons/hicolor/128x128/apps/garmin-tracker-rs.png".to_string();
});
