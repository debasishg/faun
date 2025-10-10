#[allow(unused_imports)] // HashMap is used in return types of methods called below
use std::collections::HashMap;

// Import all types from lib.rs - no more duplicates!
use example_app::{Order, OrderStatus, PaymentMethod};

fn main() {
    ddd_repository_demo();
}

/// Demonstration of DDD Repository pattern using generated OrderStore
fn ddd_repository_demo() {
    println!("\n🏛️ DDD Repository Pattern with SoA Backend");
    println!("{}", "=".repeat(60));

    // Import the OrderStore types - other types already available from global imports
    use example_app::{OrderShardedStore, OrderStore};

    // 1. Domain-Driven Design Repository Pattern
    let mut order_repository = OrderStore::new();

    println!("📦 Adding orders to DDD repository...");

    // Add orders using DDD-style add method
    order_repository.add(
        Order::new_with_payment(1, 1001, 2001, 2, 50.0, PaymentMethod::CreditCard)
            .with_status(OrderStatus::Delivered),
    );

    order_repository.add(
        Order::new_with_payment(2, 1002, 2002, 1, 75.0, PaymentMethod::PayPal)
            .with_status(OrderStatus::Delivered),
    );

    order_repository.add(
        Order::new_with_payment(3, 1003, 2003, 3, 25.0, PaymentMethod::BankTransfer)
            .with_status(OrderStatus::Pending),
    );

    println!("✅ Added {} orders", order_repository.kernel().len());

    // 2. Query using SoA performance with DDD API
    println!("\n💰 Repository Analytics:");
    let kernel = order_repository.kernel();
    let delivered_revenue: f64 = kernel
        .iter()
        .filter(|order| matches!(*order.status, OrderStatus::Delivered))
        .map(|order| *order.total_amount)
        .sum();

    println!("  Total delivered revenue: ${:.2}", delivered_revenue);

    // 3. Sharded Repository for High Performance
    println!("\n🚀 High-Performance Sharded Repository:");
    let mut sharded_repo = OrderShardedStore::with_shards(4, 1000);

    // Add orders - automatically sharded by order_id
    for i in 100..110 {
        sharded_repo.add(
            Order::new_with_payment(
                i,
                1000 + i,
                2000 + (i % 10),
                1,
                100.0,
                PaymentMethod::CreditCard,
            )
            .with_status(OrderStatus::Delivered),
        );
    }

    println!(
        "  Added 10 orders across {} shards",
        sharded_repo.shard_count()
    );

    // Process each shard independently (great for parallel processing)
    for shard_id in 0..sharded_repo.shard_count() {
        let shard = sharded_repo.shard(shard_id);
        if shard.len() > 0 {
            println!("  Shard {}: {} orders", shard_id, shard.len());
        }
    }

    println!("\n✨ Key Benefits:");
    println!("  • DDD-style API: Clean, intuitive domain methods");
    println!("  • SoA Performance: Optimized memory layout underneath");
    println!("  • Thread Safety: Arc-based sharing with copy-on-write");
    println!("  • Sharding: Automatic partitioning for parallel processing");
    println!("  • Zero Cost: Repository wrapper has no runtime overhead");
}
