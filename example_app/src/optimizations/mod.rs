pub mod cache_blocking;
pub mod direct_access;
pub mod memory_layout;

#[cfg(target_arch = "x86_64")]
pub mod simd;

pub use cache_blocking::*;
pub use direct_access::*;
pub use memory_layout::*;

#[cfg(target_arch = "x86_64")]
pub use simd::*;

#[cfg(not(target_arch = "x86_64"))]
pub use cache_blocking::cache_blocked_aggregation as simd_revenue_analysis;
