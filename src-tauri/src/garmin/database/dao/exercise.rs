use garmin_tracker_rs_macros::Entity;

#[derive(Clone, Debug, Default, Entity)]
#[entity(hasheable = true, comparable = true)]
pub struct Exercise {
    #[id]
    pub category: String,
    #[id]
    pub id: u16,
    pub name: String,
}
