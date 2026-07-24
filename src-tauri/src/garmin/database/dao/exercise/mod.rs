use std::hash::Hash;

use garmin_tracker_rs_macros::Entity;

#[derive(Clone, Debug, Default, Entity)]
pub struct Exercise {
    #[id]
    pub category: String,
    #[id]
    pub id: u16,
    pub name: String,
}
impl PartialEq for Exercise {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.category == other.category
    }
}
impl Eq for Exercise {}
impl Hash for Exercise {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.category.hash(state);
    }
}
