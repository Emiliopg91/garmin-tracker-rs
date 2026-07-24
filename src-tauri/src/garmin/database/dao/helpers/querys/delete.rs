use std::marker::PhantomData;

use rusqlite::params_from_iter;

use crate::garmin::database::{
    DATABASE_INST,
    dao::{
        Entity,
        helpers::{
            querys::QueryBuilder,
            types::{value::Value, where_clause::Where},
        },
    },
    errors::DatabaseError,
};

pub struct DeleteBuilder<T> {
    condition: Option<Where>,
    _marker: PhantomData<T>,
}

impl<T> QueryBuilder<T> for DeleteBuilder<T> {
    fn new() -> Self {
        DeleteBuilder {
            condition: None,
            _marker: PhantomData,
        }
    }
}

impl<T> DeleteBuilder<T>
where
    T: Entity,
{
    pub fn where_(mut self, condition: Where) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn execute_in_tx(
        &self,
        tx: &rusqlite::Transaction,
    ) -> crate::garmin::database::errors::Result<()> {
        let mut sentence = format!("DELETE FROM {} ", T::TABLE_NAME);

        let params: Vec<Value> = Vec::new();
        if let Some(cond) = &self.condition {
            sentence.push_str(&format!(" WHERE {}", cond.to_sql()));
        }

        Self::log_query_start(&sentence, &params);
        let updated = tx
            .execute(&sentence, params_from_iter(params))
            .map_err(DatabaseError::Update)?;
        Self::log_query_ending(updated, false);

        Ok(())
    }

    pub fn execute(&self) -> crate::garmin::database::errors::Result<()> {
        let mut db = DATABASE_INST.lock().unwrap();
        db.run_in_mut_tx(|tx| self.execute_in_tx(tx))?;
        Ok(())
    }
}
