pub mod dao;
pub mod errors;

use std::{
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
    time::Duration,
};

use garmin_tracker_rs_macros::dlls;
use rusqlite::{Connection, Transaction, backup::Backup};
use tauri_plugin_log::log::debug;

use self::errors::{DatabaseError, Result};

dlls!();

pub struct Database {
    connection: Option<Connection>,
    path: Option<PathBuf>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            connection: None,
            path: None,
        }
    }

    pub fn open<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file_connection = Connection::open(&path)
            .map_err(|e| DatabaseError::Connection(path.as_ref().display().to_string(), e))?;

        let mut mem_connection = Connection::open_in_memory()
            .map_err(|e| DatabaseError::Connection(":memory:".to_string(), e))?;

        Backup::new(&file_connection, &mut mem_connection)
            .map_err(|e| DatabaseError::Dump(":memory:".to_string(), e))?
            .run_to_completion(i32::MAX, Duration::from_secs(0), None)
            .map_err(|e| DatabaseError::Dump(":memory:".to_string(), e))?;

        drop(file_connection);

        mem_connection
            .execute("PRAGMA foreign_keys = ON", [])
            .map_err(DatabaseError::ForeignKeysPragma)?;

        self.connection = Some(mem_connection);
        self.path = Some(path.as_ref().to_path_buf());

        Ok(())
    }

    fn consolidate(&self) -> Result<()> {
        let path = self.path.as_ref().unwrap();
        let connection = self.connection.as_ref().unwrap();

        let mut file_connection = Connection::open(&path)
            .map_err(|e| DatabaseError::Connection(path.display().to_string(), e))?;

        Backup::new(connection, &mut file_connection)
            .map_err(|e| DatabaseError::Dump(path.display().to_string(), e))?
            .run_to_completion(i32::MAX, Duration::from_secs(0), None)
            .map_err(|e| DatabaseError::Dump(path.display().to_string(), e))?;

        Ok(())
    }

    pub fn run_in_tx<F>(&mut self, mut f: F) -> Result<()>
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

    pub fn run_in_mut_tx<F>(&mut self, f: F) -> Result<()>
    where
        F: FnMut(&mut Transaction) -> Result<()>,
    {
        let res = self.run_in_tx(f);
        self.consolidate()?;
        res
    }

    pub fn create_schema(&mut self) -> Result<()> {
        self.run_in_mut_tx(|tx| {
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
