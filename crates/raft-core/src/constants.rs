use std::cell::LazyCell;

const JITTER_MIN: u64 = 1;
const JITTER_MAX: u64 = 25;

pub const ELECTION_TIMEOUT: LazyCell<std::time::Duration> = LazyCell::new(|| {
    let jitter = rand::random_range(JITTER_MIN..=JITTER_MAX);
    std::time::Duration::from_millis(500 + jitter)
});
