#![allow(dead_code)]

pub mod helpers;

use crate::garmin::database::dao::helpers::{
    querys::{
        QueryBuilder, delete::DeleteBuilder, insert::InsertBuilder, select::SelectBuilder,
        update::UpdateBuilder,
    },
    types::where_clause::Where,
};

use self::helpers::types::{column_name::ColumnName, value::Value};

pub mod device;
pub mod exercise;
pub mod serie;
pub mod session;
pub mod user;

pub trait Entity: Sized {
    const TABLE_NAME: &'static str;
    const FIELDS: &'static [ColumnName];

    fn map_from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error>;

    fn insert() -> InsertBuilder<Self> {
        InsertBuilder::new()
    }

    fn select() -> SelectBuilder<Self> {
        SelectBuilder::<Self>::new()
    }

    fn update() -> UpdateBuilder<Self> {
        UpdateBuilder::new()
    }

    fn delete() -> DeleteBuilder<Self> {
        DeleteBuilder::new()
    }

    fn get_values(&self) -> Vec<Value>;
}
