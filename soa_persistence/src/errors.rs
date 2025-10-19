use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("Arrow conversion error: {0}")]
    ArrowError(#[from] arrow::error::ArrowError),

    #[error("Schema mismatch: expected {expected}, found {found}")]
    SchemaMismatch { expected: String, found: String },

    #[error("Column not found: {column_name}")]
    ColumnNotFound { column_name: String },

    #[error("Type conversion error: {message}")]
    TypeConversion { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, PersistenceError>;
