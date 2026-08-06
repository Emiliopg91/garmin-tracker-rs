use rusqlite_orm_macros::Entity;

#[derive(Clone, Entity)]
#[entity(hashable = true, comparable = true)]
#[primary_key(category, id)]
pub struct Exercise {
    pub category: String,
    pub id: u16,
    pub name: String,
}
