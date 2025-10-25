pub mod arrow_conversion;
pub mod arrow_persistence;
pub mod arrow_schema;
pub mod errors;
pub mod parquet_persistence;
pub mod persistence;

pub use arrow_conversion::ToArrow;
pub use arrow_persistence::{ArrowPersistence, MemoryStats};
pub use arrow_schema::ArrowSchemaGen;
pub use errors::{PersistenceError, Result};
pub use parquet_persistence::ParquetPersistence;
pub use persistence::{SoABatchPersistence, SoAPersistence};

// Re-export commonly used types
pub use arrow_array::RecordBatch;
