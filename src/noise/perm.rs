//! Permutation table.
//!
//! Implementation taken from crate [`noise-rs`](https://github.com/Razaekel/noise-rs).
//! Its `PermutationTable` type is not exposed for external.
use rand::{
    distributions::{Distribution, Standard},
    seq::SliceRandom,
    Rng, SeedableRng,
};
use rand_xorshift::XorShiftRng;
use reduce::Reduce;
use rust_wren::prelude::*;
use std::fmt;

const TABLE_SIZE: usize = 256;

#[wren_class]
#[derive(Copy, Clone)]
pub struct PermutationTable {
    values: [u8; TABLE_SIZE],
}

#[wren_methods]
impl PermutationTable {
    #[construct]
    fn new_() -> Self {
        todo!()
    }

    #[method(name = sample)]
    fn sample_1(&self) {}
}

impl PermutationTable {
    /// Deterministically generates a new permutation table based on a `u32` seed value.
    ///
    /// Internally this uses a `XorShiftRng`, but we don't really need to worry
    /// about cryptographic security when working with procedural noise.
    pub fn new(seed: u32) -> Self {
        let mut real = [0; 16];
        real[0] = 1;
        for i in 1..4 {
            real[i * 4] = seed as u8;
            real[(i * 4) + 1] = (seed >> 8) as u8;
            real[(i * 4) + 2] = (seed >> 16) as u8;
            real[(i * 4) + 3] = (seed >> 24) as u8;
        }
        let mut rng: XorShiftRng = SeedableRng::from_seed(real);
        rng.gen()
    }

    pub fn sample(&self, to_hash: &[isize]) -> usize {
        // let index: usize = Reduce::reduce(to_hash.iter().map(|&a| (a & 0xff) as usize), |a, b| {
        //     self.values[a] as usize ^ b
        // })
        let index: usize = to_hash
            .iter()
            .map(|&a| (a & 0xff) as usize)
            .reduce(|a, b| self.values[a] as usize ^ b)
            .unwrap();
        self.values[index] as usize
    }
}

impl Distribution<PermutationTable> for Standard {
    /// Generates a PermutationTable using a random seed.
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PermutationTable {
        let mut seq: Vec<u8> = (0..TABLE_SIZE).map(|x| x as u8).collect();
        seq.shuffle(rng);

        // It's unfortunate that this double-initializes the array, but Rust
        // doesn't currently provide a clean way to do this in one pass. Hopefully
        // it won't matter, as Seed creation will usually be a one-time event.
        let mut perm_table = PermutationTable {
            values: [0; TABLE_SIZE],
        };
        let seq_it = seq.iter();
        for (x, y) in perm_table.values.iter_mut().zip(seq_it) {
            *x = *y
        }
        perm_table
    }
}

impl fmt::Debug for PermutationTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PermutationTable {{ .. }}")
    }
}
