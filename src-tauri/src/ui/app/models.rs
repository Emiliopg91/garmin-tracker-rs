use serde::Serialize;

#[derive(Serialize)]
pub enum AppEnvironment {
    Debug,
    Release,
}
