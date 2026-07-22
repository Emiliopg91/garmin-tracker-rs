use rusqlite::params_from_iter;
use tauri_plugin_log::log::debug;

use crate::garmin::database::{
    DATABASE_INST,
    dao::{Entity, ToSqlStr, Where, helpers::querys::QueryBuilder},
    errors::DatabaseError,
};

pub struct UpdateQuery<T> {
    condition: Option<Where>,
    field_values: Vec<(&'static str, Box<dyn ToSqlStr>)>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> QueryBuilder<T> for UpdateQuery<T>
where
    T: Entity,
{
    fn new() -> Self {
        Self {
            condition: None,
            field_values: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> UpdateQuery<T>
where
    T: Entity,
{
    pub fn where_(mut self, condition: Where) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn set(mut self, field: &'static str, value: impl ToSqlStr + 'static) -> Self {
        self.field_values.push((field, Box::new(value)));
        self
    }

    pub fn execute_in_transaction(
        &self,
        tx: &rusqlite::Transaction,
    ) -> crate::garmin::database::errors::Result<()> {
        if self.field_values.is_empty() {
            return Ok(()); // o un error, según prefieras
        }

        let mut sentence = format!("UPDATE {} SET ", T::TABLE_NAME);
        sentence.push_str(
            &self
                .field_values
                .iter()
                .map(|(f, _)| format!("{}=?", f))
                .collect::<Vec<String>>()
                .join(", "),
        );

        let mut cond_params: Vec<Box<dyn ToSqlStr>> = Vec::new();
        if let Some(cond) = &self.condition {
            sentence.push_str(&format!(" WHERE {}", cond.to_sql()));
            cond_params = cond.clone().into_params();
        }

        let mut params: Vec<&dyn ToSqlStr> =
            self.field_values.iter().map(|(_, v)| v.as_ref()).collect();
        params.extend(cond_params.iter().map(|v| v.as_ref()));

        debug!("Running SQL sentence {}", sentence);
        let updated = tx
            .execute(&sentence, params_from_iter(params))
            .map_err(DatabaseError::Update)?;
        debug!("Updated {} rows", updated);

        Ok(())
    }

    pub fn execute(&self) -> crate::garmin::database::errors::Result<()> {
        let mut db = DATABASE_INST.lock().unwrap();
        db.run_in_transaction(|tx| self.execute_in_transaction(tx))?;
        Ok(())
    }
}
