//! Given a set of samples, a query, and a number k, this program finds the k samples that are most similar to the query.

#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolValue;
use zkvdb_lib::{compute_best_sample, PublicValuesStruct};

pub fn main() {
    let samples = sp1_zkvm::io::read::<Vec<Vec<f32>>>();
    let query = sp1_zkvm::io::read::<Vec<f32>>();

    let (idx, _) = compute_best_sample(&samples, &query);

    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct { idx: idx as u32 });
    sp1_zkvm::io::commit_slice(&bytes);

    // TODO: return hash of the returned vector here as well
}
