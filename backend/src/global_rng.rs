#[allow(unused_imports)]
use rand::rngs::{StdRng, ThreadRng};
#[allow(unused_imports)]
use rand::{RngCore, SeedableRng};
use std::sync::OnceLock;

#[allow(unused_imports)]
static SEED: OnceLock<u64> = OnceLock::new();

#[allow(dead_code)]
fn deterministic_seed() -> u64 {
    *SEED.get_or_init(|| {
        // Generate a new seed each run
        let mut rng = rand::rng();
        rng.next_u64()
    })
}

#[allow(dead_code)]
#[cfg(feature = "deterministic")]
pub fn global_rng() -> StdRng {
    StdRng::seed_from_u64(deterministic_seed())
}

#[cfg(not(feature = "deterministic"))]
pub fn global_rng() -> ThreadRng {
    rand::rng()
}