use soa_macros::{SoA, SoAStore};
use std::collections::HashMap;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

    pub fn new_with_payment(
        order_id: u64,
        customer_id: u64,
        product_id: u64,
        quantity: u32,
        unit_price: f64,
        payment_method: PaymentMethod,
    ) -> Self {
        let mut order = Self::new(order_id, customer_id, product_id, quantity, unit_price);
        order.payment_method = payment_method;
        order
    }

    pub fn with_status(mut self, status: OrderStatus) -> Self {
        self.status = status;
        self
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

    // Business query: Customer lifetime value
    pub fn customer_lifetime_values(&self) -> HashMap<u64, f64> {
        let mut customer_values: HashMap<u64, f64> = HashMap::new();

        for order in self.store.kernel().iter() {
            if matches!(order.status, OrderStatus::Delivered) {
                *customer_values.entry(*order.customer_id).or_insert(0.0) += *order.total_amount;
            }
        }

        customer_values
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

    // Advanced business query: Product performance analysis
    pub fn product_performance(&self) -> HashMap<u64, (u32, f64, f64)> {
        let mut product_stats: HashMap<u64, (u32, f64, f64)> = HashMap::new();

        for order in self.store.kernel().iter() {
            let entry = product_stats
                .entry(*order.product_id)
                .or_insert((0, 0.0, 0.0));
            entry.0 += *order.quantity; // total quantity sold
            entry.1 += *order.total_amount; // total revenue
            if matches!(order.status, OrderStatus::Delivered) {
                entry.2 += *order.total_amount; // delivered revenue
            }
        }

        product_stats
    }

    // Business query: Payment method distribution
    pub fn payment_method_distribution(&self) -> HashMap<PaymentMethod, u32> {
        let mut method_counts: HashMap<PaymentMethod, u32> = HashMap::new();

        for order in self.store.kernel().iter() {
            *method_counts.entry(*order.payment_method).or_insert(0) += 1;
        }

        method_counts
    }

    // Advanced analytics: Order status funnel
    pub fn order_status_funnel(&self) -> HashMap<OrderStatus, u32> {
        let mut status_counts: HashMap<OrderStatus, u32> = HashMap::new();

        for order in self.store.kernel().iter() {
            *status_counts.entry(*order.status).or_insert(0) += 1;
        }

        status_counts
    }

    // Business insights: High-value customer detection
    pub fn high_value_customers(&self, min_lifetime_value: f64) -> Vec<(u64, f64, u32)> {
        let lifetime_values = self.customer_lifetime_values();
        let order_counts = self.top_customers_by_volume(usize::MAX);
        let order_count_map: HashMap<u64, u32> = order_counts.into_iter().collect();

        lifetime_values
            .into_iter()
            .filter(|(_, value)| *value >= min_lifetime_value)
            .map(|(customer_id, value)| {
                let order_count = order_count_map.get(&customer_id).copied().unwrap_or(0);
                (customer_id, value, order_count)
            })
            .collect()
    }

    pub fn get_store(&self) -> &OrderStore {
        &self.store
    }
}

// Extension methods for the store to demonstrate advanced SoA usage
impl OrderStore {
    pub fn fraud_detection_scan(&self) -> Vec<u64> {
        let recent_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (7 * 24 * 60 * 60); // 7 days ago

        self.kernel()
            .iter()
            .filter(|order| {
                // Complex business rules benefit from SoA performance
                *order.total_amount > 1000.0
                    && matches!(order.payment_method, PaymentMethod::CreditCard)
                    && *order.order_timestamp > recent_timestamp
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

    // Demonstrate efficient filtering and aggregation
    pub fn daily_revenue_trend(&self, days: u64) -> Vec<(u64, f64)> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let start_timestamp = now - (days * 24 * 60 * 60);

        let mut daily_revenue: HashMap<u64, f64> = HashMap::new();

        for order in self.kernel().iter() {
            if *order.order_timestamp >= start_timestamp
                && matches!(order.status, OrderStatus::Delivered)
            {
                let day = *order.order_timestamp / (24 * 60 * 60);
                *daily_revenue.entry(day).or_insert(0.0) += *order.total_amount;
            }
        }

        let mut trend: Vec<_> = daily_revenue.into_iter().collect();
        trend.sort_by_key(|&(day, _)| day);
        trend
    }
}

// Parallel processing demonstrations
pub struct ParallelOrderAnalytics {
    sharded_store: OrderShardedStore,
}

impl ParallelOrderAnalytics {
    pub fn new() -> Self {
        Self {
            sharded_store: OrderShardedStore::with_shards(16, 1000),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        self.sharded_store.add(order);
    }

    #[cfg(feature = "parallel")]
    pub fn parallel_revenue_by_payment_method(&self) -> HashMap<PaymentMethod, f64> {
        use std::sync::Mutex;

        let revenue_map = Mutex::new(HashMap::new());

        (0..self.sharded_store.shard_count())
            .into_par_iter()
            .for_each(|shard_id| {
                let mut local_revenue = HashMap::new();

                for order in self.sharded_store.shard(shard_id).iter() {
                    let revenue = match order.status {
                        OrderStatus::Delivered => *order.total_amount,
                        _ => 0.0,
                    };
                    *local_revenue.entry(*order.payment_method).or_insert(0.0) += revenue;
                }

                // Merge local results into global map
                let mut global_map = revenue_map.lock().unwrap();
                for (method, revenue) in local_revenue {
                    *global_map.entry(method).or_insert(0.0) += revenue;
                }
            });

        revenue_map.into_inner().unwrap()
    }

    pub fn shard_count(&self) -> usize {
        self.sharded_store.shard_count()
    }

    pub fn add_bulk_orders(&mut self, orders: Vec<Order>) {
        for order in orders {
            self.add_order(order);
        }
    }
}

#[cfg(feature = "parallel")]
pub fn demonstrate_parallel_processing() {
    println!("🚀 Parallel Processing Demo with Sharded Storage");
    println!("================================================\n");

    let mut parallel_analytics = ParallelOrderAnalytics::new();

    println!(
        "📦 Adding 1000 sample orders across {} shards...",
        parallel_analytics.shard_count()
    );

    // Generate a larger dataset for parallel processing demonstration
    let orders: Vec<Order> = (1..=1000)
        .map(|i| {
            let customer_id = 1000 + (i % 50); // 50 different customers
            let product_id = 3000 + (i % 20); // 20 different products
            let payment_method = match i % 3 {
                0 => PaymentMethod::CreditCard,
                1 => PaymentMethod::PayPal,
                _ => PaymentMethod::BankTransfer,
            };
            let status = match i % 5 {
                0 | 1 => OrderStatus::Delivered, // 40% delivered
                2 => OrderStatus::Shipped,       // 20% shipped
                3 => OrderStatus::Processing,    // 20% processing
                _ => OrderStatus::Pending,       // 20% pending
            };

            Order::new_with_payment(
                i as u64,
                customer_id,
                product_id,
                1 + (i % 5) as u32,
                10.0 + (i % 100) as f64,
                payment_method,
            )
            .with_status(status)
        })
        .collect();

    parallel_analytics.add_bulk_orders(orders);
    println!("✅ Added 1000 orders distributed across shards\n");

    // Demonstrate parallel revenue analysis
    println!("💰 Parallel Revenue Analysis:");
    let start_time = std::time::Instant::now();
    let parallel_revenue = parallel_analytics.parallel_revenue_by_payment_method();
    let parallel_duration = start_time.elapsed();

    let total_parallel_revenue: f64 = parallel_revenue.values().sum();
    for (method, revenue) in &parallel_revenue {
        println!("  {:?}: ${:.2}", method, revenue);
    }
    println!("  Total Revenue: ${:.2}", total_parallel_revenue);
    println!("  Parallel processing time: {:?}\n", parallel_duration);

    println!("🎯 Parallel Processing Benefits:");
    println!(
        "  ✅ {} shards processed concurrently",
        parallel_analytics.shard_count()
    );
    println!("  ✅ Work distributed across CPU cores");
    println!("  ✅ Cache-efficient per-shard processing");
    println!("  ✅ Reduced contention with shard isolation");
    println!("  ✅ Scalable to large datasets");

    println!("\n💡 Sharded SoA provides:");
    println!("  • Parallel processing capabilities");
    println!("  • Cache-friendly data access within shards");
    println!("  • Reduced lock contention");
    println!("  • Linear scalability with core count");
}

#[cfg(not(feature = "parallel"))]
pub fn demonstrate_parallel_processing() {
    println!("🚀 Parallel Processing Demo");
    println!("===========================\n");
    println!("⚠️  Parallel processing features not enabled.");
    println!("    To enable parallel processing, run:");
    println!("    cargo run --package example_app_advanced --features parallel");
    println!("\n💡 This would demonstrate:");
    println!("  • Sharded storage with 16 concurrent shards");
    println!("  • Parallel analytics across CPU cores");
    println!("  • Rayon-powered parallel iterators");
    println!("  • Performance scaling with core count");
}

fn main() {
    println!("🏪 Advanced E-commerce Order Analytics Demo");
    println!("🔄 Combining Domain-Driven Design with Structure of Arrays Performance");
    println!("================================================================\n");

    let mut analytics = OrderAnalytics::new();

    println!("📦 Adding sample orders...");

    // Add comprehensive sample data demonstrating various scenarios
    analytics.add_order(Order::new_with_payment(
        1001,
        501,
        2001,
        2,
        29.99,
        PaymentMethod::PayPal,
    ));
    analytics.add_order(Order::new_with_payment(
        1002,
        502,
        2002,
        1,
        149.99,
        PaymentMethod::CreditCard,
    ));
    analytics.add_order(Order::new_with_payment(
        1003,
        501,
        2003,
        3,
        19.99,
        PaymentMethod::CreditCard,
    ));

    // Add delivered orders for revenue calculation
    analytics.add_order(
        Order::new_with_payment(1004, 503, 2001, 1, 29.99, PaymentMethod::CreditCard)
            .with_status(OrderStatus::Delivered),
    );
    analytics.add_order(
        Order::new_with_payment(1005, 501, 2004, 5, 9.99, PaymentMethod::BankTransfer)
            .with_status(OrderStatus::Delivered),
    );

    // High-value orders
    analytics.add_order(
        Order::new_with_payment(1006, 505, 2005, 1, 1299.99, PaymentMethod::CreditCard)
            .with_status(OrderStatus::Delivered),
    );

    // Add some processing and shipped orders
    analytics.add_order(
        Order::new_with_payment(1007, 502, 2002, 2, 75.00, PaymentMethod::PayPal)
            .with_status(OrderStatus::Processing),
    );
    analytics.add_order(
        Order::new_with_payment(1008, 504, 2003, 1, 45.00, PaymentMethod::BankTransfer)
            .with_status(OrderStatus::Shipped),
    );

    // Add cancelled order
    analytics.add_order(
        Order::new_with_payment(1009, 506, 2006, 3, 25.00, PaymentMethod::CreditCard)
            .with_status(OrderStatus::Cancelled),
    );

    println!("✅ Added 9 orders to the store\n");

    // Business Analytics Demonstrations
    println!("💰 Revenue Analysis:");
    let revenue_by_method = analytics.revenue_by_payment_method();
    let total_revenue: f64 = revenue_by_method.values().sum();

    for (method, revenue) in &revenue_by_method {
        println!("  {:?}: ${:.2}", method, revenue);
    }
    println!("  Total Revenue: ${:.2}\n", total_revenue);

    println!("👥 Customer Analysis:");
    let top_customers = analytics.top_customers_by_volume(5);
    let customer_values = analytics.customer_lifetime_values();

    for (i, (customer_id, order_count)) in top_customers.iter().enumerate() {
        let lifetime_value = customer_values.get(customer_id).unwrap_or(&0.0);
        println!(
            "  {}. Customer {}: {} orders, ${:.2} lifetime value",
            i + 1,
            customer_id,
            order_count,
            lifetime_value
        );
    }
    println!();

    println!("📊 Order Status Funnel:");
    let status_funnel = analytics.order_status_funnel();
    for (status, count) in &status_funnel {
        println!("  {:?}: {} orders", status, count);
    }
    println!();

    println!("💳 Payment Method Distribution:");
    let payment_distribution = analytics.payment_method_distribution();
    for (method, count) in &payment_distribution {
        println!("  {:?}: {} orders", method, count);
    }
    println!();

    println!("🎯 Product Performance Analysis:");
    let product_performance = analytics.product_performance();
    for (product_id, (total_qty, total_revenue, delivered_revenue)) in &product_performance {
        let delivery_rate = if *total_revenue > 0.0 {
            delivered_revenue / total_revenue * 100.0
        } else {
            0.0
        };
        println!(
            "  Product {}: {} units sold, ${:.2} total revenue, {:.1}% delivered",
            product_id, total_qty, total_revenue, delivery_rate
        );
    }
    println!();

    println!("💎 High-Value Customer Detection:");
    let high_value_customers = analytics.high_value_customers(100.0);
    if high_value_customers.is_empty() {
        println!("  No high-value customers found (minimum $100 lifetime value)");
    } else {
        for (customer_id, lifetime_value, order_count) in &high_value_customers {
            println!(
                "  Customer {}: ${:.2} lifetime value, {} orders",
                customer_id, lifetime_value, order_count
            );
        }
    }
    println!();

    println!("🔍 Fraud Detection Analysis:");
    let suspicious_orders = analytics.get_store().fraud_detection_scan();
    if suspicious_orders.is_empty() {
        println!("  No suspicious orders detected");
    } else {
        println!(
            "  Suspicious orders (high value, recent, credit card): {:?}",
            suspicious_orders
        );
    }
    println!();

    println!("📈 Daily Revenue Trend (last 30 days):");
    let revenue_trend = analytics.get_store().daily_revenue_trend(30);
    if revenue_trend.is_empty() {
        println!("  No revenue data for the specified period");
    } else {
        for (day, revenue) in revenue_trend.iter().take(5) {
            // Show first 5 days
            println!("  Day {}: ${:.2}", day, revenue);
        }
        if revenue_trend.len() > 5 {
            println!("  ... and {} more days", revenue_trend.len() - 5);
        }
    }
    println!();

    println!("🚀 Performance Benefits:");
    println!("  ✅ Cache-efficient columnar queries (SoA layout)");
    println!("  ✅ Type-safe domain modeling with business logic");
    println!("  ✅ Zero-cost abstraction - no runtime overhead");
    println!("  ✅ Thread-safe stores with copy-on-write semantics");
    println!("  ✅ Sharded storage for concurrent access (16 shards configured)");

    println!("\n💡 This demonstrates how SoA provides:");
    println!("  • Domain-driven APIs for business logic");
    println!("  • Data-oriented performance for analytics");
    println!("  • Best of both paradigms in a single framework");

    println!("\n{}", "=".repeat(60));

    // Demonstrate parallel processing capabilities
    demonstrate_parallel_processing();

    println!("\n🔬 For detailed performance analysis, run:");
    println!("  cargo bench  # Criterion-based benchmarks");
    println!("  cargo bench  # Criterion performance benchmarks");
    println!("  cargo run --package example_app_advanced --features parallel  # Parallel demo");
}
