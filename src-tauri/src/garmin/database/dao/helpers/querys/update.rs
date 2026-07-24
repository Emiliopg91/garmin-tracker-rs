use rusqlite::params_from_iter;

use crate::garmin::database::{
    DATABASE_INST,
    dao::{
        Entity, Where,
        helpers::{
            querys::QueryBuilder,
            types::{column_name::ColumnName, value::Value},
        },
    },
    errors::DatabaseError,
};

pub struct UpdateBuilder<T> {
    condition: Option<Where>,
    field_values: Vec<(ColumnName, Value)>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> QueryBuilder<T> for UpdateBuilder<T>
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

impl<T> UpdateBuilder<T>
where
    T: Entity,
{
    pub fn where_(mut self, condition: Where) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn set(mut self, field: ColumnName, value: Value) -> Self {
        self.field_values.push((field, value));
        self
    }

    pub fn execute_in_tx(
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

        let mut cond_params: Vec<Value> = Vec::new();
        if let Some(cond) = &self.condition {
            sentence.push_str(&format!(" WHERE {}", cond.to_sql()));
            cond_params = cond.clone().into_params();
        }

        let mut params = self
            .field_values
            .iter()
            .map(|(_, v)| v)
            .cloned()
            .collect::<Vec<Value>>();
        params.extend(cond_params);

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
