use rusqlite_orm_macros::Entity;

#[derive(Clone, Default, Entity)]
#[entity(hashable = true, comparable = true)]
pub struct Exercise {
    #[primary_key]
    pub category: String,
    #[primary_key]
    pub id: u16,
    pub name: String,
}
