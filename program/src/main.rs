#![no_main]
sp1_zkvm::entrypoint!(main);

use sha2::{Digest, Sha256};
use vnns_lib::compute_best_sample;

pub fn main() {
    let samples = sp1_zkvm::io::read::<Vec<Vec<f32>>>();
    let query = sp1_zkvm::io::read::<Vec<f32>>();

    // compute similarity and return index
    let idx = compute_best_sample(&samples, &query);

    // commit to output
    let output_bytes = samples[idx as usize]
        .iter()
        .flat_map(|f| f.to_ne_bytes())
        .collect::<Vec<_>>();
    let output_commit = Sha256::digest(&output_bytes);

    // commit to query
    let query_bytes = query
        .into_iter()
        .flat_map(|f| f.to_ne_bytes())
        .collect::<Vec<_>>();
    let query_commit = Sha256::digest(&query_bytes);

    // commit to samples
    let samples_bytes = samples
        .into_iter()
        .flatten()
        .flat_map(|f| f.to_ne_bytes())
        .collect::<Vec<_>>();
    let samples_commit = Sha256::digest(&samples_bytes);

    sp1_zkvm::io::commit_slice(&(idx as u32).to_ne_bytes()); // 8 byte (u32)
    sp1_zkvm::io::commit_slice(&query_commit); // 32 byte
    sp1_zkvm::io::commit_slice(&samples_commit); // 32 byte
    sp1_zkvm::io::commit_slice(&output_commit); // 32 byte
}
