#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::{OrderStatus, OrderStore, PaymentMethod};
use std::collections::HashMap;

/// SIMD-optimized revenue analysis using AVX2 instructions
/// Processes 4 f64 values simultaneously for significant speedup
#[cfg(target_arch = "x86_64")]
pub fn simd_revenue_analysis(store: &OrderStore) -> HashMap<PaymentMethod, f64> {
    let soa = store.kernel();
    let statuses = soa.status_raw_array();
    let payments = soa.payment_method_raw_array();
    let amounts = soa.total_amount_raw_array();

    let mut results = HashMap::new();
    let len = statuses.len();

    // Process 4 elements at a time using SIMD
    let simd_len = len & !3; // Round down to multiple of 4

    unsafe {
        // SIMD processing for bulk of data
        for i in (0..simd_len).step_by(4) {
            // Load 4 amounts into SIMD register
            let amounts_vec = _mm256_loadu_pd(&amounts[i]);

            // Create mask for delivered orders
            let delivered_mask = create_delivered_mask(&statuses[i..i + 4]);

            // Apply mask to amounts (zero out non-delivered)
            let filtered_amounts = _mm256_and_pd(amounts_vec, delivered_mask);

            // Extract and accumulate results
            let mut extracted = [0.0; 4];
            _mm256_storeu_pd(extracted.as_mut_ptr(), filtered_amounts);

            for j in 0..4 {
                if extracted[j] != 0.0 {
                    *results.entry(payments[i + j]).or_insert(0.0) += extracted[j];
                }
            }
        }
    }

    // Handle remaining elements with scalar code
    for i in simd_len..len {
        if matches!(statuses[i], OrderStatus::Delivered) {
            *results.entry(payments[i]).or_insert(0.0) += amounts[i];
        }
    }

    results
}

/// Create SIMD mask for delivered orders
#[cfg(target_arch = "x86_64")]
unsafe fn create_delivered_mask(statuses: &[OrderStatus]) -> __m256d {
    let mut mask_values = [0.0; 4];

    for (i, &status) in statuses.iter().enumerate().take(4) {
        if matches!(status, OrderStatus::Delivered) {
            mask_values[i] = f64::from_bits(0xFFFFFFFFFFFFFFFF); // All bits set = true mask
        }
    }

    _mm256_loadu_pd(mask_values.as_ptr())
}

/// SIMD-optimized customer aggregation
#[cfg(target_arch = "x86_64")]
pub fn simd_customer_analysis(store: &OrderStore) -> HashMap<u64, f64> {
    let soa = store.kernel();
    let customers = soa.customer_id_raw_array();
    let statuses = soa.status_raw_array();
    let amounts = soa.total_amount_raw_array();

    let mut results = HashMap::new();
    let len = customers.len();
    let simd_len = len & !3;

    unsafe {
        for i in (0..simd_len).step_by(4) {
            let amounts_vec = _mm256_loadu_pd(&amounts[i]);
            let delivered_mask = create_delivered_mask(&statuses[i..i + 4]);
            let filtered_amounts = _mm256_and_pd(amounts_vec, delivered_mask);

            let mut extracted = [0.0; 4];
            _mm256_storeu_pd(extracted.as_mut_ptr(), filtered_amounts);

            for j in 0..4 {
                if extracted[j] != 0.0 {
                    *results.entry(customers[i + j]).or_insert(0.0) += extracted[j];
                }
            }
        }
    }

    // Scalar remainder
    for i in simd_len..len {
        if matches!(statuses[i], OrderStatus::Delivered) {
            *results.entry(customers[i]).or_insert(0.0) += amounts[i];
        }
    }

    results
}

/// Vectorized bulk filtering using SIMD comparisons
#[cfg(target_arch = "x86_64")]
pub fn simd_bulk_filter(store: &OrderStore, min_amount: f64) -> Vec<u64> {
    let soa = store.kernel();
    let order_ids = soa.order_id_raw_array();
    let amounts = soa.total_amount_raw_array();
    let statuses = soa.status_raw_array();

    let mut results = Vec::new();
    let len = order_ids.len();
    let simd_len = len & !3;

    unsafe {
        let min_vec = _mm256_set1_pd(min_amount);

        for i in (0..simd_len).step_by(4) {
            let amounts_vec = _mm256_loadu_pd(&amounts[i]);

            // SIMD comparison: amounts >= min_amount
            let cmp_mask = _mm256_cmp_pd(amounts_vec, min_vec, _CMP_GE_OQ);

            // Create delivered status mask
            let status_mask = create_delivered_mask(&statuses[i..i + 4]);

            // Combine masks
            let combined_mask = _mm256_and_pd(cmp_mask, status_mask);

            // Extract mask and collect matching order IDs
            let mask_int = _mm256_movemask_pd(combined_mask);

            for j in 0..4 {
                if (mask_int & (1 << j)) != 0 {
                    results.push(order_ids[i + j]);
                }
            }
        }
    }

    // Scalar remainder
    for i in simd_len..len {
        if amounts[i] >= min_amount && matches!(statuses[i], OrderStatus::Delivered) {
            results.push(order_ids[i]);
        }
    }

    results
}

/// Mixed scalar-vector optimization that uses SIMD for computation
/// but scalar logic for complex branching
#[cfg(target_arch = "x86_64")]
pub fn hybrid_simd_aggregation(store: &OrderStore) -> HashMap<PaymentMethod, f64> {
    let soa = store.kernel();
    let statuses = soa.status_raw_array();
    let payments = soa.payment_method_raw_array();
    let amounts = soa.total_amount_raw_array();

    let mut results = HashMap::new();
    let len = statuses.len();
    let simd_len = len & !7; // Process 8 elements at a time for better throughput

    unsafe {
        for i in (0..simd_len).step_by(8) {
            // Load two 256-bit vectors (4 f64 each)
            let amounts_vec1 = _mm256_loadu_pd(&amounts[i]);
            let amounts_vec2 = _mm256_loadu_pd(&amounts[i + 4]);

            // Create status masks
            let mask1 = create_delivered_mask(&statuses[i..i + 4]);
            let mask2 = create_delivered_mask(&statuses[i + 4..i + 8]);

            // Apply masks
            let filtered1 = _mm256_and_pd(amounts_vec1, mask1);
            let filtered2 = _mm256_and_pd(amounts_vec2, mask2);

            // Extract and process results
            let mut extracted1 = [0.0; 4];
            let mut extracted2 = [0.0; 4];
            _mm256_storeu_pd(extracted1.as_mut_ptr(), filtered1);
            _mm256_storeu_pd(extracted2.as_mut_ptr(), filtered2);

            // Accumulate results (scalar part)
            for j in 0..4 {
                if extracted1[j] != 0.0 {
                    *results.entry(payments[i + j]).or_insert(0.0) += extracted1[j];
                }
                if extracted2[j] != 0.0 {
                    *results.entry(payments[i + j + 4]).or_insert(0.0) += extracted2[j];
                }
            }
        }
    }

    // Scalar remainder
    for i in simd_len..len {
        if matches!(statuses[i], OrderStatus::Delivered) {
            *results.entry(payments[i]).or_insert(0.0) += amounts[i];
        }
    }

    results
}

/// Check if CPU supports required SIMD instructions
#[cfg(target_arch = "x86_64")]
pub fn cpu_supports_avx2() -> bool {
    #[cfg(target_feature = "avx2")]
    {
        true
    }
    #[cfg(not(target_feature = "avx2"))]
    {
        // Runtime detection would go here
        // For now, assume false if not compiled with AVX2
        false
    }
}

/// Fallback implementations for non-x86_64 platforms
#[cfg(not(target_arch = "x86_64"))]
pub fn simd_revenue_analysis(store: &OrderStore) -> HashMap<PaymentMethod, f64> {
    // Fall back to cache-blocked implementation
    crate::optimizations::cache_blocking::cache_blocked_aggregation(store)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn simd_customer_analysis(store: &OrderStore) -> HashMap<u64, f64> {
    crate::optimizations::cache_blocking::cache_blocked_customer_analysis(store)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn simd_bulk_filter(store: &OrderStore, min_amount: f64) -> Vec<u64> {
    crate::optimizations::direct_access::direct_access_bulk_filter(store, min_amount)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn hybrid_simd_aggregation(store: &OrderStore) -> HashMap<PaymentMethod, f64> {
    crate::optimizations::cache_blocking::cache_blocked_aggregation(store)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn cpu_supports_avx2() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Order, OrderAnalytics};

    fn create_large_test_store(size: usize) -> OrderStore {
        let mut analytics = OrderAnalytics::new();

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

            analytics.add_order(
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

        analytics.get_store().clone()
    }

    #[test]
    fn test_simd_revenue_analysis() {
        let store = create_large_test_store(1000);
        let results = simd_revenue_analysis(&store);

        // Should have results for all payment methods
        assert!(results.len() > 0);
        assert!(results.values().all(|&v| v > 0.0));
    }

    #[test]
    fn test_simd_vs_scalar() {
        let store = create_large_test_store(1000);

        let simd_results = simd_revenue_analysis(&store);
        let scalar_results =
            crate::optimizations::direct_access::direct_access_revenue_analysis(&store);

        // Results should be approximately equal
        for (method, simd_amount) in &simd_results {
            let scalar_amount = scalar_results.get(method).unwrap_or(&0.0);
            assert!(
                (simd_amount - scalar_amount).abs() < 0.01,
                "SIMD vs scalar mismatch for {:?}: {} vs {}",
                method,
                simd_amount,
                scalar_amount
            );
        }
    }
}
