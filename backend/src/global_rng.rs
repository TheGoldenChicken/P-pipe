#[allow(unused_imports)]
use rand::SeedableRng;
#[allow(unused_imports)]
use rand::rngs::{StdRng, ThreadRng};

#[allow(dead_code)]
#[cfg(feature = "deterministic")]
pub fn global_rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

#[cfg(not(feature = "deterministic"))]
pub fn global_rng() -> ThreadRng {
    rand::rng()
}