use crate::errors::Result;
use async_trait::async_trait;

/// Core persistence trait for SoA structures enabling storage backend abstraction.
///
/// This trait provides a unified interface for persisting Structure-of-Arrays (SoA) data
/// across different storage backends (in-memory Arrow, Parquet files, DuckDB, etc.).
///
/// **Why `#[async_trait]`?**
/// - **I/O Operations**: Most persistence backends involve I/O (file system, network, database)
/// - **Non-blocking**: Async operations prevent blocking the application during persistence
/// - **Scalability**: Enables concurrent persistence operations and better resource utilization
/// - **Future-proofing**: Even in-memory implementations benefit from async for consistency
///
/// **Design Principles**:
/// - **Zero-copy where possible**: Leverage SoA → Arrow conversion without data copying
/// - **Backend agnostic**: Same interface works with Arrow, Parquet, DuckDB, etc.
/// - **Error handling**: Comprehensive error types for robust applications
/// - **Batch-friendly**: Efficient operations on columnar data structures
#[async_trait]
pub trait SoAPersistence<T> {
    /// Save data, replacing any existing content
    async fn save(&mut self, data: &T) -> Result<()>;

    /// Load data from storage
    async fn load(&self) -> Result<Option<T>>;

    /// Append data to existing storage
    async fn append(&mut self, data: &T) -> Result<()>;

    /// Query data with a predicate function
    async fn query<F>(&self, predicate: F) -> Result<Option<T>>
    where
        F: Fn(&T) -> bool + Send + Sync;

    /// Get the count of records in storage
    async fn count(&self) -> Result<usize>;

    /// Clear all data from storage
    async fn clear(&mut self) -> Result<()>;

    /// Check if storage is empty
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.count().await? == 0)
    }
}

/// Specialized trait for efficient batch operations on SoA data.
///
/// Extends `SoAPersistence` with optimized bulk operations that leverage the columnar
/// nature of SoA structures. This trait enables high-throughput scenarios where
/// processing large datasets in batches provides significant performance benefits.
///
/// **Key Benefits**:
/// - **Bulk efficiency**: Single transaction/operation for multiple batches
/// - **Memory optimization**: Better memory usage patterns for large datasets  
/// - **Columnar optimization**: Leverages Arrow's batch processing capabilities
/// - **Reduced overhead**: Minimizes per-operation costs (locks, I/O, transactions)
///
/// **Use Cases**:
/// - Data ingestion pipelines
/// - ETL operations
/// - Analytics workloads
/// - Bulk data migration
#[async_trait]
pub trait SoABatchPersistence<T>: SoAPersistence<T> {
    /// Save multiple data batches efficiently
    async fn save_batches(&mut self, batches: &[T]) -> Result<()>;

    /// Load data in batches with a specified batch size
    async fn load_batches(&self, batch_size: usize) -> Result<Vec<T>>;

    /// Append multiple batches efficiently
    async fn append_batches(&mut self, batches: &[T]) -> Result<()>;
}
