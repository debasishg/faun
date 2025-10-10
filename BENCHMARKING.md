# Performance Benchmarking

This project now includes proper benchmarking capabilities using the `criterion` crate.

## Running Benchmarks

To run the performance benchmarks:

```bash
cargo bench -p example_app
```

This will run comprehensive benchmarks comparing:
- Array of Structs (AoS) approach vs Structure of Arrays (SoA) approach
- Filter operations (filtering by status and summing values)
- Aggregation operations (grouping by category)
- Different dataset sizes (10K, 100K, 1M records)

## Running the Interactive Demo

To see an educational demonstration with explanations (not statistical benchmarks):

```bash
# Interactive demonstration with optimization showcase
cargo run --package example_app
```

## Benchmark Results

The criterion benchmarks will show:
- Precise timing measurements with statistical analysis
- Performance comparisons between AoS and SoA approaches
- Confidence intervals and outlier detection
- HTML reports (if gnuplot is available)

## Understanding the Results

From the benchmark results, you can see:

### Filter Operations
- **SoA advantage**: For filter operations, SoA typically shows 20-30% better performance
- **Cache efficiency**: SoA accesses only the required fields (status, value) leading to better cache utilization
- **Sequential access**: Perfect for CPU prefetching and cache line utilization

### Aggregation Operations
- **AoS advantage**: Traditional AoS can be 3-4x faster for aggregation workloads
- **Spatial locality**: All required fields (status, category, value) are in the same cache line
- **Cache pressure**: SoA requires multiple cache line accesses per record

> **📖 For detailed cache analysis**: See [CACHE_ANALYSIS.md](CACHE_ANALYSIS.md) for an in-depth explanation of why these performance differences occur at the CPU cache level.

### Key Insights
- Performance differences become more pronounced with larger datasets
- SoA excels when accessing a subset of fields frequently
- The framework provides SoA performance benefits while maintaining clean API design

## Benchmark Output Location

Criterion saves detailed reports to `target/criterion/` with:
- HTML reports for visualization
- Statistical analysis of performance data
- Historical performance comparisons

## Key Insights

### 🔍 **Performance Analysis:**

1. **Filter Operations (SoA wins ~30%)**:
   - **Sequential access** to individual arrays maximizes cache line utilization
   - **Cache efficiency**: SoA ~100% vs AoS 32.5% 
   - **Perfect prefetching**: CPU can predict and load next cache lines
   - **Minimal memory bandwidth**: Only load needed fields

2. **Aggregation Operations (AoS wins ~4x)**:
   - **Spatial locality**: All 3 fields (status, category, value) in same cache line
   - **Memory accesses**: AoS needs 1 cache line per record vs SoA needs 3
   - **Cache pressure**: SoA creates competition between multiple arrays
   - **Memory bandwidth**: SoA requires 3x more cache line fetches

### 🎯 **The Core Lesson:**

This perfectly demonstrates that **cache behavior trumps theoretical advantages**. SoA's sequential access wins when you need few fields, but AoS's spatial locality wins when you need multiple fields from the same logical record.

The performance inversion occurs precisely because:
- **2 fields**: Cost of cache line "waste" < Cost of multiple cache line fetches  
- **3+ fields**: Cost of cache line "waste" > Cost of multiple cache line fetches

This is a textbook example of why empirical benchmarking is crucial in systems performance!