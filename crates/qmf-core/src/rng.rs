use serde::{Deserialize, Serialize};

/// SplitMix64 — a fast, high-quality PRNG suitable for game logic.
///
/// Deterministic: same seed → same sequence, enabling reproducible games
/// and replay/sharing via seed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advance internal state and return next u64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Return a float in [0.0, 1.0).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1_u64 << 53) as f64
    }

    /// Return a usize in [0, bound) using rejection sampling to avoid modulo bias.
    pub fn next_usize(&mut self, bound: usize) -> usize {
        if bound <= 1 {
            return 0;
        }
        loop {
            let x = self.next_u64();
            let bucket = x as usize % bound;
            // Accept if the remainder doesn't fall in the incomplete last bucket.
            if x.wrapping_sub(bucket as u64) <= u64::MAX - (bound as u64 - 1) {
                return bucket;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn f64_range() {
        let mut rng = SplitMix64::new(123);
        for _ in 0..1000 {
            let v = rng.next_f64();
            assert!((0.0..1.0).contains(&v), "f64 out of range: {v}");
        }
    }

    #[test]
    fn next_usize_range() {
        let mut rng = SplitMix64::new(999);
        for bound in [1, 2, 5, 10, 100, 1000] {
            for _ in 0..200 {
                let v = rng.next_usize(bound);
                assert!(v < bound, "next_usize({bound}) produced {v}");
            }
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = SplitMix64::new(0);
        let mut b = SplitMix64::new(1);
        // Extremely unlikely to collide on all 10 outputs
        let same = (0..10).all(|_| a.next_u64() == b.next_u64());
        assert!(!same);
    }
}
