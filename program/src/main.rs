//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolValue;
use zkvdb_lib::{index_and_query, PublicValuesStruct};

pub fn main() {
    let samples = sp1_zkvm::io::read::<Vec<Vec<f32>>>();
    let query = sp1_zkvm::io::read::<Vec<f32>>();
    let k = sp1_zkvm::io::read::<u32>();

    let dest = index_and_query(samples, query, k);

    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct { k, dest });
    sp1_zkvm::io::commit_slice(&bytes);
}
