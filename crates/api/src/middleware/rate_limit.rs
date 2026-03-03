use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;


pub type SharedRateLimiter = Arc<RateLimiter<String, governor::state::keyed::DefaultKeyedStateStore<String>, governor::clock::DefaultClock>>;

pub fn create_keyed_rate_limiter(per_second: u32) -> SharedRateLimiter {
    let quota = Quota::per_second(NonZeroU32::new(per_second).expect("rate limit must be > 0"));
    Arc::new(RateLimiter::keyed(quota))
}

pub fn create_keyed_rate_limiter_per_minute(per_minute: u32) -> SharedRateLimiter {
    let quota = Quota::per_minute(NonZeroU32::new(per_minute).expect("rate limit must be > 0"));
    Arc::new(RateLimiter::keyed(quota))
}
