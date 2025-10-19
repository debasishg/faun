use crate::errors::Result;
use async_trait::async_trait;

/// Core persistence trait for SoA structures
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

/// Batch operations trait for efficient bulk operations
#[async_trait]
pub trait SoABatchPersistence<T>: SoAPersistence<T> {
    /// Save multiple data batches efficiently
    async fn save_batches(&mut self, batches: &[T]) -> Result<()>;

    /// Load data in batches with a specified batch size
    async fn load_batches(&self, batch_size: usize) -> Result<Vec<T>>;

    /// Append multiple batches efficiently
    async fn append_batches(&mut self, batches: &[T]) -> Result<()>;
}
