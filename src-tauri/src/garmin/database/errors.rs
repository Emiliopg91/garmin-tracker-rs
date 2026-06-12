use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Cannot operate on database. Closed connection")]
    ClosedConnection(),
    #[error("Error while opening connection to {0}:\n  {1}")]
    Connection(String, rusqlite::Error),
    #[error("Error while enabling foreign keys pragma:\n  {0}")]
    ForeignKeysPragma(rusqlite::Error),
    #[error("Error while creating schema:\n  {0}")]
    SchemaCreation(rusqlite::Error),
    #[error("Error on transaction:\n  {0}")]
    Transaction(rusqlite::Error),
    #[error("Error on insert:\n  {0}")]
    Insert(rusqlite::Error),
    #[error("Error on select:\n  {0}")]
    Select(rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;
