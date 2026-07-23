use std::marker::PhantomData;

use rusqlite::params_from_iter;
use tauri_plugin_log::log::debug;

use crate::garmin::database::{
    DATABASE_INST,
    dao::{
        Entity,
        helpers::{querys::QueryBuilder, types::where_clause::Value},
    },
    errors::DatabaseError,
};

pub struct InsertBuilder<T> {
    items: Vec<T>,
    or_ignore: bool,
    _marker: PhantomData<T>,
}

impl<T> QueryBuilder<T> for InsertBuilder<T> {
    fn new() -> Self {
        InsertBuilder {
            items: Vec::new(),
            or_ignore: false,
            _marker: PhantomData,
        }
    }
}

impl<T> InsertBuilder<T>
where
    T: Entity,
{
    pub fn item(mut self, item: T) -> Self {
        self.items.push(item);
        self
    }

    pub fn or_ignore(mut self, or_ignore: bool) -> Self {
        self.or_ignore = or_ignore;
        self
    }

    pub fn execute_in_transaction(
        &self,
        tx: &rusqlite::Transaction,
    ) -> crate::garmin::database::errors::Result<()> {
        let mut sentence = "INSERT ".to_string();

        if self.or_ignore {
            sentence.push_str("OR IGNORE ");
        }

        sentence.push_str(&format!(
            "INTO {} ({}) VALUES ",
            T::TABLE_NAME,
            T::FIELDS.join(", "),
        ));

        let values_str =
            vec![format!("({})", vec!["?"; T::FIELDS.len()].join(", ")); self.items.len()]
                .join(", ");

        sentence.push_str(&values_str);

        let values = self
            .items
            .iter()
            .flat_map(|item| T::get_values(item).into_iter())
            .collect::<Vec<Value>>();

        debug!("Running SQL sentence {}", sentence);
        let inserted = tx
            .execute(&sentence, params_from_iter(values.iter()))
            .map_err(DatabaseError::Insert)?;
        debug!("Inserted {} rows", inserted);

        Ok(())
    }

    pub fn execute(&self) -> crate::garmin::database::errors::Result<()> {
        let mut db = DATABASE_INST.lock().unwrap();
        db.run_in_transaction(|tx| self.execute_in_transaction(tx))?;
        Ok(())
    }
}
