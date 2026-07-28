use garmin_tracker_rs_macros::Entity;

#[derive(Clone, Default, Entity)]
#[entity(hasheable = true, comparable = true)]
#[indexes((category, id))]
pub struct Exercise {
    #[id]
    pub category: String,
    #[id]
    pub id: u16,
    pub name: String,
}
