use crate::arrow_conversion::ToArrow;
use crate::errors::{PersistenceError, Result};
use crate::persistence::SoAPersistence;
use async_trait::async_trait;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::metadata::ParquetMetaDataReader;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ParquetPersistence<T> {
    base_path: PathBuf,
    compression: Compression,
    writer_properties: Arc<WriterProperties>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ParquetPersistence<T> {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        let compression = Compression::SNAPPY;
        let writer_properties = Arc::new(
            WriterProperties::builder()
                .set_compression(compression)
                .build(),
        );

        Self {
            base_path: base_path.as_ref().to_path_buf(),
            compression,
            writer_properties,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.compression = compression;
        self.writer_properties = Arc::new(
            WriterProperties::builder()
                .set_compression(compression)
                .build(),
        );
        self
    }

    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.writer_properties = Arc::new(
            WriterProperties::builder()
                .set_compression(self.compression)
                .set_data_page_size_limit(page_size)
                .build(),
        );
        self
    }

    fn file_path(&self) -> PathBuf {
        self.base_path.join("data.parquet")
    }
}

#[async_trait]
impl<T> SoAPersistence<T> for ParquetPersistence<T>
where
    T: ToArrow + Send + Sync + 'static,
{
    async fn save(&mut self, data: &T) -> Result<()> {
        let batch = data.to_record_batch()?;
        let file_path = self.file_path();
        let props = (*self.writer_properties).clone();

        tokio::task::spawn_blocking(move || {
            let file = File::create(&file_path).map_err(PersistenceError::Io)?;
            let mut writer = ArrowWriter::try_new(file, batch.schema(), Some(props))
                .map_err(|e| PersistenceError::ArrowError(e.into()))?;
            writer
                .write(&batch)
                .map_err(|e| PersistenceError::ArrowError(e.into()))?;
            writer
                .close()
                .map_err(|e| PersistenceError::ArrowError(e.into()))?;
            Ok::<(), PersistenceError>(())
        })
        .await
        .map_err(|e| PersistenceError::TaskJoin(e.to_string()))??;

        Ok(())
    }

    async fn load(&self) -> Result<Option<T>> {
        let file_path = self.file_path();

        if !tokio::fs::try_exists(&file_path)
            .await
            .map_err(PersistenceError::Io)?
        {
            return Ok(None);
        }

        tokio::task::spawn_blocking(move || -> Result<Option<T>> {
            let file = File::open(&file_path).map_err(PersistenceError::Io)?;
            let builder = ParquetRecordBatchReaderBuilder::try_new(file)
                .map_err(|e| PersistenceError::ArrowError(e.into()))?;
            let reader = builder
                .build()
                .map_err(|e| PersistenceError::ArrowError(e.into()))?;

            let mut batches = Vec::new();
            for maybe_batch in reader {
                let batch = maybe_batch.map_err(PersistenceError::ArrowError)?;
                batches.push(batch);
            }

            if batches.is_empty() {
                return Ok(None);
            }

            let combined_batch = if batches.len() == 1 {
                batches.into_iter().next().unwrap()
            } else {
                let schema = batches[0].schema();
                arrow::compute::concat_batches(&schema, &batches)
                    .map_err(PersistenceError::ArrowError)?
            };

            let data = T::from_record_batch(&combined_batch)?;
            Ok(Some(data))
        })
        .await
        .map_err(|e| PersistenceError::TaskJoin(e.to_string()))?
    }

    async fn append(&mut self, data: &T) -> Result<()> {
        let existing = self.load().await?;
        let new_batch = data.to_record_batch()?;

        let combined_batch = if let Some(existing_data) = existing {
            let existing_batch = existing_data.to_record_batch()?;
            let schema = existing_batch.schema();
            arrow::compute::concat_batches(&schema, &[existing_batch, new_batch])
                .map_err(PersistenceError::ArrowError)?
        } else {
            new_batch
        };

        let file_path = self.file_path();
        let props = (*self.writer_properties).clone();

        tokio::task::spawn_blocking(move || {
            let file = File::create(&file_path).map_err(PersistenceError::Io)?;
            let mut writer = ArrowWriter::try_new(file, combined_batch.schema(), Some(props))
                .map_err(|e| PersistenceError::ArrowError(e.into()))?;
            writer
                .write(&combined_batch)
                .map_err(|e| PersistenceError::ArrowError(e.into()))?;
            writer
                .close()
                .map_err(|e| PersistenceError::ArrowError(e.into()))?;
            Ok::<(), PersistenceError>(())
        })
        .await
        .map_err(|e| PersistenceError::TaskJoin(e.to_string()))??;

        Ok(())
    }

    async fn query<F>(&self, predicate: F) -> Result<Option<T>>
    where
        F: Fn(&T) -> bool + Send + Sync,
    {
        if let Some(data) = self.load().await? {
            if predicate(&data) {
                Ok(Some(data))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn count(&self) -> Result<usize> {
        let file_path = self.file_path();

        if !tokio::fs::try_exists(&file_path)
            .await
            .map_err(PersistenceError::Io)?
        {
            return Ok(0);
        }

        tokio::task::spawn_blocking(move || -> Result<usize> {
            let file = File::open(&file_path).map_err(PersistenceError::Io)?;
            let reader = ParquetMetaDataReader::new()
                .parse_and_finish(&file)
                .map_err(|e| PersistenceError::ArrowError(e.into()))?;
            Ok(reader.file_metadata().num_rows() as usize)
        })
        .await
        .map_err(|e| PersistenceError::TaskJoin(e.to_string()))?
    }

    async fn clear(&mut self) -> Result<()> {
        let file_path = self.file_path();

        if tokio::fs::try_exists(&file_path)
            .await
            .map_err(PersistenceError::Io)?
        {
            tokio::fs::remove_file(&file_path)
                .await
                .map_err(PersistenceError::Io)?;
        }

        Ok(())
    }
}
