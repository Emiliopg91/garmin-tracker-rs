use std::marker::PhantomData;

use rusqlite::params_from_iter;

use crate::garmin::database::{
    DATABASE_INST,
    dao::{
        Entity, Where,
        helpers::{querys::QueryBuilder, types::order_by::OrderBy},
    },
    errors::DatabaseError,
};

pub struct SelectBuilder<T> {
    condition: Option<Where>,
    order: Vec<OrderBy>,
    limit: Option<u32>,
    _marker: PhantomData<T>,
}

impl<T> QueryBuilder<T> for SelectBuilder<T>
where
    T: Entity,
{
    fn new() -> Self {
        Self {
            condition: None,
            order: Vec::new(),
            limit: None,
            _marker: PhantomData,
        }
    }
}

impl<T> SelectBuilder<T>
where
    T: Entity,
{
    pub fn where_(mut self, condition: Where) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.order.push(order);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn fetch_in_tx(
        &self,
        tx: &rusqlite::Transaction,
    ) -> crate::garmin::database::errors::Result<Vec<T>> {
        let mut sentence = format!(
            "SELECT {} FROM {}",
            T::FIELDS
                .iter()
                .map(|f| f.as_ref().to_string())
                .collect::<Vec<String>>()
                .join(", "),
            T::TABLE_NAME
        );

        let mut params = Vec::new();
        if let Some(condition) = &self.condition {
            sentence.push_str(&format!(" WHERE {}", condition.to_sql()));
            params = condition.clone().into_params();
        }

        if !self.order.is_empty() {
            sentence.push_str(" ORDER BY ");
            sentence.push_str(
                &self
                    .order
                    .iter()
                    .map(|o| o.to_sql())
                    .collect::<Vec<String>>()
                    .join(", "),
            );
        }

        if let Some(limit) = self.limit {
            sentence.push_str(&format!(" LIMIT {}", limit));
        }

        Self::log_query_start(&sentence, &params);
        let mut stmt = tx.prepare(&sentence).map_err(DatabaseError::Select)?;
        let rows = stmt
            .query_map(params_from_iter(params.iter()), T::map_from_row)
            .map_err(DatabaseError::Select)?;

        let res: Vec<T> = rows.filter_map(|r| r.ok()).collect();
        Self::log_query_ending(res.len(), true);

        Ok(res)
    }

    pub fn fetch(&self) -> crate::garmin::database::errors::Result<Vec<T>> {
        let mut db = DATABASE_INST.lock().unwrap();
        let mut res = Vec::new();
        db.run_in_transaction(|tx| {
            res = self.fetch_in_tx(tx)?;
            Ok(())
        })?;
        Ok(res)
    }

    pub fn fetch_one_in_tx(
        &self,
        tx: &rusqlite::Transaction,
    ) -> crate::garmin::database::errors::Result<Option<T>> {
        Ok(self.fetch_in_tx(tx)?.into_iter().next())
    }

    pub fn fetch_one(&self) -> crate::garmin::database::errors::Result<Option<T>> {
        let mut db = DATABASE_INST.lock().unwrap();
        let mut res = Vec::new();
        db.run_in_transaction(|tx| {
            res = self.fetch_in_tx(tx)?;
            Ok(())
        })?;
        Ok(res.into_iter().next())
    }
}
