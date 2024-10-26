//! The MIT License (MIT)
//!
//! Copyright (c) 2023 Succinct Labs
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in
//! all copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
//! THE SOFTWARE.
//!
//! A simple program that aggregates the proofs of multiple programs proven with the zkVM.
//!
//! https://github.com/succinctlabs/sp1/tree/main/examples/aggregation
//!
//! cargo prove build --elf-name riscv32im-succinct-aggregator-elf   

#![no_main]
sp1_zkvm::entrypoint!(main);

use sha2::Digest;
use sha2::Sha256;

pub fn words_to_bytes_le(words: &[u32; 8]) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    for i in 0..8 {
        let word_bytes = words[i].to_le_bytes();
        bytes[i * 4..(i + 1) * 4].copy_from_slice(&word_bytes);
    }
    bytes
}

/// Encode a list of vkeys and committed values into a single byte array. In the future this could
/// be a merkle tree or some other commitment scheme.
///
/// ( vkeys.len() || vkeys || committed_values[0].len as u32 || committed_values[0] || ... )
pub fn commit_proof_pairs(vkeys: &[[u32; 8]], committed_values: &[Vec<u8>]) -> Vec<u8> {
    assert_eq!(vkeys.len(), committed_values.len());
    let mut res = Vec::with_capacity(
        4 + vkeys.len() * 32
            + committed_values.len() * 4
            + committed_values
                .iter()
                .map(|vals| vals.len())
                .sum::<usize>(),
    );

    // Note we use big endian because abi.encodePacked in solidity does also
    res.extend_from_slice(&(vkeys.len() as u32).to_be_bytes());
    for vkey in vkeys.iter() {
        res.extend_from_slice(&words_to_bytes_le(vkey));
    }
    for vals in committed_values.iter() {
        res.extend_from_slice(&(vals.len() as u32).to_be_bytes());
        res.extend_from_slice(vals);
    }

    res
}

pub fn main() {
    // Read the verification keys.
    let vkeys = sp1_zkvm::io::read::<Vec<[u32; 8]>>();

    // Read the public values.
    let public_values = sp1_zkvm::io::read::<Vec<Vec<u8>>>();

    // Verify the proofs.
    assert_eq!(vkeys.len(), public_values.len());
    for i in 0..vkeys.len() {
        let vkey = &vkeys[i];
        let public_values = &public_values[i];
        let public_values_digest = Sha256::digest(public_values);
        sp1_zkvm::lib::verify::verify_sp1_proof(vkey, &public_values_digest.into());
    }

    // TODO: Do something interesting with the proofs here.
    //
    // For example, commit to the verified proofs in a merkle tree. For now, we'll just commit to
    // all the (vkey, input) pairs.
    let commitment = commit_proof_pairs(&vkeys, &public_values);
    sp1_zkvm::io::commit_slice(&commitment);
}
