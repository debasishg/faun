use example_app::persistence::PersistentOrderStore;
use example_app::{Order, OrderStatus, PaymentMethod};

/// Demonstration of persistent SoA store with Arrow backend
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏛️ Persistent SoA Store with Arrow Backend Demo");
    println!("{}", "=".repeat(70));

    // 1. Create a persistent store
    let mut persistent_store = PersistentOrderStore::with_capacity(100);

    println!("📦 Creating persistent order store...");
    println!("✅ Store created with Arrow in-memory persistence");

    // 2. Add orders with automatic persistence
    println!("\n💾 Adding orders with automatic persistence:");

    let orders = vec![
        Order::new_with_payment(1, 1001, 2001, 2, 99.99, PaymentMethod::CreditCard)
            .with_status(OrderStatus::Delivered),
        Order::new_with_payment(2, 1002, 2002, 1, 149.50, PaymentMethod::PayPal)
            .with_status(OrderStatus::Shipped),
        Order::new_with_payment(3, 1003, 2003, 3, 75.25, PaymentMethod::BankTransfer)
            .with_status(OrderStatus::Delivered),
        Order::new_with_payment(4, 1004, 2004, 1, 299.99, PaymentMethod::CreditCard)
            .with_status(OrderStatus::Processing),
        Order::new_with_payment(5, 1005, 2005, 2, 199.00, PaymentMethod::PayPal)
            .with_status(OrderStatus::Delivered),
    ];

    // Add orders in batch for efficiency
    let indices = persistent_store.add_batch(orders).await?;
    println!("  Added {} orders at indices: {:?}", indices.len(), indices);

    // 3. Verify persistence
    let memory_count = persistent_store.len();
    let storage_count = persistent_store.storage_count().await?;

    println!("\n📊 Storage Status:");
    println!("  In-memory orders: {}", memory_count);
    println!("  Persisted orders: {}", storage_count);
    assert_eq!(memory_count, storage_count);

    // 4. Memory usage statistics
    let stats = persistent_store.memory_stats().await?;
    println!("\n💾 Memory Statistics:");
    println!("  Total bytes: {} bytes", stats.total_bytes);
    println!("  Total rows: {}", stats.total_rows);
    println!("  Number of batches: {}", stats.num_batches);
    println!("  Average batch size: {} bytes", stats.avg_batch_size);

    // 5. Query operations on persisted data
    println!("\n🔍 Querying persisted data:");

    // Query for delivered orders
    let delivered_query = persistent_store
        .query_storage(|soa| soa.status_raw_array().contains(&OrderStatus::Delivered))
        .await?;

    if let Some(delivered_orders) = delivered_query {
        let delivered_count = delivered_orders
            .status_raw_array()
            .iter()
            .filter(|&&status| status == OrderStatus::Delivered)
            .count();
        println!("  Found {} delivered orders in storage", delivered_count);

        // Calculate revenue from delivered orders
        let delivered_revenue: f64 = delivered_orders
            .status_raw_array()
            .iter()
            .zip(delivered_orders.total_amount_raw_array().iter())
            .filter(|(&status, _)| status == OrderStatus::Delivered)
            .map(|(_, &amount)| amount)
            .sum();

        println!("  Total delivered revenue: ${:.2}", delivered_revenue);
    }

    // 6. Demonstrate persistence across "sessions"
    println!("\n🔄 Simulating application restart:");

    // Save current state explicitly
    persistent_store.save_to_storage().await?;
    println!("  Saved current state to storage");

    // Create new store instance (simulating app restart)
    let new_session_store = PersistentOrderStore::new();
    println!("  Created new store instance (simulating restart)");

    // The new store starts empty
    assert!(new_session_store.is_empty());
    println!("  New store is empty: {}", new_session_store.len());

    // In a real scenario, you would load from shared persistent storage
    // For this demo, we show the persistence layer is working
    println!(
        "  Original store still has persisted data: {} orders",
        persistent_store.storage_count().await?
    );

    // 7. Advanced operations
    println!("\n⚡ Advanced operations:");

    // Add more data to show incremental persistence
    let new_order = Order::new_with_payment(6, 1006, 2006, 4, 399.99, PaymentMethod::CreditCard)
        .with_status(OrderStatus::Pending);

    persistent_store.add(new_order).await?;
    println!("  Added order #6, total orders: {}", persistent_store.len());

    // Append operation (for backup scenarios)
    persistent_store.append_to_storage().await?;
    println!("  Appended current state to storage");

    let final_count = persistent_store.storage_count().await?;
    println!("  Final persisted count: {}", final_count);

    // 8. Performance characteristics
    println!("\n🚀 Performance characteristics:");
    println!("  ✅ Zero-copy conversion between SoA and Arrow");
    println!("  ✅ Columnar storage optimized for analytics");
    println!("  ✅ In-memory persistence for fast access");
    println!("  ✅ Batch operations for efficiency");
    println!("  ✅ Domain API preserved with transparent persistence");

    // 9. Integration benefits
    println!("\n🔗 Integration benefits:");
    println!("  • Arrow format compatible with Polars, DataFusion");
    println!("  • Can extend to Parquet for disk persistence");
    println!("  • DuckDB integration for SQL queries");
    println!("  • Standard columnar format for data science tools");

    println!("\n✨ Demo completed successfully!");

    Ok(())
}

/// Helper function to demonstrate query patterns
#[allow(dead_code)]
async fn demonstrate_queries(
    store: &PersistentOrderStore,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Advanced Query Patterns:");

    // Query by payment method
    let credit_card_query = store
        .query_storage(|soa| {
            soa.payment_method_raw_array()
                .contains(&PaymentMethod::CreditCard)
        })
        .await?;

    if let Some(cc_orders) = credit_card_query {
        let cc_count = cc_orders
            .payment_method_raw_array()
            .iter()
            .filter(|&&method| method == PaymentMethod::CreditCard)
            .count();
        println!("  Credit card orders: {}", cc_count);
    }

    // Query high-value orders
    let high_value_query = store
        .query_storage(|soa| {
            soa.total_amount_raw_array()
                .iter()
                .any(|&amount| amount > 200.0)
        })
        .await?;

    if let Some(high_value) = high_value_query {
        let high_value_count = high_value
            .total_amount_raw_array()
            .iter()
            .filter(|&&amount| amount > 200.0)
            .count();
        println!("  High-value orders (>$200): {}", high_value_count);
    }

    Ok(())
}
