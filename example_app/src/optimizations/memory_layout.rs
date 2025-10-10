use crate::{Order, OrderStatus, PaymentMethod};
use std::collections::HashMap;

/// Optimized memory layout that interleaves frequently accessed fields
/// This reduces the "3 cache lines per record" problem in aggregation workloads
#[derive(Debug, Clone)]
pub struct OptimizedOrderLayout {
    // Pack frequently accessed fields together (16 bytes per element)
    // This puts related data in the same cache lines
    status_payment_amount: Vec<(OrderStatus, PaymentMethod, f64)>,

    // Secondary fields for business logic (16 bytes per element)
    customer_product: Vec<(u64, u64)>,

    // Other fields that are less frequently accessed together
    other_fields: Vec<OrderMetadata>,
}

#[derive(Debug, Clone, Copy)]
pub struct OrderMetadata {
    pub order_id: u64,
    pub quantity: u32,
    pub unit_price: f64,
    pub order_timestamp: u64,
    pub shipping_address_hash: u64,
}

impl OptimizedOrderLayout {
    pub fn new() -> Self {
        Self {
            status_payment_amount: Vec::new(),
            customer_product: Vec::new(),
            other_fields: Vec::new(),
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            status_payment_amount: Vec::with_capacity(cap),
            customer_product: Vec::with_capacity(cap),
            other_fields: Vec::with_capacity(cap),
        }
    }

    pub fn len(&self) -> usize {
        self.status_payment_amount.len()
    }

    pub fn is_empty(&self) -> bool {
        self.status_payment_amount.is_empty()
    }

    pub fn push(&mut self, order: Order) {
        self.status_payment_amount
            .push((order.status, order.payment_method, order.total_amount));
        self.customer_product
            .push((order.customer_id, order.product_id));
        self.other_fields.push(OrderMetadata {
            order_id: order.order_id,
            quantity: order.quantity,
            unit_price: order.unit_price,
            order_timestamp: order.order_timestamp,
            shipping_address_hash: order.shipping_address_hash,
        });
    }

    /// Optimized revenue analysis - all needed fields in same cache lines
    pub fn revenue_by_payment_method(&self) -> HashMap<PaymentMethod, f64> {
        let mut results = HashMap::new();

        // This is the key optimization: status, payment_method, and amount
        // are all in the same cache line, eliminating the multiple cache line
        // accesses that made naive SoA slower than AoS for aggregation
        for &(status, payment, amount) in &self.status_payment_amount {
            if matches!(status, OrderStatus::Delivered) {
                *results.entry(payment).or_insert(0.0) += amount;
            }
        }

        results
    }

    /// Customer lifetime value with optimized memory access
    pub fn customer_lifetime_values(&self) -> HashMap<u64, f64> {
        let mut results = HashMap::new();

        // We need both customer_id and the aggregation fields
        // This requires accessing two arrays, but they're both optimally laid out
        for i in 0..self.len() {
            let (status, _, amount) = self.status_payment_amount[i];
            let (customer_id, _) = self.customer_product[i];

            if matches!(status, OrderStatus::Delivered) {
                *results.entry(customer_id).or_insert(0.0) += amount;
            }
        }

        results
    }

    /// Product performance analysis
    pub fn product_performance(&self) -> HashMap<u64, (u32, f64, f64)> {
        let mut results = HashMap::new();

        for i in 0..self.len() {
            let (status, _, amount) = self.status_payment_amount[i];
            let (_, product_id) = self.customer_product[i];
            let metadata = self.other_fields[i];

            let entry = results.entry(product_id).or_insert((0, 0.0, 0.0));
            entry.0 += metadata.quantity;
            entry.1 += amount;

            if matches!(status, OrderStatus::Delivered) {
                entry.2 += amount;
            }
        }

        results
    }
}

impl Default for OptimizedOrderLayout {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert from standard SoA to optimized layout
impl From<&crate::OrderStore> for OptimizedOrderLayout {
    fn from(store: &crate::OrderStore) -> Self {
        let soa = store.kernel();
        let len = soa.len();
        let mut optimized = OptimizedOrderLayout::with_capacity(len);

        // Reconstruct orders and re-layout them optimally
        for i in 0..len {
            let view = soa.view(i);
            let order = Order {
                order_id: *view.order_id,
                customer_id: *view.customer_id,
                product_id: *view.product_id,
                quantity: *view.quantity,
                unit_price: *view.unit_price,
                total_amount: *view.total_amount,
                status: *view.status,
                payment_method: *view.payment_method,
                order_timestamp: *view.order_timestamp,
                shipping_address_hash: *view.shipping_address_hash,
            };
            optimized.push(order);
        }

        optimized
    }
}

/// Alternative layout: Hot/Cold field separation
/// Separates frequently accessed ("hot") fields from rarely accessed ("cold") fields
#[derive(Debug, Clone)]
pub struct HotColdOrderLayout {
    // Hot fields: accessed frequently in analytics
    hot_fields: Vec<HotOrderData>,

    // Cold fields: accessed rarely, stored separately to avoid cache pollution
    cold_fields: Vec<ColdOrderData>,
}

#[derive(Debug, Clone, Copy)]
pub struct HotOrderData {
    pub status: OrderStatus,
    pub payment_method: PaymentMethod,
    pub total_amount: f64,
    pub customer_id: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct ColdOrderData {
    pub order_id: u64,
    pub product_id: u64,
    pub quantity: u32,
    pub unit_price: f64,
    pub order_timestamp: u64,
    pub shipping_address_hash: u64,
}

impl HotColdOrderLayout {
    pub fn new() -> Self {
        Self {
            hot_fields: Vec::new(),
            cold_fields: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.hot_fields.len()
    }

    pub fn push(&mut self, order: Order) {
        self.hot_fields.push(HotOrderData {
            status: order.status,
            payment_method: order.payment_method,
            total_amount: order.total_amount,
            customer_id: order.customer_id,
        });

        self.cold_fields.push(ColdOrderData {
            order_id: order.order_id,
            product_id: order.product_id,
            quantity: order.quantity,
            unit_price: order.unit_price,
            order_timestamp: order.order_timestamp,
            shipping_address_hash: order.shipping_address_hash,
        });
    }

    /// Ultra-fast revenue analysis - only touches hot fields
    pub fn revenue_by_payment_method(&self) -> HashMap<PaymentMethod, f64> {
        let mut results = HashMap::new();

        // Only access hot fields, cold fields stay out of cache
        for hot in &self.hot_fields {
            if matches!(hot.status, OrderStatus::Delivered) {
                *results.entry(hot.payment_method).or_insert(0.0) += hot.total_amount;
            }
        }

        results
    }

    /// Customer analysis using only hot fields
    pub fn customer_lifetime_values(&self) -> HashMap<u64, f64> {
        let mut results = HashMap::new();

        for hot in &self.hot_fields {
            if matches!(hot.status, OrderStatus::Delivered) {
                *results.entry(hot.customer_id).or_insert(0.0) += hot.total_amount;
            }
        }

        results
    }
}

impl Default for HotColdOrderLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&crate::OrderStore> for HotColdOrderLayout {
    fn from(store: &crate::OrderStore) -> Self {
        let soa = store.kernel();
        let len = soa.len();
        let mut layout = HotColdOrderLayout::new();

        for i in 0..len {
            let view = soa.view(i);
            let order = Order {
                order_id: *view.order_id,
                customer_id: *view.customer_id,
                product_id: *view.product_id,
                quantity: *view.quantity,
                unit_price: *view.unit_price,
                total_amount: *view.total_amount,
                status: *view.status,
                payment_method: *view.payment_method,
                order_timestamp: *view.order_timestamp,
                shipping_address_hash: *view.shipping_address_hash,
            };
            layout.push(order);
        }

        layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Order;

    fn create_test_orders() -> Vec<Order> {
        vec![
            Order::new_with_payment(1, 100, 200, 2, 50.0, PaymentMethod::CreditCard)
                .with_status(OrderStatus::Delivered),
            Order::new_with_payment(2, 101, 201, 1, 75.0, PaymentMethod::PayPal)
                .with_status(OrderStatus::Delivered),
            Order::new_with_payment(3, 100, 202, 3, 25.0, PaymentMethod::BankTransfer)
                .with_status(OrderStatus::Pending),
        ]
    }

    #[test]
    fn test_optimized_layout() {
        let mut layout = OptimizedOrderLayout::new();
        for order in create_test_orders() {
            layout.push(order);
        }

        assert_eq!(layout.len(), 3);

        let revenue = layout.revenue_by_payment_method();
        assert_eq!(revenue.len(), 2); // Only delivered orders

        let customers = layout.customer_lifetime_values();
        assert!(customers.contains_key(&100));
        assert!(customers.contains_key(&101));
    }

    #[test]
    fn test_hot_cold_layout() {
        let mut layout = HotColdOrderLayout::new();
        for order in create_test_orders() {
            layout.push(order);
        }

        assert_eq!(layout.len(), 3);

        let revenue = layout.revenue_by_payment_method();
        assert_eq!(revenue.len(), 2);
    }
}
