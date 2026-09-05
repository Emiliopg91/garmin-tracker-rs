use rusqlite_orm_macros::Entity;
use serde::{Deserialize, Serialize};

use crate::dao::sport::{self, Sport};

#[derive(Entity, Clone, Serialize, Deserialize)]
#[entity("sub_sport")]
#[primary_key(sport, id)]
pub struct SubSport {
    pub sport: u8,
    pub id: u8,

    #[relationship((sport, sport::entity::columns::ID))]
    pub sport_obj: Option<Sport>,
}
