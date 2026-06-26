use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Cannot operate on database. Closed connection")]
    ClosedConnection(),
    #[error("Error while opening connection to {0}: {1}")]
    Connection(String, rusqlite::Error),
    #[error("Error while enabling foreign keys pragma: {0}")]
    ForeignKeysPragma(rusqlite::Error),
    #[error("Error while creating schema: {0}")]
    SchemaCreation(rusqlite::Error),
    #[error("Error on transaction: {0}")]
    Transaction(rusqlite::Error),
    #[error("Error on insert: {0}")]
    Insert(rusqlite::Error),
    #[error("Error on select: {0}")]
    Select(rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;
