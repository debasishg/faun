use crate::{OrderStatus, OrderStore, PaymentMethod};
use std::collections::HashMap;

/// Direct array access optimization - eliminates iterator overhead
/// and enables cache-line aware processing
pub fn direct_access_revenue_analysis(store: &OrderStore) -> HashMap<PaymentMethod, f64> {
    let soa = store.kernel();
    let statuses = soa.status_raw_array();
    let payments = soa.payment_method_raw_array();
    let amounts = soa.total_amount_raw_array();

    let mut results = HashMap::new();

    // Process 8 elements per iteration to optimize cache line usage
    // Each cache line is 64 bytes, so we can fit multiple small values
    for chunk_start in (0..statuses.len()).step_by(8) {
        let chunk_end = (chunk_start + 8).min(statuses.len());

        // Process a cache-line worth of data
        for i in chunk_start..chunk_end {
            if matches!(statuses[i], OrderStatus::Delivered) {
                *results.entry(payments[i]).or_insert(0.0) += amounts[i];
            }
        }
    }

    results
}

/// Direct access customer analysis - demonstrates columnar processing
pub fn direct_access_customer_analysis(store: &OrderStore) -> HashMap<u64, (u32, f64)> {
    let soa = store.kernel();
    let customers = soa.customer_id_raw_array();
    let statuses = soa.status_raw_array();
    let amounts = soa.total_amount_raw_array();

    let mut results: HashMap<u64, (u32, f64)> = HashMap::new();

    // Sequential access to three arrays - much better cache behavior
    // than scattered access through iterators
    for i in 0..customers.len() {
        let entry = results.entry(customers[i]).or_insert((0, 0.0));
        entry.0 += 1; // order count

        if matches!(statuses[i], OrderStatus::Delivered) {
            entry.1 += amounts[i]; // lifetime value
        }
    }

    results
}

/// Direct access product performance - shows multi-field aggregation
pub fn direct_access_product_performance(store: &OrderStore) -> HashMap<u64, (u32, f64, f64)> {
    let soa = store.kernel();
    let products = soa.product_id_raw_array();
    let quantities = soa.quantity_raw_array();
    let statuses = soa.status_raw_array();
    let amounts = soa.total_amount_raw_array();

    let mut results: HashMap<u64, (u32, f64, f64)> = HashMap::new();

    // Direct array access allows us to process all needed fields efficiently
    for i in 0..products.len() {
        let entry = results.entry(products[i]).or_insert((0, 0.0, 0.0));
        entry.0 += quantities[i]; // total quantity
        entry.1 += amounts[i]; // total revenue

        if matches!(statuses[i], OrderStatus::Delivered) {
            entry.2 += amounts[i]; // delivered revenue
        }
    }

    results
}

/// Bulk filtering using direct access - much faster than iterator chains
pub fn direct_access_bulk_filter(store: &OrderStore, min_amount: f64) -> Vec<u64> {
    let soa = store.kernel();
    let order_ids = soa.order_id_raw_array();
    let amounts = soa.total_amount_raw_array();
    let statuses = soa.status_raw_array();
    let payments = soa.payment_method_raw_array();

    let mut results = Vec::new();

    // Fast filtering with direct array access
    for i in 0..order_ids.len() {
        if amounts[i] > min_amount
            && matches!(statuses[i], OrderStatus::Delivered)
            && matches!(payments[i], PaymentMethod::CreditCard)
        {
            results.push(order_ids[i]);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Order;

    fn create_test_store() -> OrderStore {
        let mut store = OrderStore::new();

        // Add test data
        store.add(
            Order::new_with_payment(1, 100, 200, 2, 50.0, PaymentMethod::CreditCard)
                .with_status(OrderStatus::Delivered),
        );
        store.add(
            Order::new_with_payment(2, 100, 201, 1, 75.0, PaymentMethod::PayPal)
                .with_status(OrderStatus::Delivered),
        );

        store
    }

    #[test]
    fn test_direct_access_revenue() {
        let store = create_test_store();
        let results = direct_access_revenue_analysis(&store);

        assert!(results.contains_key(&PaymentMethod::CreditCard));
        assert!(results.contains_key(&PaymentMethod::PayPal));
    }
}
