# SoA Framework: Bridging Domain-Driven Design with Data-Oriented Performance

A Rust framework that combines the intuitive APIs of Domain-Driven Design (DDD) with the performance benefits of Data-Oriented Design and Structure of Arrays (SoA). Write domain-focused code while automatically benefiting from cache-friendly data layouts and optimized memory access patterns.

## 🎯 Philosophy

The SoA Framework addresses a fundamental tension in software development: **the trade-off between code clarity and performance**.

**Domain-Driven Design (DDD)** gives us:
- Intuitive, business-focused APIs
- Type safety and clear domain modeling  
- Code that domain experts can understand
- Natural object-oriented patterns

**Data-Oriented Design (DoD)** gives us:
- Cache-friendly memory layouts
- Vectorization opportunities
- Reduced memory bandwidth usage
- 2-10x performance improvements

**Our Solution**: Use procedural macros to automatically transform DDD entities into DoD implementations, giving you the best of both worlds without compromise.

```rust
// Write this (familiar DDD code)...
#[derive(SoA, SoAStore)]
struct Order { id: u64, amount: f64, status: Status }

```

## 🚀 Key Features

- **Domain-First API**: Write code using familiar domain entities and business logic
- **Automatic SoA Generation**: Macros transparently convert your domain structs to Structure of Arrays
- **Zero-Cost Abstraction**: No runtime overhead - the macro generates efficient native code
- **Thread-Safe Stores**: Built-in Arc-based stores with copy-on-write semantics
- **Sharded Storage**: Optional sharding for high-performance concurrent access
- **Cache-Friendly**: Columnar data layout improves CPU cache utilization by 2-10x

## 📚 Why Structure of Arrays?

### Traditional Array of Structs (AoS) - Poor Cache Performance
```rust
struct Order { id: u64, amount: f64, status: Status, timestamp: u64 }
let orders = Vec<Order>; // [Order1][Order2][Order3]...

// Memory layout: |id|amount|status|timestamp|id|amount|status|timestamp|...
// When filtering by status, CPU loads unnecessary data (id, amount, timestamp)
```

### Structure of Arrays (SoA) - Excellent Cache Performance
```rust
struct OrderSoA {
    id: Vec<u64>,        // [id1, id2, id3, ...]
    amount: Vec<f64>,    // [amt1, amt2, amt3, ...]
    status: Vec<Status>, // [status1, status2, status3, ...]
    timestamp: Vec<u64>, // [ts1, ts2, ts3, ...]
}

// When filtering by status, CPU only loads status data - 4x better cache utilization!
```

**The Problem**: SoA is fast but cumbersome to write and maintain.
**Our Solution**: Generate SoA automatically from familiar DDD structs.

## 🔧 Quick Start

Add to your `Cargo.toml`:
```toml
[dependencies]
soa_macros = { path = "soa_macros" }
soa_runtime = { path = "soa_runtime" }
```

### Basic Usage

```rust
use soa_macros::{SoA, SoAStore};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OrderStatus { Pending, Completed, Cancelled }

// 1. Define your domain entity with familiar DDD patterns
#[derive(SoA, SoAStore, Debug, Copy, Clone)]
#[soa_store(key = "id", shards = 8)]
pub struct Order {
    pub id: u64,
    pub amount: f64,
    pub status: OrderStatus,
    pub timestamp: u64,
}

fn main() {
    // 2. Use intuitive domain-focused APIs
    let mut store = OrderStore::new();
    
    // Add orders using familiar domain objects
    store.add(Order { 
        id: 1, 
        amount: 100.0, 
        status: OrderStatus::Completed, 
        timestamp: 1697731200 
    });
    // 3. Query with business logic - but get SoA performance!
    let completed_revenue: f64 = store.kernel()
        .iter()
        .filter(|order| order.status == &OrderStatus::Completed)
        .map(|order| *order.amount)
        .sum();
    
    println!("Total completed revenue: ${}", completed_revenue);
    
    // 4. Use sharded storage for high-performance concurrent access
    let mut sharded_store = OrderShardedStore::with_shards(8, 1000);
    sharded_store.add(Order { 
        id: 100, 
        amount: 500.0, 
        status: OrderStatus::Pending, 
        timestamp: 1697731400 
    });
}
```

## 🛠️ Macros

The framework provides two main procedural macros:

### `#[derive(SoA)]`
Generates Structure of Arrays implementation from your domain struct:

```rust
#[derive(SoA)]
struct Order {
    id: u64,
    amount: f64,
    status: OrderStatus,
}
```

**Generates:**
- `OrderSoA` - Structure of Arrays implementation
- Efficient iterators and accessors
- Raw array methods for high-performance algorithms
- View types for safe borrowing

### `#[derive(SoAStore)]`
Generates thread-safe store with domain-focused API:

```rust
#[derive(SoAStore)]
#[soa_store(key = "id", shards = 8)]
struct Order { /* fields */ }
```

**Generates:**
- `OrderStore` - Thread-safe Arc-based store
- `OrderShardedStore` - High-performance sharded storage
- Domain methods like `add()`, `find_by_id()`, `filter()`
- Parallel processing capabilities

**Attributes:**
- `key = "field_name"` - Designates the primary key field
- `shards = N` - Number of shards for parallel processing

## 📊 Performance Benefits

### Cache Efficiency
- **Filtering**: 2-4x faster by loading only relevant fields
- **Aggregation**: 2-8x faster with optimized memory access patterns  
- **SIMD**: Vectorization opportunities for arithmetic operations
- **Memory**: 20-40% less bandwidth usage for typical queries

### Typical Benchmark Results
```
Filtering 100K orders by status:
├─ AoS (traditional):     312 µs
├─ SoA (naive):          555 µs  ❌ Naive SoA can be slower!
└─ SoA (optimized):      185 µs  ✅ 1.7x faster than AoS

Counting delivered orders:
├─ AoS:                   53 µs
└─ SoA (optimized):       13 µs  ✅ 4x faster

Summing order amounts:
├─ AoS:                  725 µs  
└─ SoA (optimized):      596 µs  ✅ 1.2x faster
```

**Key Insight**: Naive SoA can be slower than AoS. Our framework includes optimization techniques that ensure SoA consistently outperforms AoS across all workload types.

## 🧪 How to Run Benchmarks

Run comprehensive Criterion benchmarks:

```bash
# Run all benchmarks
cargo bench -p example_app

# Run the interactive demo
cargo run -p example_app
```

The benchmarks include:
- **SoA Advantages**: Scenarios where SoA excels (filtering, counting, field access)
- **Aggregation Comparison**: How different SoA optimization techniques perform
- **Statistical Analysis**: Confidence intervals, outlier detection, HTML reports

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

*"Write code like a domain expert, get performance like a systems programmer."*
