use rusqlite_orm_macros::Entity;

#[derive(Default, Entity)]
#[indexes((date))]
pub struct User {
    #[primary_key]
    pub date: i64,
    pub weight: f32,
    pub fat_ratio: f32,
    pub lean_mass: f32,
    pub water_ratio: f32,
}
