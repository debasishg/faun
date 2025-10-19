use crate::arrow_conversion::ToArrow;
use crate::errors::Result;
use crate::persistence::{SoABatchPersistence, SoAPersistence};
use arrow_array::RecordBatch;
use arrow_schema::Schema;
use async_trait::async_trait;
use std::sync::{Arc, RwLock};

/// In-memory Arrow-based persistence implementation
pub struct ArrowPersistence<T> {
    batches: Arc<RwLock<Vec<RecordBatch>>>,
    schema: Arc<Schema>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ArrowPersistence<T>
where
    T: ToArrow + Send + Sync,
{
    /// Create a new Arrow persistence instance
    pub fn new() -> Self {
        Self {
            batches: Arc::new(RwLock::new(Vec::new())),
            schema: T::arrow_schema(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create with initial capacity for batches
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            batches: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
            schema: T::arrow_schema(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the Arrow schema
    pub fn schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }

    /// Get all stored RecordBatches (for advanced operations)
    pub fn get_batches(&self) -> Result<Vec<RecordBatch>> {
        let batches = self.batches.read().map_err(|_| {
            crate::errors::PersistenceError::Serialization(
                "Failed to acquire read lock on batches".to_string(),
            )
        })?;
        Ok(batches.clone())
    }

    /// Merge all batches into a single RecordBatch
    pub fn merge_batches(&self) -> Result<Option<RecordBatch>> {
        let batches = self.get_batches()?;

        if batches.is_empty() {
            return Ok(None);
        }

        if batches.len() == 1 {
            return Ok(Some(batches[0].clone()));
        }

        // Concatenate multiple batches
        let merged = arrow::compute::concat_batches(&self.schema, batches.iter())?;
        Ok(Some(merged))
    }

    /// Get memory usage statistics
    pub fn memory_usage(&self) -> Result<MemoryStats> {
        let batches = self.get_batches()?;
        let mut total_bytes = 0;
        let mut total_rows = 0;

        for batch in &batches {
            total_bytes += batch.get_array_memory_size();
            total_rows += batch.num_rows();
        }

        Ok(MemoryStats {
            total_bytes,
            total_rows,
            num_batches: batches.len(),
            avg_batch_size: if batches.is_empty() {
                0
            } else {
                total_bytes / batches.len()
            },
        })
    }
}

#[async_trait]
impl<T> SoAPersistence<T> for ArrowPersistence<T>
where
    T: ToArrow + Send + Sync,
{
    async fn save(&mut self, data: &T) -> Result<()> {
        let batch = data.to_record_batch()?;

        let mut batches = self.batches.write().map_err(|_| {
            crate::errors::PersistenceError::Serialization(
                "Failed to acquire write lock on batches".to_string(),
            )
        })?;

        // Replace all existing batches with the new one
        batches.clear();
        batches.push(batch);

        Ok(())
    }

    async fn load(&self) -> Result<Option<T>> {
        let merged_batch = self.merge_batches()?;

        match merged_batch {
            Some(batch) => Ok(Some(T::from_record_batch(&batch)?)),
            None => Ok(None),
        }
    }

    async fn append(&mut self, data: &T) -> Result<()> {
        let batch = data.to_record_batch()?;

        let mut batches = self.batches.write().map_err(|_| {
            crate::errors::PersistenceError::Serialization(
                "Failed to acquire write lock on batches".to_string(),
            )
        })?;

        batches.push(batch);
        Ok(())
    }

    async fn query<F>(&self, predicate: F) -> Result<Option<T>>
    where
        F: Fn(&T) -> bool + Send + Sync,
    {
        let data = self.load().await?;

        match data {
            Some(d) if predicate(&d) => Ok(Some(d)),
            _ => Ok(None),
        }
    }

    async fn count(&self) -> Result<usize> {
        let batches = self.get_batches()?;
        let total_rows = batches.iter().map(|batch| batch.num_rows()).sum();
        Ok(total_rows)
    }

    async fn clear(&mut self) -> Result<()> {
        let mut batches = self.batches.write().map_err(|_| {
            crate::errors::PersistenceError::Serialization(
                "Failed to acquire write lock on batches".to_string(),
            )
        })?;

        batches.clear();
        Ok(())
    }
}

#[async_trait]
impl<T> SoABatchPersistence<T> for ArrowPersistence<T>
where
    T: ToArrow + Send + Sync,
{
    async fn save_batches(&mut self, batches_data: &[T]) -> Result<()> {
        let mut batches = self.batches.write().map_err(|_| {
            crate::errors::PersistenceError::Serialization(
                "Failed to acquire write lock on batches".to_string(),
            )
        })?;

        // Clear existing and convert all data to RecordBatches
        batches.clear();

        for data in batches_data {
            let batch = data.to_record_batch()?;
            batches.push(batch);
        }

        Ok(())
    }

    async fn load_batches(&self, batch_size: usize) -> Result<Vec<T>> {
        let stored_batches = self.get_batches()?;
        let mut result = Vec::new();

        for batch in stored_batches {
            // If batch is larger than requested size, we could split it
            // For now, return the whole batch as-is
            let data = T::from_record_batch(&batch)?;
            result.push(data);

            // Could implement batch splitting logic here if needed
            if result.len() >= batch_size {
                break;
            }
        }

        Ok(result)
    }

    async fn append_batches(&mut self, batches_data: &[T]) -> Result<()> {
        let mut batches = self.batches.write().map_err(|_| {
            crate::errors::PersistenceError::Serialization(
                "Failed to acquire write lock on batches".to_string(),
            )
        })?;

        for data in batches_data {
            let batch = data.to_record_batch()?;
            batches.push(batch);
        }

        Ok(())
    }
}

impl<T> Default for ArrowPersistence<T>
where
    T: ToArrow + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for ArrowPersistence<T> {
    fn clone(&self) -> Self {
        Self {
            batches: self.batches.clone(),
            schema: self.schema.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Memory usage statistics for Arrow persistence
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_bytes: usize,
    pub total_rows: usize,
    pub num_batches: usize,
    pub avg_batch_size: usize,
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "MemoryStats {{ total_bytes: {}, total_rows: {}, num_batches: {}, avg_batch_size: {} }}",
            self.total_bytes,
            self.total_rows,
            self.num_batches,
            self.avg_batch_size
        )
    }
}
