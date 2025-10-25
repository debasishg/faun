# Cache Analysis: Why SoA Wins at Filtering but Loses at Aggregation

## Executive Summary

The benchmark results reveal a fascinating case study in cache behavior:
- **Filter operations**: SoA is ~30% faster (15.4µs vs 11.2µs for 10K records)
- **Aggregation operations**: AoS is ~4x faster (4.47µs vs 16.8µs for 10K records)

This performance inversion is entirely due to **memory access patterns** and **cache line utilization**.

## Memory Layout Analysis

### Traditional Array of Structs (AoS)
```rust
struct TraditionalRecord {  // 40 bytes total
    pub id: u64,           // 0-7
    pub value: f64,        // 8-15
    pub status: Status,    // 16-16 (1 byte + 3 padding)
    pub timestamp: u64,    // 20-27
    pub category: u32,     // 28-31
    pub metadata: u64,     // 32-39
}
```

**Memory layout in array:**
```
Cache Line 1 (64 bytes): [Record0][Record1.partial]
Cache Line 2 (64 bytes): [Record1.rest][Record2.partial]
...
```

### Structure of Arrays (SoA)
```rust
// Conceptually stored as separate arrays:
ids: [u64; n]          // All IDs together
values: [f64; n]       // All values together  
statuses: [Status; n]  // All statuses together
categories: [u32; n]   // All categories together
...
```

**Memory layout:**
```
Cache Line 1: [id0, id1, id2, id3, id4, id5, id6, id7]
Cache Line 2: [value0, value1, value2, value3, value4, value5, value6, value7]
Cache Line 3: [status0, status1, ..., status15]  // 16 statuses per cache line
Cache Line 4: [cat0, cat1, cat2, cat3, ..., cat15]  // 16 categories per cache line
```

## Cache Behavior Analysis

### Filter Operation: `record.status == Active` → `record.value`

**AoS Access Pattern:**
```rust
for record in data {
    if record.status == Status::Active {  // Touch bytes 16-19 of each record
        sum += record.value;              // Touch bytes 8-15 of same record
    }
}
```

**Memory accesses per record:**
- Load entire 40-byte record into cache line
- Only use 13 bytes (status=1 + value=8 + padding)
- **Cache efficiency: 13/40 = 32.5%**
- **Waste: 27 bytes per record**

**SoA Access Pattern:**
```rust
for i in 0..len {
    if statuses[i] == Status::Active {  // Sequential access to status array
        sum += values[i];               // Sequential access to value array  
    }
}
```

**Memory accesses:**
- Status array: 16 statuses per 64-byte cache line
- Value array: 8 values per 64-byte cache line
- **Cache efficiency: Nearly 100%** (using entire cache line)
- **Prefetching works perfectly** (sequential access)

### Aggregation Operation: `status + category + value`

**AoS Access Pattern:**
```rust
for record in data {
    if record.status == Status::Active {     // Bytes 16-19
        sums[record.category] += record.value;  // Bytes 28-31 + 8-15
    }
}
```

**Why AoS wins here:**
- All three fields (status, category, value) are in the **same cache line**
- Single cache line load gives us all needed data
- **Spatial locality**: Related data accessed together
- **Cache efficiency: 13/40 = 32.5%** but this is acceptable since we need the data

**SoA Access Pattern:**
```rust
for i in 0..len {
    if statuses[i] == Status::Active {       // Cache line from status array
        sums[categories[i]] += values[i];    // Cache line from category array + value array
    }
}
```

**Why SoA loses here:**
- **Three separate cache line accesses** for each iteration:
  1. `statuses[i]` → Load from status array cache line
  2. `categories[i]` → Load from category array cache line  
  3. `values[i]` → Load from value array cache line
- **Cache pressure**: Multiple arrays compete for cache space
- **Random access pattern**: Accessing same index across different arrays
- **Memory bandwidth**: 3x more cache line fetches required

## Detailed Cache Line Mathematics

### Cache Line Analysis for 10,000 Records

**AoS (40 bytes per record):**
- Records per cache line: 64/40 = 1.6 → 1 complete record per line
- Total cache lines needed: ~10,000 cache lines
- For aggregation: 1 cache line access per record = 10,000 accesses

**SoA:**
- Status array: 10,000 bytes → 157 cache lines (16 statuses/line)
- Category array: 40,000 bytes → 625 cache lines (16 categories/line)  
- Value array: 80,000 bytes → 1,250 cache lines (8 values/line)
- For aggregation: 3 cache line accesses per ~16 records = 1,875 total accesses

## CPU Cache Hierarchy Impact

Modern CPUs have:
- **L1 Cache**: 32KB, ~1 cycle latency
- **L2 Cache**: 256KB, ~10 cycle latency  
- **L3 Cache**: 8MB+, ~40 cycle latency
- **Main Memory**: ~300 cycle latency

### Filter Operation Cache Behavior:
- **AoS**: Poor spatial locality → frequent L3/memory accesses
- **SoA**: Excellent spatial locality → stays in L1/L2 cache

### Aggregation Operation Cache Behavior:
- **AoS**: Related fields in same cache line → fewer total memory accesses
- **SoA**: Multiple arrays → cache thrashing between different memory regions

## The Performance Paradox

```
Filter (2 fields):    SoA wins  → Sequential access beats spatial locality
Aggregation (3 fields): AoS wins  → Spatial locality beats sequential access
```

**Tipping Point**: When accessing 3+ fields from the same logical record, the cost of multiple cache line loads in SoA exceeds the benefit of sequential access.

## Optimization Strategies

### Why Simple "Chunking" Doesn't Help
Our benchmark shows that naive chunking actually makes performance worse because:
1. **Same number of iterator calls**: We're still calling the iterator for each record
2. **Added overhead**: Extra chunking logic without reducing memory accesses
3. **No direct array access**: The SoA framework doesn't expose raw arrays

### Real SoA Aggregation Optimizations:

#### 1. Direct Array Access
```rust
// Hypothetical direct access to underlying arrays
let statuses = store.status_array();
let categories = store.category_array(); 
let values = store.value_array();

for i in (0..len).step_by(8) {
    // Load 8 elements at once - fits in cache line
    let status_chunk = &statuses[i..i+8];
    let category_chunk = &categories[i..i+8];
    let value_chunk = &values[i..i+8];
    
    // Process cache line worth of data
    for j in 0..8 {
        if status_chunk[j] == Active {
            sums[category_chunk[j] as usize] += value_chunk[j];
        }
    }
}
```

#### 2. SIMD Vectorization
```rust
use std::arch::x86_64::*;

// Process 4 f64 values simultaneously using AVX2
unsafe {
    let value_vec = _mm256_load_pd(&values[i]);
    let status_mask = create_active_mask(&statuses[i..i+4]);
    let filtered_values = _mm256_and_pd(value_vec, status_mask);
    // ... SIMD aggregation logic
}
```

#### 3. Cache-Blocking Algorithm
```rust
const CACHE_BLOCK_SIZE: usize = 1024; // Fit in L1 cache

for block_start in (0..len).step_by(CACHE_BLOCK_SIZE) {
    let block_end = (block_start + CACHE_BLOCK_SIZE).min(len);
    
    // Process one cache-sized block at a time
    // All three arrays for this block stay in L1 cache
    for i in block_start..block_end {
        if statuses[i] == Active {
            sums[categories[i] as usize] += values[i];
        }
    }
}
```

#### 4. Memory Layout Optimization
```rust
// Interleave frequently accessed fields
struct OptimizedSoA {
    // Pack status + category together (both small)
    status_category: Vec<(Status, u32)>,  // 8 bytes per element
    values: Vec<f64>,                     // 8 bytes per element
    // Keep other fields separate
    ids: Vec<u64>,
    timestamps: Vec<u64>,
    metadata: Vec<u64>,
}
```

### Performance Expectations

With proper optimizations, SoA aggregation could potentially match or beat AoS by:
- **SIMD**: 2-4x speedup from vectorization
- **Cache blocking**: Reduced cache misses
- **Direct access**: Eliminated iterator overhead
- **Layout optimization**: Better spatial locality for related fields

### The Lesson

The benchmark demonstrates that **naive SoA isn't always better** - you need:
1. **Proper tooling**: Direct access to underlying arrays
2. **Algorithm awareness**: Cache-conscious processing patterns  
3. **Hardware utilization**: SIMD, prefetching, etc.
4. **Workload analysis**: Understanding your access patterns

**Bottom line**: The choice between AoS and SoA should be based on empirical measurement with your specific workload and proper optimization techniques.

## Conclusion

This benchmark perfectly illustrates that **there's no universal "best" data layout**. The optimal choice depends on:

1. **Field access patterns**: Sequential (favors SoA) vs Random (favors AoS)
2. **Number of fields accessed**: Few fields (SoA) vs Many fields (AoS)
3. **Temporal locality**: Do you access the same record's fields together?
4. **Cache size constraints**: Large datasets may favor SoA's predictable access patterns

**The key insight**: Cache performance is about **minimizing total memory traffic**, not just maximizing cache line utilization. Sometimes fetching "wasted" data in the same cache line is more efficient than making multiple precise fetches.