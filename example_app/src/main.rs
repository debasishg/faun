use soa_macros::{SoA, SoAStore};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Status {
    Pending,
    Completed,
    Cancelled,
}

#[derive(SoA, SoAStore, Debug, Copy, Clone)]
#[soa_store(key = "id", shards = 8)]
pub struct Order {
    pub id: u64,
    pub amount: f64,
    pub status: Status,
    pub timestamp: u64,
}

fn main() {
    // Generated Store (Arc + COW)
    let mut store = OrderStore::new();
    store.add(Order {
        id: 1,
        amount: 10.0,
        status: Status::Completed,
        timestamp: 1111,
    });
    store.add(Order {
        id: 2,
        amount: 20.0,
        status: Status::Pending,
        timestamp: 2222,
    });
    store.add(Order {
        id: 3,
        amount: 30.0,
        status: Status::Completed,
        timestamp: 3333,
    });

    let total_completed: f64 = store
        .kernel()
        .iter()
        .filter(|v| matches!(v.status, Status::Completed))
        .map(|v| *v.amount)
        .sum();
    println!("store: total_completed = {}", total_completed);

    // Sharded store
    let mut shards = OrderShardedStore::with_shards(OrderShardedStore::DEFAULT_SHARDS, 4);
    shards.add(Order {
        id: 11,
        amount: 7.0,
        status: Status::Pending,
        timestamp: 4444,
    });
    shards.add(Order {
        id: 12,
        amount: 13.0,
        status: Status::Completed,
        timestamp: 5555,
    });
    shards.add(Order {
        id: 13,
        amount: 9.0,
        status: Status::Completed,
        timestamp: 6666,
    });

    let mut sum = 0.0;
    for si in 0..shards.shard_count() {
        let s = shards.shard(si);
        sum += s
            .iter()
            .filter(|v| matches!(v.status, Status::Completed))
            .map(|v| *v.amount)
            .sum::<f64>();
    }
    println!("sharded: total_completed = {}", sum);
}
