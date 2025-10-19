use crate::arrow_schema::ArrowSchemaGen;
use crate::errors::{PersistenceError, Result};
use arrow_array::{Array, RecordBatch};

/// Trait for converting between SoA structures and Arrow RecordBatch
pub trait ToArrow: ArrowSchemaGen {
    fn to_record_batch(&self) -> Result<RecordBatch>;
    fn from_record_batch(batch: &RecordBatch) -> Result<Self>
    where
        Self: Sized;
}

/// Helper function to safely downcast Arrow array to specific type
pub fn downcast_array<'a, T: Array + 'static>(
    array: &'a dyn Array,
    column_name: &str,
) -> Result<&'a T> {
    array
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| PersistenceError::ColumnNotFound {
            column_name: column_name.to_string(),
        })
}

/// Helper function to convert enum to u8 for Arrow storage
pub fn enum_to_u8<T>(value: T) -> u8
where
    T: Copy + Into<u8>,
{
    value.into()
}

/// Helper function to convert u8 back to enum from Arrow storage
pub fn u8_to_enum<T>(value: u8) -> Result<T>
where
    T: TryFrom<u8>,
    T::Error: std::fmt::Display,
{
    T::try_from(value).map_err(|e| PersistenceError::TypeConversion {
        message: format!("Failed to convert u8 to enum: {}", e),
    })
}
