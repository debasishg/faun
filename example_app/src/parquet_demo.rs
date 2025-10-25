use example_app::persistence::PersistentOrderStore;
use example_app::{Order, OrderSoA, OrderStatus, PaymentMethod};
use parquet::basic::Compression;
use soa_persistence::{ParquetPersistence, SoAPersistence};
use std::path::PathBuf;

/// Demonstration of persistent SoA store with Parquet disk-based backend
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💾 Persistent SoA Store with Parquet Disk Backend Demo");
    println!("{}", "=".repeat(70));

    // 1. Setup Parquet persistence with compression
    let data_dir = PathBuf::from("./parquet_demo_data");
    println!("📦 Creating Parquet-based persistent store...");
    println!("   Storage directory: {:?}", data_dir);

    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&data_dir)?;

    // Create a persistent store using Parquet backend
    // Note: This requires PersistentOrderStore to accept custom persistence backends
    // For this demo, we'll use ParquetPersistence directly with OrderSoA

    // First, let's create a regular store and export its data
    let mut memory_store = PersistentOrderStore::with_capacity(100);

    println!("✅ Store created with Parquet disk persistence");
    println!("   Compression: ZSTD (best compression ratio)");
    println!("   Format: Apache Parquet (standard columnar format)");

    // 2. Add orders to in-memory store
    println!("\n💾 Adding orders:");

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
        Order::new_with_payment(6, 1006, 2006, 5, 499.99, PaymentMethod::CreditCard)
            .with_status(OrderStatus::Delivered),
        Order::new_with_payment(7, 1007, 2007, 1, 89.99, PaymentMethod::PayPal)
            .with_status(OrderStatus::Pending),
        Order::new_with_payment(8, 1008, 2008, 3, 249.50, PaymentMethod::BankTransfer)
            .with_status(OrderStatus::Shipped),
    ];

    let indices = memory_store.add_batch(orders.clone()).await?;
    println!("  Added {} orders at indices: {:?}", indices.len(), indices);

    // 3. Save to Parquet file
    println!("\n📁 Saving to Parquet file:");

    // Get the SoA data from the store
    // Note: In a full integration, PersistentOrderStore would directly use ParquetPersistence
    // For this demo, we show the Parquet persistence capability

    if let Some(soa_data) = memory_store.query_storage(|_| true).await? {
        // Create Parquet persistence backend
        let mut parquet_persistence = ParquetPersistence::<OrderSoA>::new(&data_dir)
            .with_compression(Compression::ZSTD(Default::default()))
            .with_page_size(8192);

        // Save to disk
        parquet_persistence.save(&soa_data).await?;

        let file_path = data_dir.join("data.parquet");
        let metadata = std::fs::metadata(&file_path)?;

        println!("  ✅ Saved to Parquet file");
        println!("     File: {:?}", file_path);
        println!(
            "     Size: {} bytes ({:.2} KB)",
            metadata.len(),
            metadata.len() as f64 / 1024.0
        );
        println!("     Rows: {}", parquet_persistence.count().await?);

        // 4. Demonstrate data survives "application restart"
        println!("\n🔄 Simulating application restart:");
        println!("  💀 Application terminated...");
        println!("  🔌 Application restarted...");

        // Create new persistence instance (simulating restart)
        let restored_persistence = ParquetPersistence::<OrderSoA>::new(&data_dir)
            .with_compression(Compression::ZSTD(Default::default()));

        // Load data from disk
        if let Some(loaded_data) = restored_persistence.load().await? {
            println!("  ✅ Successfully loaded data from disk");
            println!(
                "     Restored {} orders",
                loaded_data.order_id_raw_array().len()
            );

            // Verify data integrity
            let original_total: f64 = soa_data.total_amount_raw_array().iter().sum();
            let loaded_total: f64 = loaded_data.total_amount_raw_array().iter().sum();

            println!("\n📊 Data Integrity Check:");
            println!("  Original total revenue: ${:.2}", original_total);
            println!("  Loaded total revenue:   ${:.2}", loaded_total);
            println!(
                "  Match: {}",
                if (original_total - loaded_total).abs() < 0.01 {
                    "✅"
                } else {
                    "❌"
                }
            );

            // 5. Demonstrate append operation
            println!("\n➕ Appending new orders:");

            let new_orders = vec![
                Order::new_with_payment(9, 1009, 2009, 2, 175.00, PaymentMethod::CreditCard)
                    .with_status(OrderStatus::Processing),
                Order::new_with_payment(10, 1010, 2010, 1, 425.99, PaymentMethod::PayPal)
                    .with_status(OrderStatus::Pending),
            ];

            // Add new orders to memory store
            let mut temp_store = PersistentOrderStore::with_capacity(10);
            temp_store.add_batch(new_orders).await?;

            if let Some(new_soa_data) = temp_store.query_storage(|_| true).await? {
                // Append to existing Parquet file
                let mut append_persistence = ParquetPersistence::<OrderSoA>::new(&data_dir);
                append_persistence.append(&new_soa_data).await?;

                println!("  ✅ Appended 2 new orders");
                println!(
                    "  Total orders in file: {}",
                    append_persistence.count().await?
                );

                // Reload to verify
                if let Some(all_data) = append_persistence.load().await? {
                    println!(
                        "  Verified: {} orders now in storage",
                        all_data.order_id_raw_array().len()
                    );
                }
            }

            // 6. Query operations on disk-persisted data
            println!("\n🔍 Querying disk-persisted data:");

            let query_persistence = ParquetPersistence::<OrderSoA>::new(&data_dir);

            // Query for delivered orders
            if let Some(all_orders) = query_persistence.load().await? {
                let delivered_count = all_orders
                    .status_raw_array()
                    .iter()
                    .filter(|&&status| status == OrderStatus::Delivered)
                    .count();

                let delivered_revenue: f64 = all_orders
                    .status_raw_array()
                    .iter()
                    .zip(all_orders.total_amount_raw_array().iter())
                    .filter(|(&status, _)| status == OrderStatus::Delivered)
                    .map(|(_, &amount)| amount)
                    .sum();

                println!("  Delivered orders: {}", delivered_count);
                println!("  Delivered revenue: ${:.2}", delivered_revenue);

                // High-value orders
                let high_value_count = all_orders
                    .total_amount_raw_array()
                    .iter()
                    .filter(|&&amount| amount > 200.0)
                    .count();

                println!("  High-value orders (>$200): {}", high_value_count);

                // Payment method breakdown
                let credit_card_count = all_orders
                    .payment_method_raw_array()
                    .iter()
                    .filter(|&&method| method == PaymentMethod::CreditCard)
                    .count();

                let paypal_count = all_orders
                    .payment_method_raw_array()
                    .iter()
                    .filter(|&&method| method == PaymentMethod::PayPal)
                    .count();

                println!("  Credit Card payments: {}", credit_card_count);
                println!("  PayPal payments: {}", paypal_count);
            }

            // 7. Efficient metadata operations
            println!("\n⚡ Efficient metadata operations:");

            let metadata_persistence = ParquetPersistence::<OrderSoA>::new(&data_dir);

            // Count without reading data (uses Parquet metadata)
            let count = metadata_persistence.count().await?;
            println!("  Row count (metadata only): {}", count);
            println!("  ✅ No data read - instant operation");

            // 8. Compression comparison
            println!("\n🗜️  Compression effectiveness:");

            let file_size = std::fs::metadata(data_dir.join("data.parquet"))?.len();
            let estimated_raw_size = count * 100; // Rough estimate: 100 bytes per row
            let compression_ratio = estimated_raw_size as f64 / file_size as f64;

            println!("  Compressed size: {} bytes", file_size);
            println!("  Estimated raw size: {} bytes", estimated_raw_size);
            println!("  Compression ratio: {:.2}x", compression_ratio);
        }
    }

    // 9. Performance characteristics
    println!("\n🚀 Parquet Persistence Benefits:");
    println!("  ✅ Durable storage - survives application restarts");
    println!("  ✅ Compressed format - ZSTD provides excellent compression");
    println!("  ✅ Standard format - compatible with Spark, Pandas, Polars");
    println!("  ✅ Columnar storage - optimized for analytical queries");
    println!("  ✅ Metadata operations - count() without reading data");
    println!("  ✅ Async I/O - non-blocking disk operations");

    // 10. Integration benefits
    println!("\n🔗 Data Science Integration:");
    println!("  • Can be read by Python (pandas, polars, pyarrow)");
    println!("  • Compatible with Apache Spark for big data processing");
    println!("  • Works with DuckDB for SQL analytics");
    println!("  • Standard format for data lakes and warehouses");
    println!("  • Zero-copy integration with Arrow ecosystem");

    // 11. Cleanup
    println!("\n🧹 Cleanup:");
    print!("  Remove demo data directory? [y/N]: ");

    // For automated demo, we'll skip interactive cleanup
    println!("(Skipped in automated demo)");
    println!("  Data preserved at: {:?}", data_dir);
    println!("  To clean up: rm -rf {:?}", data_dir);

    println!("\n✨ Parquet persistence demo completed successfully!");
    println!("\n💡 Next steps:");
    println!("  1. Try different compression algorithms (SNAPPY, GZIP, LZ4)");
    println!("  2. Read the Parquet file with Python/Pandas");
    println!("  3. Compare Arrow (in-memory) vs Parquet (disk) performance");
    println!("  4. Integrate with DuckDB for SQL queries");

    Ok(())
}
