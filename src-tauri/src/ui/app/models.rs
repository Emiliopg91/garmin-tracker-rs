use serde::Serialize;

#[derive(Serialize, Debug)]
pub enum AppEnvironment {
    Debug,
    Release,
}
