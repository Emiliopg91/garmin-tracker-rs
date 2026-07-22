pub mod dao;
pub mod errors;

use std::{
    path::Path,
    sync::{LazyLock, Mutex},
};

use garmin_tracker_rs_macros::dlls;
use rusqlite::{Connection, Transaction};
use tauri_plugin_log::log::debug;

use self::errors::{DatabaseError, Result};

dlls!();

pub struct Database {
    connection: Option<Connection>,
}

impl Database {
    pub fn new() -> Self {
        Self { connection: None }
    }

    pub fn open<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let connection = Connection::open(&path)
            .map_err(|e| DatabaseError::Connection(path.as_ref().display().to_string(), e))?;

        connection
            .execute("PRAGMA foreign_keys = ON", [])
            .map_err(DatabaseError::ForeignKeysPragma)?;

        self.connection = Some(connection);

        Ok(())
    }

    pub fn run_in_transaction<F>(&mut self, mut f: F) -> Result<()>
    where
        F: FnMut(&mut Transaction) -> Result<()>,
    {
        if let Some(connection) = self.connection.as_mut() {
            let mut tx = connection
                .transaction()
                .map_err(DatabaseError::Transaction)?;

            f(&mut tx)?;

            tx.commit().map_err(DatabaseError::Transaction)
        } else {
            Err(DatabaseError::ClosedConnection())
        }
    }

    pub fn create_schema(&mut self) -> Result<()> {
        self.run_in_transaction(|tx| {
            let current_vers: u16 = tx
                .pragma_query_value(None, "user_version", |r| r.get(0))
                .map_err(DatabaseError::SchemaCreation)?;

            let updates = DDLS
                .iter()
                .filter(|update| update.version > current_vers)
                .cloned()
                .collect::<Vec<DdlVersion>>();

            for update in &updates {
                debug!(
                    "Applying database DDL patch v{}: {}",
                    update.version, update.description
                );
                tx.execute_batch(update.sql)
                    .map_err(DatabaseError::SchemaCreation)?;
            }

            if !updates.is_empty() {
                debug!("Database updated succesfully")
            }

            Ok(())
        })
    }
}

pub static DATABASE_INST: LazyLock<Mutex<Database>> = LazyLock::new(|| Mutex::new(Database::new()));
