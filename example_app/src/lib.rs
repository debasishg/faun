use soa_macros::{SoA, SoAStore};
use std::collections::HashMap;

pub mod optimizations;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentMethod {
    CreditCard,
    PayPal,
    BankTransfer,
}

#[derive(Debug, Clone, Copy, SoA, SoAStore)]
#[soa_store(key = "order_id", shards = 16)]
pub struct Order {
    pub order_id: u64,
    pub customer_id: u64,
    pub product_id: u64,
    pub quantity: u32,
    pub unit_price: f64,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub payment_method: PaymentMethod,
    pub order_timestamp: u64,
    pub shipping_address_hash: u64,
}

impl Order {
    pub fn new(
        order_id: u64,
        customer_id: u64,
        product_id: u64,
        quantity: u32,
        unit_price: f64,
    ) -> Self {
        Self {
            order_id,
            customer_id,
            product_id,
            quantity,
            unit_price,
            total_amount: quantity as f64 * unit_price,
            status: OrderStatus::Pending,
            payment_method: PaymentMethod::CreditCard,
            order_timestamp: 1234567890,
            shipping_address_hash: 0xdeadbeef,
        }
    }

    pub fn new_with_payment(
        order_id: u64,
        customer_id: u64,
        product_id: u64,
        quantity: u32,
        unit_price: f64,
        payment_method: PaymentMethod,
    ) -> Self {
        Self {
            order_id,
            customer_id,
            product_id,
            quantity,
            unit_price,
            total_amount: quantity as f64 * unit_price,
            status: OrderStatus::Pending,
            payment_method,
            order_timestamp: 1234567890,
            shipping_address_hash: 0xdeadbeef,
        }
    }

    pub fn with_status(mut self, status: OrderStatus) -> Self {
        self.status = status;
        self
    }
}

// Extension methods for the macro-generated OrderSoA to demonstrate optimizations
impl OrderSoA {
    /// Traditional iterator-style approach (what users would naturally write)
    /// Uses the macro-generated iter() method but accesses multiple fields per iteration
    pub fn revenue_by_payment_method_iterator(&self) -> HashMap<PaymentMethod, f64> {
        let mut revenue_map = HashMap::new();

        // Using the generated iterator - clean API but potential cache misses
        for order_view in self.iter() {
            if matches!(*order_view.status, OrderStatus::Delivered) {
                *revenue_map.entry(*order_view.payment_method).or_insert(0.0) +=
                    *order_view.total_amount;
            }
        }

        revenue_map
    }

    /// Optimized direct field access approach
    /// Accesses the generated fields directly for better cache behavior
    pub fn revenue_by_payment_method_optimized(&self) -> HashMap<PaymentMethod, f64> {
        let mut revenue_map = HashMap::new();

        // Direct access to generated fields (self.status, self.payment_method, etc.)
        // Process in chunks to improve cache behavior
        const CHUNK_SIZE: usize = 1024; // Process in cache-friendly chunks
        let len = self.len();

        for chunk_start in (0..len).step_by(CHUNK_SIZE) {
            let chunk_end = (chunk_start + CHUNK_SIZE).min(len);

            // Process this chunk with better cache locality
            for i in chunk_start..chunk_end {
                if matches!(self.status[i], OrderStatus::Delivered) {
                    *revenue_map.entry(self.payment_method[i]).or_insert(0.0) +=
                        self.total_amount[i];
                }
            }
        }

        revenue_map
    }

    /// Memory-optimized layout that interleaves frequently accessed fields
    /// Demonstrates how to reorganize data for optimal cache usage
    pub fn revenue_by_payment_method_memory_optimized(&self) -> HashMap<PaymentMethod, f64> {
        // Create a temporary structure that packs the fields we need together
        // This simulates what an optimized memory layout would look like
        #[derive(Clone, Copy)]
        struct CompactOrder {
            status: OrderStatus,
            payment: PaymentMethod,
            amount: f64,
        }

        // Pack the data we need into a cache-friendly structure
        // Access the macro-generated fields directly
        let mut compact_orders: Vec<CompactOrder> = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            compact_orders.push(CompactOrder {
                status: self.status[i],
                payment: self.payment_method[i],
                amount: self.total_amount[i],
            });
        }

        // Now process the compact, cache-friendly data
        let mut revenue_map = HashMap::new();
        for order in &compact_orders {
            if matches!(order.status, OrderStatus::Delivered) {
                *revenue_map.entry(order.payment).or_insert(0.0) += order.amount;
            }
        }

        revenue_map
    }
}

// Simple AoS wrapper for comparison benchmarks
pub struct OrderAoS {
    pub orders: Vec<Order>,
}

impl OrderAoS {
    pub fn new() -> Self {
        Self { orders: Vec::new() }
    }

    pub fn push(&mut self, order: Order) {
        self.orders.push(order);
    }

    pub fn len(&self) -> usize {
        self.orders.len()
    }

    pub fn revenue_by_payment_method(&self) -> HashMap<PaymentMethod, f64> {
        let mut results = HashMap::new();

        for order in &self.orders {
            if matches!(order.status, OrderStatus::Delivered) {
                *results.entry(order.payment_method).or_insert(0.0) += order.total_amount;
            }
        }

        results
    }
}

impl Default for OrderAoS {
    fn default() -> Self {
        Self::new()
    }
}
