use serde::Deserialize;

#[derive(Deserialize)]
pub struct NotificationDefinition {
    pub title: String,
    pub body: String,
}
