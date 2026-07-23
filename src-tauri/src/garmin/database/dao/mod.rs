#![allow(dead_code)]

pub mod helpers;

use crate::garmin::database::{
    DATABASE_INST,
    dao::helpers::{
        querys::{QueryBuilder, insert::InsertBuilder, select::SelectQuery, update::UpdateQuery},
        types::where_clause::{Value, Where},
    },
};

pub mod device;
pub mod exercise;
pub mod serie;
pub mod session;
pub mod user;

pub trait Entity: Sized {
    const TABLE_NAME: &'static str;
    const FIELDS: &'static [&'static str];
    const ID_FIELDS: &'static [&'static str];
    const NO_ID_FIELDS: &'static [&'static str];

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

    fn update_one(&self) -> crate::garmin::database::errors::Result<()> {
        let mut db = DATABASE_INST.lock().unwrap();
        db.run_in_transaction(|tx| self.update_one_in_transaction(tx))?;

        Ok(())
    }

    fn select_one(values: Vec<Value>) -> crate::garmin::database::errors::Result<Option<Self>> {
        if Self::ID_FIELDS.is_empty() {
            panic!("No id defined for table {}", Self::TABLE_NAME)
        }
        if Self::ID_FIELDS.len() != values.len() {
            panic!(
                "Id fields and values differs in len: {}-{}",
                Self::ID_FIELDS.len(),
                values.len()
            )
        }

        let mut fields = Self::ID_FIELDS.iter();
        let mut values = values.into_iter();

        let mut cond = Where::Eq(fields.next().unwrap(), values.next().unwrap());
        for field in fields {
            cond = Where::And(
                Box::new(cond),
                Box::new(Where::Eq(field, values.next().unwrap())),
            );
        }

        Ok(SelectQuery::new().where_(cond).fetch()?.into_iter().next())
    }

    fn update_one_in_transaction(
        &self,
        tx: &mut rusqlite::Transaction,
    ) -> crate::garmin::database::errors::Result<()> {
        let mut fields = Self::ID_FIELDS.iter();
        let mut values = self.get_id_values().into_iter();

        let mut cond = Where::Eq(fields.next().unwrap(), values.next().unwrap());
        for field in fields {
            cond = Where::And(
                Box::new(cond),
                Box::new(Where::Eq(field, values.next().unwrap())),
            );
        }

        let mut upd = Self::update().where_(cond);
        let values = self.get_no_id_values();
        for i in 0..Self::NO_ID_FIELDS.len() {
            upd = upd.set(
                Self::NO_ID_FIELDS.get(i).unwrap(),
                values.get(i).unwrap().clone(),
            );
        }

        upd.execute_in_transaction(tx)
    }

    fn get_values(&self) -> Vec<Value>;
    fn get_id_values(&self) -> Vec<Value>;
    fn get_no_id_values(&self) -> Vec<Value>;
}
