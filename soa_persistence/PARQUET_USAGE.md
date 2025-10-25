# Parquet Persistence Usage Guide

## Overview

The `ParquetPersistence<T>` implementation provides durable disk-based storage for SoA structures using the Apache Parquet format. The implementation is complete and functional, but requires integration with SoA-generated types that implement the `ToArrow` trait.

## Requirements

To use `ParquetPersistence<T>`, your type `T` must:

1. Implement `ToArrow` trait (conversion to/from Arrow RecordBatch)
2. Implement `ArrowSchemaGen` trait (schema generation)
3. Be `Send + Sync + 'static` (for async operations)

## Basic Usage Example

```rust
use soa_persistence::{ParquetPersistence, SoAPersistence};
use parquet::basic::Compression;

// Assuming you have a SoA-generated type that implements ToArrow:
// #[derive(SoA)]
// struct Order { ... }
// impl ToArrow for OrderSoA { ... }
// impl ArrowSchemaGen for OrderSoA { ... }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create persistence with default settings (SNAPPY compression)
    let mut persistence = ParquetPersistence::<OrderSoA>::new("./data");
    
    // Or configure compression and page size
    let mut persistence = ParquetPersistence::<OrderSoA>::new("./data")
        .with_compression(Compression::ZSTD(Default::default()))
        .with_page_size(8192);
    
    // Save data to Parquet file
    let orders = OrderSoA { /* ... */ };
    persistence.save(&orders).await?;
    
    // Load data (survives application restart)
    if let Some(loaded_orders) = persistence.load().await? {
        println!("Loaded {} orders", loaded_orders.len());
    }
    
    // Append more data
    let more_orders = OrderSoA { /* ... */ };
    persistence.append(&more_orders).await?;
    
    // Efficient count using metadata (no data read)
    let count = persistence.count().await?;
    println!("Total orders: {}", count);
    
    // Query with predicate
    if let Some(result) = persistence.query(|orders| {
        // Your business logic here
        orders.total_amount_sum() > 1000.0
    }).await? {
        println!("Found matching orders");
    }
    
    // Clear all data
    persistence.clear().await?;
    
    Ok(())
}
```

## Compression Options

The Parquet format supports multiple compression algorithms:

```rust
use parquet::basic::{Compression, GzipLevel, ZstdLevel};

// SNAPPY - Fast compression with moderate ratio (default, recommended)
ParquetPersistence::new("./data")
    .with_compression(Compression::SNAPPY)

// GZIP - Better compression ratio, slower than SNAPPY
ParquetPersistence::new("./data")
    .with_compression(Compression::GZIP(GzipLevel::default()))

// ZSTD - Best compression ratio, configurable levels
ParquetPersistence::new("./data")
    .with_compression(Compression::ZSTD(ZstdLevel::default()))

// LZ4 - Very fast, moderate compression
ParquetPersistence::new("./data")
    .with_compression(Compression::LZ4)

// BROTLI - High compression ratio, slower
ParquetPersistence::new("./data")
    .with_compression(Compression::BROTLI(Default::default()))

// UNCOMPRESSED - No compression (fastest writes, largest files)
ParquetPersistence::new("./data")
    .with_compression(Compression::UNCOMPRESSED)
```

## Configuration Options

### Page Size

Smaller pages improve random access but increase metadata overhead:

```rust
ParquetPersistence::new("./data")
    .with_page_size(1024)     // Small pages (1KB)
    .with_page_size(1048576)  // Large pages (1MB, default)
```

## API Reference

All methods are async and return `Result<T, PersistenceError>`:

- **`save(&mut self, data: &T)`** - Save data, replacing existing content
- **`load(&self)`** - Load all data from file (returns `Option<T>`)
- **`append(&mut self, data: &T)`** - Append data (read-merge-write strategy)
- **`query<F>(&self, predicate: F)`** - Query with predicate function
- **`count(&self)`** - Get row count efficiently using metadata
- **`clear(&mut self)`** - Delete the Parquet file

## Performance Characteristics

### Strengths
- **Compression**: 2-10x space savings depending on data and algorithm
- **Durability**: Data persists across application restarts
- **Interoperability**: Standard format readable by Spark, Pandas, Polars, etc.
- **Efficient metadata**: `count()` doesn't read data
- **Async I/O**: Non-blocking operations via `tokio::task::spawn_blocking`

### Limitations
- **Append is expensive**: Parquet files are immutable, so append requires read-merge-write
- **Not for high-frequency updates**: Best for batch writes or append-mostly workloads
- **Single file**: Current implementation uses one file per dataset

## Integration with Data Science Tools

Parquet files created by this library can be read by:

```python
# Python with Pandas
import pandas as pd
df = pd.read_parquet('./data/data.parquet')

# Python with Polars
import polars as pl
df = pl.read_parquet('./data/data.parquet')

# Python with PyArrow
import pyarrow.parquet as pq
table = pq.read_table('./data/data.parquet')
```

## File Storage

- Files are stored in the specified directory
- Filename is `data.parquet`
- Full path: `{base_path}/data.parquet`

## Error Handling

The implementation uses the `PersistenceError` enum:

```rust
use soa_persistence::PersistenceError;

match persistence.load().await {
    Ok(Some(data)) => { /* process data */ }
    Ok(None) => { /* file doesn't exist */ }
    Err(PersistenceError::Io(e)) => { /* I/O error */ }
    Err(PersistenceError::ArrowError(e)) => { /* Arrow/Parquet error */ }
    Err(PersistenceError::TaskJoin(e)) => { /* Async task error */ }
    Err(e) => { /* Other errors */ }
}
```

## Testing

To create tests for your SoA types:

1. Implement `ToArrow` and `ArrowSchemaGen` for your SoA type
2. Create test instances of your SoA structure
3. Use `ParquetPersistence<YourSoAType>` in your tests

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut persistence = ParquetPersistence::<YourSoA>::new(temp_dir.path());
        
        let data = YourSoA { /* ... */ };
        persistence.save(&data).await.unwrap();
        
        let loaded = persistence.load().await.unwrap();
        assert!(loaded.is_some());
    }
}
```

## Next Steps

To use this persistence layer:

1. Ensure your SoA types implement the required traits (`ToArrow`, `ArrowSchemaGen`)
2. Add the persistence layer to your domain stores
3. Configure compression based on your workload
4. Monitor file sizes and performance

For more information, see:
- [columnar_persistence_architecture.md](../columnar_persistence_architecture.md) - Overall architecture
- [columnar_persistence_implementation.md](../columnar_persistence_implementation.md) - Implementation details
