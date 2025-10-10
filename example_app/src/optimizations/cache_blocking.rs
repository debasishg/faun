use crate::{OrderStatus, OrderStore, PaymentMethod};
use std::collections::HashMap;

// L1 cache is typically 32KB, L2 is 256KB
// We'll use conservative estimates for cache-blocking
const L1_CACHE_SIZE: usize = 32 * 1024;
const L2_CACHE_SIZE: usize = 256 * 1024;

// Estimate bytes per logical record for our three main fields
// OrderStatus (1 byte) + PaymentMethod (1 byte) + f64 (8 bytes) = ~10 bytes
// With some padding and cache line alignment, use 16 bytes per record
const BYTES_PER_RECORD: usize = 16;
const L1_CACHE_RECORDS: usize = L1_CACHE_SIZE / BYTES_PER_RECORD;
const L2_CACHE_RECORDS: usize = L2_CACHE_SIZE / BYTES_PER_RECORD;

/// Cache-blocking optimization - processes data in chunks that fit in L1 cache
/// This keeps the working set small and minimizes cache misses
pub fn cache_blocked_aggregation(store: &OrderStore) -> HashMap<PaymentMethod, f64> {
    let soa = store.kernel();
    let statuses = soa.status_raw_array();
    let payments = soa.payment_method_raw_array();
    let amounts = soa.total_amount_raw_array();

    let mut results = HashMap::new();
    let len = statuses.len();

    // Process in L1-cache-sized blocks
    for block_start in (0..len).step_by(L1_CACHE_RECORDS) {
        let block_end = (block_start + L1_CACHE_RECORDS).min(len);

        // Process this block completely before moving to the next
        // This keeps all three arrays for this block in L1 cache
        process_revenue_block(
            &statuses[block_start..block_end],
            &payments[block_start..block_end],
            &amounts[block_start..block_end],
            &mut results,
        );
    }

    results
}

/// Process a single cache block - all data should fit in L1 cache
fn process_revenue_block(
    statuses: &[OrderStatus],
    payments: &[PaymentMethod],
    amounts: &[f64],
    results: &mut HashMap<PaymentMethod, f64>,
) {
    // Inner loop has excellent cache locality
    for i in 0..statuses.len() {
        if matches!(statuses[i], OrderStatus::Delivered) {
            *results.entry(payments[i]).or_insert(0.0) += amounts[i];
        }
    }
}

/// Two-level cache-blocking for very large datasets
/// Uses L2 cache for outer blocks, L1 cache for inner blocks
pub fn hierarchical_cache_blocked_aggregation(store: &OrderStore) -> HashMap<PaymentMethod, f64> {
    let soa = store.kernel();
    let len = soa.len();
    let mut results = HashMap::new();

    // Outer loop: L2 cache blocks
    for l2_block_start in (0..len).step_by(L2_CACHE_RECORDS) {
        let l2_block_end = (l2_block_start + L2_CACHE_RECORDS).min(len);

        // Inner loop: L1 cache blocks within L2 block
        for l1_block_start in (l2_block_start..l2_block_end).step_by(L1_CACHE_RECORDS) {
            let l1_block_end = (l1_block_start + L1_CACHE_RECORDS).min(l2_block_end);

            let statuses = &soa.status_raw_array()[l1_block_start..l1_block_end];
            let payments = &soa.payment_method_raw_array()[l1_block_start..l1_block_end];
            let amounts = &soa.total_amount_raw_array()[l1_block_start..l1_block_end];

            process_revenue_block(statuses, payments, amounts, &mut results);
        }
    }

    results
}

/// Cache-blocked customer analysis
pub fn cache_blocked_customer_analysis(store: &OrderStore) -> HashMap<u64, f64> {
    let soa = store.kernel();
    let len = soa.len();
    let mut results = HashMap::new();

    // Process in cache-friendly blocks
    for block_start in (0..len).step_by(L1_CACHE_RECORDS) {
        let block_end = (block_start + L1_CACHE_RECORDS).min(len);

        let customers = &soa.customer_id_raw_array()[block_start..block_end];
        let statuses = &soa.status_raw_array()[block_start..block_end];
        let amounts = &soa.total_amount_raw_array()[block_start..block_end];

        // Process block with good cache locality
        for i in 0..customers.len() {
            if matches!(statuses[i], OrderStatus::Delivered) {
                *results.entry(customers[i]).or_insert(0.0) += amounts[i];
            }
        }
    }

    results
}

/// Prefetch-aware cache blocking - hints to CPU about future memory accesses
pub fn prefetch_aware_aggregation(store: &OrderStore) -> HashMap<PaymentMethod, f64> {
    let soa = store.kernel();
    let statuses = soa.status_raw_array();
    let payments = soa.payment_method_raw_array();
    let amounts = soa.total_amount_raw_array();
    let len = statuses.len();

    let mut results = HashMap::new();

    for block_start in (0..len).step_by(L1_CACHE_RECORDS) {
        let block_end = (block_start + L1_CACHE_RECORDS).min(len);

        // Prefetch next block while processing current block
        let next_block_start = block_start + L1_CACHE_RECORDS;
        if next_block_start < len {
            #[cfg(target_arch = "x86_64")]
            unsafe {
                use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};

                // Prefetch next block into L1 cache
                if next_block_start < statuses.len() {
                    _mm_prefetch(
                        statuses.as_ptr().add(next_block_start) as *const i8,
                        _MM_HINT_T0,
                    );
                }
                if next_block_start < payments.len() {
                    _mm_prefetch(
                        payments.as_ptr().add(next_block_start) as *const i8,
                        _MM_HINT_T0,
                    );
                }
                if next_block_start < amounts.len() {
                    _mm_prefetch(
                        amounts.as_ptr().add(next_block_start) as *const i8,
                        _MM_HINT_T0,
                    );
                }
            }
        }

        // Process current block
        for i in block_start..block_end {
            if matches!(statuses[i], OrderStatus::Delivered) {
                *results.entry(payments[i]).or_insert(0.0) += amounts[i];
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Order;

    fn create_large_test_store(size: usize) -> OrderStore {
        let mut store = OrderStore::new();

        for i in 0..size {
            let payment = match i % 3 {
                0 => PaymentMethod::CreditCard,
                1 => PaymentMethod::PayPal,
                _ => PaymentMethod::BankTransfer,
            };
            let status = if i % 2 == 0 {
                OrderStatus::Delivered
            } else {
                OrderStatus::Pending
            };

            store.add(
                Order::new_with_payment(
                    i as u64,
                    100 + (i % 50) as u64,
                    200,
                    1,
                    50.0 + (i % 100) as f64,
                    payment,
                )
                .with_status(status),
            );
        }

        store
    }

    #[test]
    fn test_cache_blocked_aggregation() {
        let store = create_large_test_store(10000);
        let results = cache_blocked_aggregation(&store);

        // Should have results for all payment methods
        assert_eq!(results.len(), 3);
        assert!(results.values().all(|&v| v > 0.0));
    }

    #[test]
    fn test_hierarchical_cache_blocking() {
        let store = create_large_test_store(50000);
        let results = hierarchical_cache_blocked_aggregation(&store);

        // Should produce same results as regular cache blocking
        let regular_results = cache_blocked_aggregation(&store);

        for (method, amount) in &results {
            assert!((amount - regular_results.get(method).unwrap_or(&0.0)).abs() < 0.01);
        }
    }
}
