pub mod circom;
pub mod witness_calculator;
pub mod memory;

use fnv::FnvHasher;
use std::hash::Hasher;
use ark_bn254::Fr;
use num_bigint::{BigInt, BigUint};

pub(crate) fn fnv(inp: &str) -> (u32, u32) {
    let mut hasher = FnvHasher::default();
    hasher.write(inp.as_bytes());
    let h = hasher.finish();

    ((h >> 32) as u32, h as u32)
}

pub fn conv_fp_to_bigint(){

}

