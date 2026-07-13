pub mod dao;
pub mod errors;

use std::{
    path::Path,
    sync::{LazyLock, Mutex},
};

use rusqlite::{Connection, Transaction};
use tauri_plugin_log::log::debug;

use self::errors::{DatabaseError, Result};

pub struct Database {
    connection: Option<Connection>,
}

impl Database {
    pub fn new() -> Self {
        Self { connection: None }
    }

    pub fn get_connection(&mut self) -> Result<&mut Connection> {
        if let Some(conn) = self.connection.as_mut() {
            Ok(conn)
        } else {
            Err(DatabaseError::ClosedConnection())
        }
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
        let ddls = [
            (
                "Initial scheme",
                include_str!("../../../../resources/ddl/001_base_schema.sql"),
            ),
            (
                "Adding user profile table",
                include_str!("../../../../resources/ddl/002_user_profile.sql"),
            ),
            (
                "Adding device table",
                include_str!("../../../../resources/ddl/003_devices.sql"),
            ),
            (
                "Training load support",
                include_str!("../../../../resources/ddl/004_training_load.sql"),
            ),
        ];

        self.run_in_transaction(|tx| {
            let current_vers: u16 = tx
                .pragma_query_value(None, "user_version", |r| r.get(0))
                .map_err(DatabaseError::SchemaCreation)?;

            for i in 0..ddls.len() {
                if i as u16 + 1 > current_vers {
                    debug!(
                        "Applying database DDL patch v{}: {}",
                        i + 1,
                        ddls.get(i).unwrap().0
                    );
                    tx.execute_batch(ddls.get(i).unwrap().1)
                        .map_err(DatabaseError::SchemaCreation)?;
                }

                tx.execute_batch(&format!("PRAGMA user_version = {};", ddls.len()))
                    .map_err(DatabaseError::SchemaCreation)?;
            }

            Ok(())
        })
    }
}

pub static DATABASE_INST: LazyLock<Mutex<Database>> = LazyLock::new(|| Mutex::new(Database::new()));
