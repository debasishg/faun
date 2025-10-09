#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use soa_macros::{SoA, SoAStore};
pub enum Status {
    Pending,
    Completed,
    Cancelled,
}
#[automatically_derived]
impl ::core::fmt::Debug for Status {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                Status::Pending => "Pending",
                Status::Completed => "Completed",
                Status::Cancelled => "Cancelled",
            },
        )
    }
}
#[automatically_derived]
impl ::core::marker::Copy for Status {}
#[automatically_derived]
impl ::core::clone::Clone for Status {
    #[inline]
    fn clone(&self) -> Status {
        *self
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Status {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Status {
    #[inline]
    fn eq(&self, other: &Status) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
#[automatically_derived]
impl ::core::cmp::Eq for Status {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {}
}
#[soa_store(key = "id", shards = 8)]
pub struct Order {
    pub id: u64,
    pub amount: f64,
    pub status: Status,
    pub timestamp: u64,
}
pub struct OrderSoA {
    id: ::std::vec::Vec<u64>,
    amount: ::std::vec::Vec<f64>,
    status: ::std::vec::Vec<Status>,
    timestamp: ::std::vec::Vec<u64>,
}
#[automatically_derived]
impl ::core::clone::Clone for OrderSoA {
    #[inline]
    fn clone(&self) -> OrderSoA {
        OrderSoA {
            id: ::core::clone::Clone::clone(&self.id),
            amount: ::core::clone::Clone::clone(&self.amount),
            status: ::core::clone::Clone::clone(&self.status),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
impl OrderSoA {
    pub fn new() -> Self {
        Self {
            id: ::std::vec::Vec::new(),
            amount: ::std::vec::Vec::new(),
            status: ::std::vec::Vec::new(),
            timestamp: ::std::vec::Vec::new(),
        }
    }
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            id: ::std::vec::Vec::with_capacity(cap),
            amount: ::std::vec::Vec::with_capacity(cap),
            status: ::std::vec::Vec::with_capacity(cap),
            timestamp: ::std::vec::Vec::with_capacity(cap),
        }
    }
    pub fn len(&self) -> usize {
        if true {
            match (&self.id.len(), &self.id.len()) {
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
            match (&self.id.len(), &self.amount.len()) {
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
            match (&self.id.len(), &self.status.len()) {
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
            match (&self.id.len(), &self.timestamp.len()) {
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
        self.id.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn push(&mut self, v: Order) -> usize {
        self.id.push(v.id);
        self.amount.push(v.amount);
        self.status.push(v.status);
        self.timestamp.push(v.timestamp);
        self.len() - 1
    }
    pub fn view(&self, i: usize) -> OrderView<'_> {
        OrderView {
            id: &self.id[i],
            amount: &self.amount[i],
            status: &self.status[i],
            timestamp: &self.timestamp[i],
        }
    }
    pub fn view_mut(&mut self, i: usize) -> OrderMut<'_> {
        OrderMut {
            id: &mut self.id[i],
            amount: &mut self.amount[i],
            status: &mut self.status[i],
            timestamp: &mut self.timestamp[i],
        }
    }
    pub fn iter(&self) -> impl ::std::iter::Iterator<Item = OrderView<'_>> + '_ {
        (0..self.len()).map(|i| self.view(i))
    }
}
pub struct OrderView<'a> {
    pub id: &'a u64,
    pub amount: &'a f64,
    pub status: &'a Status,
    pub timestamp: &'a u64,
}
pub struct OrderMut<'a> {
    pub id: &'a mut u64,
    pub amount: &'a mut f64,
    pub status: &'a mut Status,
    pub timestamp: &'a mut u64,
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
    pub const DEFAULT_SHARDS: usize = 8usize;
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
            let keyref = &v.id;
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
#[automatically_derived]
impl ::core::fmt::Debug for Order {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "Order",
            "id",
            &self.id,
            "amount",
            &self.amount,
            "status",
            &self.status,
            "timestamp",
            &&self.timestamp,
        )
    }
}
#[automatically_derived]
impl ::core::marker::Copy for Order {}
#[automatically_derived]
impl ::core::clone::Clone for Order {
    #[inline]
    fn clone(&self) -> Order {
        let _: ::core::clone::AssertParamIsClone<u64>;
        let _: ::core::clone::AssertParamIsClone<f64>;
        let _: ::core::clone::AssertParamIsClone<Status>;
        *self
    }
}
fn main() {
    let mut store = OrderStore::new();
    store
        .add(Order {
            id: 1,
            amount: 10.0,
            status: Status::Completed,
            timestamp: 1111,
        });
    store
        .add(Order {
            id: 2,
            amount: 20.0,
            status: Status::Pending,
            timestamp: 2222,
        });
    store
        .add(Order {
            id: 3,
            amount: 30.0,
            status: Status::Completed,
            timestamp: 3333,
        });
    let total_completed: f64 = store
        .kernel()
        .iter()
        .filter(|v| match v.status {
            Status::Completed => true,
            _ => false,
        })
        .map(|v| *v.amount)
        .sum();
    {
        ::std::io::_print(
            format_args!("store: total_completed = {0}\n", total_completed),
        );
    };
    let mut shards = OrderShardedStore::with_shards(
        OrderShardedStore::DEFAULT_SHARDS,
        4,
    );
    shards
        .add(Order {
            id: 11,
            amount: 7.0,
            status: Status::Pending,
            timestamp: 4444,
        });
    shards
        .add(Order {
            id: 12,
            amount: 13.0,
            status: Status::Completed,
            timestamp: 5555,
        });
    shards
        .add(Order {
            id: 13,
            amount: 9.0,
            status: Status::Completed,
            timestamp: 6666,
        });
    let mut sum = 0.0;
    for si in 0..shards.shard_count() {
        let s = shards.shard(si);
        sum
            += s
                .iter()
                .filter(|v| match v.status {
                    Status::Completed => true,
                    _ => false,
                })
                .map(|v| *v.amount)
                .sum::<f64>();
    }
    {
        ::std::io::_print(format_args!("sharded: total_completed = {0}\n", sum));
    };
}
