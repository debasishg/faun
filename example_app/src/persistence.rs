use crate::{Order, OrderSoA, OrderStatus, PaymentMethod};
use ::arrow_array::{Array, Float64Array, RecordBatch, UInt32Array, UInt64Array, UInt8Array};
use ::arrow_schema::{DataType, Field, Schema};
use soa_persistence::{
    ArrowPersistence, ArrowSchemaGen, MemoryStats, PersistenceError, SoAPersistence, ToArrow,
};
use std::sync::Arc;

// Implement ArrowSchemaGen for OrderSoA
impl ArrowSchemaGen for OrderSoA {
    fn arrow_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("order_id", DataType::UInt64, false),
            Field::new("customer_id", DataType::UInt64, false),
            Field::new("product_id", DataType::UInt64, false),
            Field::new("quantity", DataType::UInt32, false),
            Field::new("unit_price", DataType::Float64, false),
            Field::new("total_amount", DataType::Float64, false),
            Field::new("status", DataType::UInt8, false),
            Field::new("payment_method", DataType::UInt8, false),
            Field::new("order_timestamp", DataType::UInt64, false),
            Field::new("shipping_address_hash", DataType::UInt64, false),
        ]))
    }

    fn arrow_field_names() -> Vec<&'static str> {
        vec![
            "order_id",
            "customer_id",
            "product_id",
            "quantity",
            "unit_price",
            "total_amount",
            "status",
            "payment_method",
            "order_timestamp",
            "shipping_address_hash",
        ]
    }

    fn arrow_field_types() -> Vec<DataType> {
        vec![
            DataType::UInt64,  // order_id
            DataType::UInt64,  // customer_id
            DataType::UInt64,  // product_id
            DataType::UInt32,  // quantity
            DataType::Float64, // unit_price
            DataType::Float64, // total_amount
            DataType::UInt8,   // status (enum)
            DataType::UInt8,   // payment_method (enum)
            DataType::UInt64,  // order_timestamp
            DataType::UInt64,  // shipping_address_hash
        ]
    }
}

// Helper functions for enum conversion
impl From<OrderStatus> for u8 {
    fn from(status: OrderStatus) -> Self {
        match status {
            OrderStatus::Pending => 0,
            OrderStatus::Processing => 1,
            OrderStatus::Shipped => 2,
            OrderStatus::Delivered => 3,
        }
    }
}

impl TryFrom<u8> for OrderStatus {
    type Error = String;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(OrderStatus::Pending),
            1 => Ok(OrderStatus::Processing),
            2 => Ok(OrderStatus::Shipped),
            3 => Ok(OrderStatus::Delivered),
            _ => Err(format!("Invalid OrderStatus value: {}", value)),
        }
    }
}

impl From<PaymentMethod> for u8 {
    fn from(method: PaymentMethod) -> Self {
        match method {
            PaymentMethod::CreditCard => 0,
            PaymentMethod::PayPal => 1,
            PaymentMethod::BankTransfer => 2,
        }
    }
}

impl TryFrom<u8> for PaymentMethod {
    type Error = String;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(PaymentMethod::CreditCard),
            1 => Ok(PaymentMethod::PayPal),
            2 => Ok(PaymentMethod::BankTransfer),
            _ => Err(format!("Invalid PaymentMethod value: {}", value)),
        }
    }
}

// Implement ToArrow for OrderSoA
impl ToArrow for OrderSoA {
    fn to_record_batch(&self) -> soa_persistence::Result<RecordBatch> {
        let schema = Self::arrow_schema();

        // Convert enums to u8 vectors
        let status_u8: Vec<u8> = self.status.iter().map(|&s| s.into()).collect();
        let payment_u8: Vec<u8> = self.payment_method.iter().map(|&p| p.into()).collect();

        // Create Arrow arrays from SoA vectors (zero-copy where possible)
        let columns: Vec<Arc<dyn Array>> = vec![
            Arc::new(UInt64Array::from(self.order_id.clone())),
            Arc::new(UInt64Array::from(self.customer_id.clone())),
            Arc::new(UInt64Array::from(self.product_id.clone())),
            Arc::new(UInt32Array::from(self.quantity.clone())),
            Arc::new(Float64Array::from(self.unit_price.clone())),
            Arc::new(Float64Array::from(self.total_amount.clone())),
            Arc::new(UInt8Array::from(status_u8)),
            Arc::new(UInt8Array::from(payment_u8)),
            Arc::new(UInt64Array::from(self.order_timestamp.clone())),
            Arc::new(UInt64Array::from(self.shipping_address_hash.clone())),
        ];

        RecordBatch::try_new(schema, columns).map_err(|e| PersistenceError::ArrowError(e))
    }

    fn from_record_batch(batch: &RecordBatch) -> soa_persistence::Result<Self> {
        use soa_persistence::arrow_conversion::downcast_array;

        // Extract columns from Arrow RecordBatch
        let order_ids = downcast_array::<UInt64Array>(batch.column(0), "order_id")?;
        let customer_ids = downcast_array::<UInt64Array>(batch.column(1), "customer_id")?;
        let product_ids = downcast_array::<UInt64Array>(batch.column(2), "product_id")?;
        let quantities = downcast_array::<UInt32Array>(batch.column(3), "quantity")?;
        let unit_prices = downcast_array::<Float64Array>(batch.column(4), "unit_price")?;
        let total_amounts = downcast_array::<Float64Array>(batch.column(5), "total_amount")?;
        let status_u8 = downcast_array::<UInt8Array>(batch.column(6), "status")?;
        let payment_u8 = downcast_array::<UInt8Array>(batch.column(7), "payment_method")?;
        let timestamps = downcast_array::<UInt64Array>(batch.column(8), "order_timestamp")?;
        let address_hashes =
            downcast_array::<UInt64Array>(batch.column(9), "shipping_address_hash")?;

        // Convert u8 arrays back to enums
        let mut statuses = Vec::with_capacity(status_u8.len());
        let mut payment_methods = Vec::with_capacity(payment_u8.len());

        for i in 0..status_u8.len() {
            statuses.push(
                OrderStatus::try_from(status_u8.value(i))
                    .map_err(|e| PersistenceError::TypeConversion { message: e })?,
            );
            payment_methods.push(
                PaymentMethod::try_from(payment_u8.value(i))
                    .map_err(|e| PersistenceError::TypeConversion { message: e })?,
            );
        }

        // Reconstruct SoA from Arrow data
        Ok(OrderSoA {
            order_id: order_ids.values().to_vec(),
            customer_id: customer_ids.values().to_vec(),
            product_id: product_ids.values().to_vec(),
            quantity: quantities.values().to_vec(),
            unit_price: unit_prices.values().to_vec(),
            total_amount: total_amounts.values().to_vec(),
            status: statuses,
            payment_method: payment_methods,
            order_timestamp: timestamps.values().to_vec(),
            shipping_address_hash: address_hashes.values().to_vec(),
        })
    }
}

/// Persistent wrapper for OrderStore with Arrow-based storage
pub struct PersistentOrderStore {
    store: crate::OrderStore,
    persistence: ArrowPersistence<OrderSoA>,
}

impl PersistentOrderStore {
    /// Create a new persistent order store
    pub fn new() -> Self {
        Self {
            store: crate::OrderStore::new(),
            persistence: ArrowPersistence::new(),
        }
    }

    /// Create with initial capacity for better performance
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            store: crate::OrderStore::new(),
            persistence: ArrowPersistence::with_capacity(capacity),
        }
    }

    /// Add an order and persist it
    pub async fn add(&mut self, order: Order) -> soa_persistence::Result<usize> {
        let index = self.store.add(order);

        // Auto-persist after each addition
        self.persistence.save(self.store.kernel()).await?;

        Ok(index)
    }

    /// Add multiple orders efficiently in a batch
    pub async fn add_batch(&mut self, orders: Vec<Order>) -> soa_persistence::Result<Vec<usize>> {
        let mut indices = Vec::with_capacity(orders.len());

        for order in orders {
            let index = self.store.add(order);
            indices.push(index);
        }

        // Single persistence operation for the batch
        self.persistence.save(self.store.kernel()).await?;

        Ok(indices)
    }

    /// Load data from persistence into the store
    pub async fn load_from_storage(&mut self) -> soa_persistence::Result<bool> {
        if let Some(soa) = self.persistence.load().await? {
            // Replace current store content with loaded data
            *self.store.kernel_mut() = soa;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Save current store state to persistence
    pub async fn save_to_storage(&mut self) -> soa_persistence::Result<()> {
        self.persistence.save(self.store.kernel()).await
    }

    /// Append current store state to persistence (for backup scenarios)
    pub async fn append_to_storage(&mut self) -> soa_persistence::Result<()> {
        self.persistence.append(self.store.kernel()).await
    }

    /// Query persistent storage with a predicate
    pub async fn query_storage<F>(&self, predicate: F) -> soa_persistence::Result<Option<OrderSoA>>
    where
        F: Fn(&OrderSoA) -> bool + Send + Sync,
    {
        self.persistence.query(predicate).await
    }

    /// Get count of records in persistent storage
    pub async fn storage_count(&self) -> soa_persistence::Result<usize> {
        self.persistence.count().await
    }

    /// Clear both in-memory store and persistent storage
    pub async fn clear_all(&mut self) -> soa_persistence::Result<()> {
        // Clear in-memory store
        self.store = crate::OrderStore::new();

        // Clear persistent storage
        self.persistence.clear().await?;

        Ok(())
    }

    /// Get memory usage statistics
    pub async fn memory_stats(&self) -> soa_persistence::Result<MemoryStats> {
        self.persistence.memory_usage()
    }

    /// Check if both store and persistence are empty
    pub async fn is_empty(&self) -> soa_persistence::Result<bool> {
        let store_empty = self.store.kernel().is_empty();
        let storage_empty = self.persistence.is_empty().await?;
        Ok(store_empty && storage_empty)
    }

    // Delegate methods to the inner store for compatibility

    /// Get access to the underlying SoA kernel (read-only)
    pub fn kernel(&self) -> &OrderSoA {
        self.store.kernel()
    }

    /// Get mutable access to the underlying SoA kernel
    pub fn kernel_mut(&mut self) -> &mut OrderSoA {
        self.store.kernel_mut()
    }

    /// Get the current length of the in-memory store
    pub fn len(&self) -> usize {
        self.store.kernel().len()
    }

    /// Check if the in-memory store is empty
    pub fn is_memory_empty(&self) -> bool {
        self.store.kernel().is_empty()
    }
}
