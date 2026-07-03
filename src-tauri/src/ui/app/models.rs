use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub enum AppEnvironment {
    Debug,
    Release,
}

#[derive(Deserialize, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}
