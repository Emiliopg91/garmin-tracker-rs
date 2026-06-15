pub mod dao;
pub mod errors;

use std::{
    path::Path,
    sync::{LazyLock, Mutex},
};

use rusqlite::{Connection, Transaction};

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
        let statement = r#"
        BEGIN;
            CREATE TABLE IF NOT EXISTS EXERCISE(
                category TEXT NOT NULL,
                id INTEGER NOT NULL,
                name TEXT NOT NULL,
        
                PRIMARY KEY(category, id)
            );
            CREATE INDEX IF NOT EXISTS EXERCISE_ID_CAT ON EXERCISE(category, id);
        
            CREATE TABLE IF NOT EXISTS SESSION(
                date INTEGER NOT NULL,
                workout TEXT NOT NULL,
                total_elapsed_time REAL NOT NULL,
                active_time REAL NOT NULL,
                total_calories INTEGER NOT NULL,
                metabolic_calories INTEGER NOT NULL,
                avg_heart_rate INTEGER NOT NULL,
                max_heart_rate INTEGER NOT NULL,
        
                PRIMARY KEY(date)
            );
            CREATE INDEX IF NOT EXISTS SESSION_WORKOUT ON SESSION(workout);
            CREATE INDEX IF NOT EXISTS SESSION_DATE ON SESSION(date);
        
            CREATE TABLE IF NOT EXISTS SERIE(
                session INTEGER NOT NULL,
                idx INTEGER NOT NULL,
                exercise_category TEXT NOT NULL,
                exercise_id INTEGER NOT NULL,
                reps INTEGER NOT NULL,
                weight REAL NOT NULL,
                pr BOOLEAN NOT NULL,
        
                PRIMARY KEY(session, idx),
        
                FOREIGN KEY(session) REFERENCES SESSION(date) ON DELETE CASCADE,
                FOREIGN KEY(exercise_category, exercise_id) REFERENCES EXERCISE(category, id) ON DELETE CASCADE
            );
        
            CREATE INDEX IF NOT EXISTS SERIE_ID ON SERIE(session, idx);
            CREATE INDEX IF NOT EXISTS SERIE_EXERCISE ON SERIE(exercise_category, exercise_id);
            
        COMMIT;
    "#;

        if let Some(connection) = self.connection.as_mut() {
            connection
                .execute_batch(statement)
                .map_err(DatabaseError::SchemaCreation)?;

            Ok(())
        } else {
            Err(DatabaseError::ClosedConnection())
        }
    }
}

pub static DATABASE_INST: LazyLock<Mutex<Database>> = LazyLock::new(|| Mutex::new(Database::new()));
