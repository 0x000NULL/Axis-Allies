//! Deterministic RNG for reproducible dice rolls.
//!
//! Uses ChaCha8 seeded from the game state's RNG seed.
//! Every dice roll consumes from the same stream, ensuring
//! deterministic replay from the same seed + action sequence.

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand::Rng;

/// A deterministic dice roller backed by ChaCha8.
pub struct DeterministicRng {
    rng: ChaCha8Rng,
    counter: u64,
}

impl DeterministicRng {
    /// Create a new RNG from a seed and a starting counter position.
    pub fn new(seed: u64, counter: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        // Fast-forward to the current counter position
        // Each "consumption" is one u32 roll
        for _ in 0..counter {
            let _: u32 = rng.gen();
        }
        DeterministicRng { rng, counter }
    }

    /// Roll a single d6 (returns 1-6).
    pub fn roll_d6(&mut self) -> u8 {
        self.counter += 1;
        (self.rng.gen_range(0..6) + 1) as u8
    }

    /// Roll multiple d6 dice.
    pub fn roll_multiple_d6(&mut self, count: usize) -> Vec<u8> {
        (0..count).map(|_| self.roll_d6()).collect()
    }

    /// Current counter position (for saving state).
    pub fn counter(&self) -> u64 {
        self.counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_rolls() {
        let mut rng1 = DeterministicRng::new(42, 0);
        let mut rng2 = DeterministicRng::new(42, 0);

        let rolls1: Vec<u8> = (0..10).map(|_| rng1.roll_d6()).collect();
        let rolls2: Vec<u8> = (0..10).map(|_| rng2.roll_d6()).collect();

        assert_eq!(rolls1, rolls2, "Same seed should produce same rolls");
    }

    #[test]
    fn test_rolls_in_range() {
        let mut rng = DeterministicRng::new(0, 0);
        for _ in 0..100 {
            let roll = rng.roll_d6();
            assert!((1..=6).contains(&roll), "Roll {} out of range", roll);
        }
    }

    #[test]
    fn test_counter_advances() {
        let mut rng = DeterministicRng::new(42, 0);
        assert_eq!(rng.counter(), 0);
        rng.roll_d6();
        assert_eq!(rng.counter(), 1);
        rng.roll_multiple_d6(5);
        assert_eq!(rng.counter(), 6);
    }
}
