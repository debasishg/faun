use soa_macros::{SoA, SoAStore};
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PaymentMethod {
    CreditCard,
    PayPal,
    BankTransfer,
}

// Domain entity - looks like traditional DDD
#[derive(SoA, SoAStore, Debug, Copy, Clone)]
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

// Business logic using domain concepts
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
            total_amount: unit_price * quantity as f64,
            status: OrderStatus::Pending,
            payment_method: PaymentMethod::CreditCard,
            order_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            shipping_address_hash: 0, // Would be computed from actual address
        }
    }

    pub fn revenue(&self) -> f64 {
        match self.status {
            OrderStatus::Delivered => self.total_amount,
            _ => 0.0,
        }
    }
}

// High-level business operations
pub struct OrderAnalytics {
    store: OrderStore,
}

impl OrderAnalytics {
    pub fn new() -> Self {
        Self {
            store: OrderStore::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        self.store.add(order);
    }

    // Business query: Revenue by payment method
    // Uses domain concepts but gets SoA performance automatically
    pub fn revenue_by_payment_method(&self) -> HashMap<PaymentMethod, f64> {
        let mut revenue_map = HashMap::new();

        // This loop is cache-efficient thanks to SoA layout!
        for order in self.store.kernel().iter() {
            let revenue = match order.status {
                OrderStatus::Delivered => *order.total_amount,
                _ => 0.0,
            };
            *revenue_map.entry(*order.payment_method).or_insert(0.0) += revenue;
        }

        revenue_map
    }

    // Business query: Top customers by order volume
    pub fn top_customers_by_volume(&self, limit: usize) -> Vec<(u64, u32)> {
        let mut customer_orders: HashMap<u64, u32> = HashMap::new();

        // Efficient iteration over customer_id column only
        for order in self.store.kernel().iter() {
            *customer_orders.entry(*order.customer_id).or_insert(0) += 1;
        }

        let mut customers: Vec<_> = customer_orders.into_iter().collect();
        customers.sort_by(|a, b| b.1.cmp(&a.1));
        customers.truncate(limit);
        customers
    }

    // Business query: Orders pending for more than N days
    pub fn orders_pending_too_long(&self, days_threshold: u64) -> Vec<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let threshold = now - (days_threshold * 24 * 60 * 60);

        self.store
            .kernel()
            .iter()
            .filter(|order| {
                matches!(order.status, OrderStatus::Pending | OrderStatus::Processing)
                    && *order.order_timestamp < threshold
            })
            .map(|order| *order.order_id)
            .collect()
    }
}

// Extended business logic with SoA performance
impl OrderStore {
    pub fn fraud_detection_scan(&self) -> Vec<u64> {
        self.kernel()
            .iter()
            .filter(|order| {
                // Complex business rules benefit from SoA performance
                *order.total_amount > 1000.0
                    && matches!(order.payment_method, PaymentMethod::CreditCard)
                    && *order.order_timestamp > self.recent_timestamp()
            })
            .map(|order| *order.order_id)
            .collect()
    }

    pub fn customer_lifetime_value(&self, customer_id: u64) -> f64 {
        self.kernel()
            .iter()
            .filter(|order| *order.customer_id == customer_id)
            .filter(|order| matches!(order.status, OrderStatus::Delivered))
            .map(|order| *order.total_amount)
            .sum()
    }

    fn recent_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (7 * 24 * 60 * 60) // 7 days ago
    }
}

fn main() {
    println!("🏪 E-commerce Order Analytics Demo");
    println!("🔄 Combining Domain-Driven Design with Structure of Arrays Performance\n");

    let mut analytics = OrderAnalytics::new();

    // Add sample orders using domain objects
    println!("📦 Adding sample orders...");
    analytics.add_order(Order::new(1001, 501, 2001, 2, 29.99));
    analytics.add_order(Order::new(1002, 502, 2002, 1, 149.99));
    analytics.add_order(Order::new(1003, 501, 2003, 3, 19.99));
    analytics.add_order(Order::new(1004, 504, 2004, 1, 799.99));
    analytics.add_order(Order::new(1005, 502, 2001, 5, 29.99));

    // Simulate some delivered orders
    let mut delivered_order1 = Order::new(1006, 503, 2001, 1, 29.99);
    delivered_order1.status = OrderStatus::Delivered;
    delivered_order1.payment_method = PaymentMethod::PayPal;
    analytics.add_order(delivered_order1);

    let mut delivered_order2 = Order::new(1007, 501, 2005, 2, 75.50);
    delivered_order2.status = OrderStatus::Delivered;
    delivered_order2.payment_method = PaymentMethod::BankTransfer;
    analytics.add_order(delivered_order2);

    let mut high_value_order = Order::new(1008, 505, 2006, 1, 1299.99);
    high_value_order.status = OrderStatus::Delivered;
    high_value_order.payment_method = PaymentMethod::CreditCard;
    analytics.add_order(high_value_order);

    println!(
        "✅ Added {} orders to the store\n",
        analytics.store.kernel().len()
    );

    // Demonstrate business queries using domain language
    println!("💰 Revenue Analysis:");
    let revenue_by_method = analytics.revenue_by_payment_method();
    for (method, revenue) in &revenue_by_method {
        println!("  {:?}: ${:.2}", method, revenue);
    }
    let total_revenue: f64 = revenue_by_method.values().sum();
    println!("  Total Revenue: ${:.2}\n", total_revenue);

    println!("👥 Customer Analysis:");
    let top_customers = analytics.top_customers_by_volume(3);
    for (i, (customer_id, order_count)) in top_customers.iter().enumerate() {
        let clv = analytics.store.customer_lifetime_value(*customer_id);
        println!(
            "  {}. Customer {}: {} orders, ${:.2} lifetime value",
            i + 1,
            customer_id,
            order_count,
            clv
        );
    }
    println!();

    println!("⚠️  Operations Analysis:");
    let overdue_orders = analytics.orders_pending_too_long(0); // Any pending orders
    if overdue_orders.is_empty() {
        println!("  ✅ No overdue orders");
    } else {
        println!("  📋 Overdue orders: {:?}", overdue_orders);
    }

    let suspicious_orders = analytics.store.fraud_detection_scan();
    if suspicious_orders.is_empty() {
        println!("  ✅ No suspicious orders detected");
    } else {
        println!("  🚨 Suspicious orders for review: {:?}", suspicious_orders);
    }
    println!();

    // Demonstrate sharded storage for high-performance scenarios
    println!("🚀 High-Performance Sharded Processing:");
    let mut sharded_store = OrderShardedStore::with_shards(4, 100);

    // Add orders to sharded store
    for i in 2000..2020 {
        let order = Order::new(i, 600 + (i % 5), 3000 + (i % 10), 1, 25.0 + (i % 50) as f64);
        sharded_store.add(order);
    }

    // Process each shard separately (could be done in parallel)
    println!("  Processing {} shards:", sharded_store.shard_count());
    let mut total_orders = 0;
    for shard_id in 0..sharded_store.shard_count() {
        let shard = sharded_store.shard(shard_id);
        let orders_in_shard = shard.len();
        total_orders += orders_in_shard;

        let shard_revenue: f64 = shard
            .iter()
            .filter(|order| matches!(order.status, OrderStatus::Delivered))
            .map(|order| *order.total_amount)
            .sum();

        println!(
            "    Shard {}: {} orders, ${:.2} potential revenue",
            shard_id, orders_in_shard, shard_revenue
        );
    }
    println!("  📊 Total orders across all shards: {}\n", total_orders);

    println!("🎯 Performance Insights:");
    println!("  • Data stored in columnar format (Structure of Arrays)");
    println!("  • Filtering operations access only relevant columns");
    println!("  • CPU cache utilization optimized for analytical queries");
    println!("  • Thread-safe stores with copy-on-write semantics");
    println!("  • Sharding enables parallel processing of large datasets");
    println!("\n✨ You wrote domain-focused code, but got data-oriented performance!");
}
