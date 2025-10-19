use arrow_schema::{DataType, Schema};
use std::sync::Arc;

/// Trait for generating Arrow schema from SoA structures
pub trait ArrowSchemaGen {
    fn arrow_schema() -> Arc<Schema>;
    fn arrow_field_names() -> Vec<&'static str>;
    fn arrow_field_types() -> Vec<DataType>;
}

/// Helper macro to implement ArrowSchemaGen for enum types
#[macro_export]
macro_rules! impl_enum_arrow_type {
    ($enum_type:ty) => {
        impl ArrowSchemaGen for $enum_type {
            fn arrow_schema() -> Arc<Schema> {
                Arc::new(Schema::new(vec![Field::new(
                    "value",
                    DataType::UInt8,
                    false,
                )]))
            }

            fn arrow_field_names() -> Vec<&'static str> {
                vec!["value"]
            }

            fn arrow_field_types() -> Vec<DataType> {
                vec![DataType::UInt8]
            }
        }
    };
}
