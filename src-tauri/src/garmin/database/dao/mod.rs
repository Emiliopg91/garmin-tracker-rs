#![allow(dead_code)]

pub mod helpers;

use crate::garmin::database::dao::helpers::{
    querys::{QueryBuilder, insert::InsertBuilder, select::SelectQuery, update::UpdateQuery},
    types::where_clause::Where,
};

use self::helpers::types::value::Value;

pub mod device;
pub mod exercise;
pub mod serie;
pub mod session;
pub mod user;

pub trait Entity: Sized {
    const TABLE_NAME: &'static str;
    const FIELDS: &'static [&'static str];

    fn map_from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error>;

    fn insert() -> InsertBuilder<Self> {
        InsertBuilder::new()
    }

    fn select() -> SelectQuery<Self> {
        SelectQuery::<Self>::new()
    }

    fn update() -> UpdateQuery<Self> {
        UpdateQuery::new()
    }

    fn get_values(&self) -> Vec<Value>;
}
