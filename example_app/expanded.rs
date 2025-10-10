#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use soa_macros::{SoA, SoAStore};
use std::collections::HashMap;
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
}
#[automatically_derived]
impl ::core::fmt::Debug for OrderStatus {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                OrderStatus::Pending => "Pending",
                OrderStatus::Processing => "Processing",
                OrderStatus::Shipped => "Shipped",
                OrderStatus::Delivered => "Delivered",
            },
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for OrderStatus {
    #[inline]
    fn clone(&self) -> OrderStatus {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for OrderStatus {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for OrderStatus {}
#[automatically_derived]
impl ::core::cmp::PartialEq for OrderStatus {
    #[inline]
    fn eq(&self, other: &OrderStatus) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for OrderStatus {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::hash::Hash for OrderStatus {
    #[inline]
    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        ::core::hash::Hash::hash(&__self_discr, state)
    }
}
pub enum PaymentMethod {
    CreditCard,
    PayPal,
    BankTransfer,
}
#[automatically_derived]
impl ::core::fmt::Debug for PaymentMethod {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                PaymentMethod::CreditCard => "CreditCard",
                PaymentMethod::PayPal => "PayPal",
                PaymentMethod::BankTransfer => "BankTransfer",
            },
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for PaymentMethod {
    #[inline]
    fn clone(&self) -> PaymentMethod {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for PaymentMethod {}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for PaymentMethod {}
#[automatically_derived]
impl ::core::cmp::PartialEq for PaymentMethod {
    #[inline]
    fn eq(&self, other: &PaymentMethod) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for PaymentMethod {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[automatically_derived]
impl ::core::hash::Hash for PaymentMethod {
    #[inline]
    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        ::core::hash::Hash::hash(&__self_discr, state)
    }
}
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
#[automatically_derived]
impl ::core::fmt::Debug for Order {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let names: &'static _ = &[
            "order_id",
            "customer_id",
            "product_id",
            "quantity",
            "unit_price",
            "total_amount",
            "status",
            "payment_method",
            "order_timestamp",
            "shipping_address_hash",
        ];
        let values: &[&dyn ::core::fmt::Debug] = &[
            &self.order_id,
            &self.customer_id,
            &self.product_id,
            &self.quantity,
            &self.unit_price,
            &self.total_amount,
            &self.status,
            &self.payment_method,
            &self.order_timestamp,
            &&self.shipping_address_hash,
        ];
        ::core::fmt::Formatter::debug_struct_fields_finish(f, "Order", names, values)
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Order {
    #[inline]
    fn clone(&self) -> Order {
        Order {
            order_id: ::core::clone::Clone::clone(&self.order_id),
            customer_id: ::core::clone::Clone::clone(&self.customer_id),
            product_id: ::core::clone::Clone::clone(&self.product_id),
            quantity: ::core::clone::Clone::clone(&self.quantity),
            unit_price: ::core::clone::Clone::clone(&self.unit_price),
            total_amount: ::core::clone::Clone::clone(&self.total_amount),
            status: ::core::clone::Clone::clone(&self.status),
            payment_method: ::core::clone::Clone::clone(&self.payment_method),
            order_timestamp: ::core::clone::Clone::clone(&self.order_timestamp),
            shipping_address_hash: ::core::clone::Clone::clone(
                &self.shipping_address_hash,
            ),
        }
    }
}
pub struct OrderSoA {
    order_id: ::std::vec::Vec<u64>,
    customer_id: ::std::vec::Vec<u64>,
    product_id: ::std::vec::Vec<u64>,
    quantity: ::std::vec::Vec<u32>,
    unit_price: ::std::vec::Vec<f64>,
    total_amount: ::std::vec::Vec<f64>,
    status: ::std::vec::Vec<OrderStatus>,
    payment_method: ::std::vec::Vec<PaymentMethod>,
    order_timestamp: ::std::vec::Vec<u64>,
    shipping_address_hash: ::std::vec::Vec<u64>,
}
#[automatically_derived]
impl ::core::clone::Clone for OrderSoA {
    #[inline]
    fn clone(&self) -> OrderSoA {
        OrderSoA {
            order_id: ::core::clone::Clone::clone(&self.order_id),
            customer_id: ::core::clone::Clone::clone(&self.customer_id),
            product_id: ::core::clone::Clone::clone(&self.product_id),
            quantity: ::core::clone::Clone::clone(&self.quantity),
            unit_price: ::core::clone::Clone::clone(&self.unit_price),
            total_amount: ::core::clone::Clone::clone(&self.total_amount),
            status: ::core::clone::Clone::clone(&self.status),
            payment_method: ::core::clone::Clone::clone(&self.payment_method),
            order_timestamp: ::core::clone::Clone::clone(&self.order_timestamp),
            shipping_address_hash: ::core::clone::Clone::clone(
                &self.shipping_address_hash,
            ),
        }
    }
}
impl OrderSoA {
    pub fn new() -> Self {
        Self {
            order_id: ::std::vec::Vec::new(),
            customer_id: ::std::vec::Vec::new(),
            product_id: ::std::vec::Vec::new(),
            quantity: ::std::vec::Vec::new(),
            unit_price: ::std::vec::Vec::new(),
            total_amount: ::std::vec::Vec::new(),
            status: ::std::vec::Vec::new(),
            payment_method: ::std::vec::Vec::new(),
            order_timestamp: ::std::vec::Vec::new(),
            shipping_address_hash: ::std::vec::Vec::new(),
        }
    }
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            order_id: ::std::vec::Vec::with_capacity(cap),
            customer_id: ::std::vec::Vec::with_capacity(cap),
            product_id: ::std::vec::Vec::with_capacity(cap),
            quantity: ::std::vec::Vec::with_capacity(cap),
            unit_price: ::std::vec::Vec::with_capacity(cap),
            total_amount: ::std::vec::Vec::with_capacity(cap),
            status: ::std::vec::Vec::with_capacity(cap),
            payment_method: ::std::vec::Vec::with_capacity(cap),
            order_timestamp: ::std::vec::Vec::with_capacity(cap),
            shipping_address_hash: ::std::vec::Vec::with_capacity(cap),
        }
    }
    pub fn len(&self) -> usize {
        if true {
            match (&self.order_id.len(), &self.order_id.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("SoA columns length mismatch"),
                            ),
                        );
                    }
                }
            };
        }
        if true {
            match (&self.order_id.len(), &self.customer_id.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("SoA columns length mismatch"),
                            ),
                        );
                    }
                }
            };
        }
        if true {
            match (&self.order_id.len(), &self.product_id.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("SoA columns length mismatch"),
                            ),
                        );
                    }
                }
            };
        }
        if true {
            match (&self.order_id.len(), &self.quantity.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("SoA columns length mismatch"),
                            ),
                        );
                    }
                }
            };
        }
        if true {
            match (&self.order_id.len(), &self.unit_price.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("SoA columns length mismatch"),
                            ),
                        );
                    }
                }
            };
        }
        if true {
            match (&self.order_id.len(), &self.total_amount.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("SoA columns length mismatch"),
                            ),
                        );
                    }
                }
            };
        }
        if true {
            match (&self.order_id.len(), &self.status.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("SoA columns length mismatch"),
                            ),
                        );
                    }
                }
            };
        }
        if true {
            match (&self.order_id.len(), &self.payment_method.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("SoA columns length mismatch"),
                            ),
                        );
                    }
                }
            };
        }
        if true {
            match (&self.order_id.len(), &self.order_timestamp.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("SoA columns length mismatch"),
                            ),
                        );
                    }
                }
            };
        }
        if true {
            match (&self.order_id.len(), &self.shipping_address_hash.len()) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        let kind = ::core::panicking::AssertKind::Eq;
                        ::core::panicking::assert_failed(
                            kind,
                            &*left_val,
                            &*right_val,
                            ::core::option::Option::Some(
                                format_args!("SoA columns length mismatch"),
                            ),
                        );
                    }
                }
            };
        }
        self.order_id.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn push(&mut self, v: Order) -> usize {
        self.order_id.push(v.order_id);
        self.customer_id.push(v.customer_id);
        self.product_id.push(v.product_id);
        self.quantity.push(v.quantity);
        self.unit_price.push(v.unit_price);
        self.total_amount.push(v.total_amount);
        self.status.push(v.status);
        self.payment_method.push(v.payment_method);
        self.order_timestamp.push(v.order_timestamp);
        self.shipping_address_hash.push(v.shipping_address_hash);
        self.len() - 1
    }
    pub fn view(&self, i: usize) -> OrderView<'_> {
        OrderView {
            order_id: &self.order_id[i],
            customer_id: &self.customer_id[i],
            product_id: &self.product_id[i],
            quantity: &self.quantity[i],
            unit_price: &self.unit_price[i],
            total_amount: &self.total_amount[i],
            status: &self.status[i],
            payment_method: &self.payment_method[i],
            order_timestamp: &self.order_timestamp[i],
            shipping_address_hash: &self.shipping_address_hash[i],
        }
    }
    pub fn view_mut(&mut self, i: usize) -> OrderMut<'_> {
        OrderMut {
            order_id: &mut self.order_id[i],
            customer_id: &mut self.customer_id[i],
            product_id: &mut self.product_id[i],
            quantity: &mut self.quantity[i],
            unit_price: &mut self.unit_price[i],
            total_amount: &mut self.total_amount[i],
            status: &mut self.status[i],
            payment_method: &mut self.payment_method[i],
            order_timestamp: &mut self.order_timestamp[i],
            shipping_address_hash: &mut self.shipping_address_hash[i],
        }
    }
    pub fn iter(&self) -> impl ::std::iter::Iterator<Item = OrderView<'_>> + '_ {
        (0..self.len()).map(|i| self.view(i))
    }
}
pub struct OrderView<'a> {
    pub order_id: &'a u64,
    pub customer_id: &'a u64,
    pub product_id: &'a u64,
    pub quantity: &'a u32,
    pub unit_price: &'a f64,
    pub total_amount: &'a f64,
    pub status: &'a OrderStatus,
    pub payment_method: &'a PaymentMethod,
    pub order_timestamp: &'a u64,
    pub shipping_address_hash: &'a u64,
}
pub struct OrderMut<'a> {
    pub order_id: &'a mut u64,
    pub customer_id: &'a mut u64,
    pub product_id: &'a mut u64,
    pub quantity: &'a mut u32,
    pub unit_price: &'a mut f64,
    pub total_amount: &'a mut f64,
    pub status: &'a mut OrderStatus,
    pub payment_method: &'a mut PaymentMethod,
    pub order_timestamp: &'a mut u64,
    pub shipping_address_hash: &'a mut u64,
}
impl soa_runtime::SoaModel for Order {
    type Soa = OrderSoA;
    type View<'a> = OrderView<'a> where Self: 'a;
    type ViewMut<'a> = OrderMut<'a> where Self: 'a;
    fn push_into(soa: &mut Self::Soa, v: Self) {
        soa.push(v);
    }
    fn view(soa: &Self::Soa, i: usize) -> Self::View<'_> {
        soa.view(i)
    }
    fn view_mut(soa: &mut Self::Soa, i: usize) -> Self::ViewMut<'_> {
        soa.view_mut(i)
    }
}
pub struct OrderStore {
    inner: ::std::sync::Arc<OrderSoA>,
}
impl ::std::clone::Clone for OrderStore {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}
impl ::std::default::Default for OrderStore {
    fn default() -> Self {
        Self {
            inner: ::std::sync::Arc::new(OrderSoA::new()),
        }
    }
}
impl OrderStore {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, v: Order) -> usize {
        let inner = ::std::sync::Arc::make_mut(&mut self.inner);
        inner.push(v)
    }
    pub fn kernel(&self) -> &OrderSoA {
        &self.inner
    }
    pub fn kernel_mut(&mut self) -> &mut OrderSoA {
        ::std::sync::Arc::make_mut(&mut self.inner)
    }
}
pub struct OrderShardedStore {
    shards: ::std::vec::Vec<soa_runtime::CachePadded<OrderSoA>>,
}
impl OrderShardedStore {
    pub const DEFAULT_SHARDS: usize = 16usize;
    pub fn with_shards(n: usize, cap_per: usize) -> Self {
        let mut shards = ::std::vec::Vec::with_capacity(n);
        for _ in 0..n {
            shards.push(soa_runtime::CachePadded(OrderSoA::with_capacity(cap_per)));
        }
        Self { shards }
    }
    #[inline]
    fn shard_idx_from_key<K: ::std::hash::Hash>(key: &K, n: usize) -> usize {
        use ::std::hash::{Hash, Hasher};
        let mut h = ::std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut h);
        (h.finish() as usize) % n
    }
    pub fn add(&mut self, v: Order) -> (usize, usize) {
        let n = self.shards.len();
        let si = {
            let keyref = &v.order_id;
            Self::shard_idx_from_key(keyref, n)
        };
        let row = self.shards[si].0.push(v);
        (si, row)
    }
    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }
    pub fn shard(&self, i: usize) -> &OrderSoA {
        &self.shards[i].0
    }
    pub fn shard_mut(&mut self, i: usize) -> &mut OrderSoA {
        &mut self.shards[i].0
    }
}
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
            total_amount: quantity as f64 * unit_price,
            status: OrderStatus::Pending,
            payment_method: PaymentMethod::CreditCard,
            order_timestamp: 1234567890,
            shipping_address_hash: 0xdeadbeef,
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
        Self {
            order_id,
            customer_id,
            product_id,
            quantity,
            unit_price,
            total_amount: quantity as f64 * unit_price,
            status: OrderStatus::Pending,
            payment_method,
            order_timestamp: 1234567890,
            shipping_address_hash: 0xdeadbeef,
        }
    }
    pub fn with_status(mut self, status: OrderStatus) -> Self {
        self.status = status;
        self
    }
}
impl OrderSoA {
    /// Traditional iterator-style approach (what users would naturally write)
    /// Uses the macro-generated iter() method but accesses multiple fields per iteration
    pub fn revenue_by_payment_method_iterator(&self) -> HashMap<PaymentMethod, f64> {
        let mut revenue_map = HashMap::new();
        for order_view in self.iter() {
            if match *order_view.status {
                OrderStatus::Delivered => true,
                _ => false,
            } {
                *revenue_map.entry(*order_view.payment_method).or_insert(0.0)
                    += *order_view.total_amount;
            }
        }
        revenue_map
    }
    /// Optimized direct field access approach
    /// Accesses the generated fields directly for better cache behavior
    pub fn revenue_by_payment_method_optimized(&self) -> HashMap<PaymentMethod, f64> {
        let mut revenue_map = HashMap::new();
        const CHUNK_SIZE: usize = 1024;
        let len = self.len();
        for chunk_start in (0..len).step_by(CHUNK_SIZE) {
            let chunk_end = (chunk_start + CHUNK_SIZE).min(len);
            for i in chunk_start..chunk_end {
                if match self.status[i] {
                    OrderStatus::Delivered => true,
                    _ => false,
                } {
                    *revenue_map.entry(self.payment_method[i]).or_insert(0.0)
                        += self.total_amount[i];
                }
            }
        }
        revenue_map
    }
    /// Memory-optimized layout that interleaves frequently accessed fields
    /// Demonstrates how to reorganize data for optimal cache usage
    pub fn revenue_by_payment_method_memory_optimized(
        &self,
    ) -> HashMap<PaymentMethod, f64> {
        struct CompactOrder {
            status: OrderStatus,
            payment: PaymentMethod,
            amount: f64,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for CompactOrder {
            #[inline]
            fn clone(&self) -> CompactOrder {
                let _: ::core::clone::AssertParamIsClone<OrderStatus>;
                let _: ::core::clone::AssertParamIsClone<PaymentMethod>;
                let _: ::core::clone::AssertParamIsClone<f64>;
                *self
            }
        }
        #[automatically_derived]
        impl ::core::marker::Copy for CompactOrder {}
        let mut compact_orders: Vec<CompactOrder> = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            compact_orders
                .push(CompactOrder {
                    status: self.status[i],
                    payment: self.payment_method[i],
                    amount: self.total_amount[i],
                });
        }
        let mut revenue_map = HashMap::new();
        for order in &compact_orders {
            if match order.status {
                OrderStatus::Delivered => true,
                _ => false,
            } {
                *revenue_map.entry(order.payment).or_insert(0.0) += order.amount;
            }
        }
        revenue_map
    }
}
pub struct OrderAoS {
    pub orders: Vec<Order>,
}
impl OrderAoS {
    pub fn new() -> Self {
        Self { orders: Vec::new() }
    }
    pub fn push(&mut self, order: Order) {
        self.orders.push(order);
    }
    pub fn len(&self) -> usize {
        self.orders.len()
    }
    pub fn revenue_by_payment_method(&self) -> HashMap<PaymentMethod, f64> {
        let mut results = HashMap::new();
        for order in &self.orders {
            if match order.status {
                OrderStatus::Delivered => true,
                _ => false,
            } {
                *results.entry(order.payment_method).or_insert(0.0)
                    += order.total_amount;
            }
        }
        results
    }
}
impl Default for OrderAoS {
    fn default() -> Self {
        Self::new()
    }
}
