//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use sp1_sdk::{
    HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use std::path::PathBuf;
use vnns_embedder::{Data, EmbeddedData};

pub const PROGRAM_ELF: &[u8] = include_bytes!("../../elf/riscv32im-succinct-vnns-elf");
pub const AGGREGATOR_ELF: &[u8] = include_bytes!("../../elf/riscv32im-succinct-aggregator-elf");

struct AggregationInput {
    pub proof: SP1ProofWithPublicValues,
    pub vk: SP1VerifyingKey,
}

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Simulate the execution of the program, without a proof.
    #[clap(long)]
    execute: bool,

    /// Generate batches of proofs.
    #[clap(long)]
    prove: bool,

    /// Aggregate the proofs to create only one final proof.
    #[clap(long)]
    aggregate: bool,

    /// Path to the data file.
    #[clap(short, long, default_value = "../data/foods-smol.json")]
    path: PathBuf,
}

enum ExecutionType {
    Execute,
    Prove,
}

fn main() {
    sp1_sdk::utils::setup_logger();
    let args = Args::parse();

    ////////// Parse execution type.
    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }
    let exec_type = if args.execute {
        ExecutionType::Execute
    } else {
        ExecutionType::Prove
    };

    ///////// Setup the prover client.
    let client = ProverClient::new();

    ///////// Setup the inputs.
    // Read samples from file
    let samples_bytes =
        std::fs::read(args.path.with_extension("index.json")).expect("failed to read the file");
    let samples_data: Vec<EmbeddedData<Data>> =
        serde_json::from_slice(&samples_bytes).expect("failed to parse JSON");
    let samples = samples_data
        .into_iter()
        .map(|data| data.embeddings)
        .collect::<Vec<Vec<f32>>>();

    // Read query from file
    let query_bytes =
        std::fs::read(args.path.with_extension("query.json")).expect("failed to read the file");
    let query: Vec<f32> = serde_json::from_slice(&query_bytes).expect("failed to parse JSON");

    match exec_type {
        ExecutionType::Execute => {
            // pass everything at once for execution
            let mut stdin = SP1Stdin::new();
            stdin.write(&samples);
            stdin.write(&query);

            // Execute the program
            println!("Executing program.");
            let (output, report) = client.execute(PROGRAM_ELF, stdin).run().unwrap();
            println!("Program executed successfully.");

            // Read the output.
            let idx = u32::from_ne_bytes(
                output.as_slice()[0..4]
                    .try_into()
                    .expect("failed to read u32 from output"),
            );

            // Read the commitments
            let query_commitment = &output.as_slice()[4..36];
            println!("Query Commitment: {}", hex::encode(query_commitment));
            let samples_commitment = &output.as_slice()[36..68];
            println!("Samples Commitment: {}", hex::encode(samples_commitment));
            let output_commitment = &output.as_slice()[68..100];
            println!("Output Commitment: {}", hex::encode(output_commitment));

            let expected_idx = vnns_lib::compute_best_sample(&samples, &query);
            assert_eq!(idx, expected_idx as u32);
            println!("Values are correct!");

            // Record the number of cycles executed.
            println!("Number of cycles: {}", report.total_instruction_count());
        }
        ExecutionType::Prove => {
            const CHUNK_SIZE: usize = 3;

            // setup the program for proving.
            let (pk, vk) = client.setup(PROGRAM_ELF);
            let (agg_pk, agg_vk) = client.setup(AGGREGATOR_ELF);

            // generate similarity proofs
            println!("Proving all chunks (chunk size {})", CHUNK_SIZE);
            let mut proofs = Vec::new();
            let mut current_samples = samples;
            while current_samples.len() > CHUNK_SIZE {
                // we will collect the best samples for this iteration here
                let mut best_samples = Vec::new();

                // process each chunk within the current samples
                for (chunk_idx, chunk) in current_samples.chunks(CHUNK_SIZE).enumerate() {
                    println!("Generating proof for chunk {}.", chunk_idx);
                    let mut stdin = SP1Stdin::new();
                    stdin.write(&chunk);
                    stdin.write(&query);

                    // create proof
                    let proof = client
                        .prove(&pk, stdin)
                        .compressed()
                        .run()
                        .expect("failed to generate proof");

                    // find idx from the public output and choose the best sample
                    let idx = u32::from_ne_bytes(
                        proof.public_values.as_slice()[0..4]
                            .try_into()
                            .expect("failed to read u32 from output"),
                    );
                    println!("Closest index: {}", idx);
                    let query_commitment = &proof.public_values.as_slice()[4..36];
                    println!("Query Commitment: {}", hex::encode(query_commitment));
                    let samples_commitment = &proof.public_values.as_slice()[36..68];
                    println!("Samples Commitment: {}", hex::encode(samples_commitment));
                    let output_commitment = &proof.public_values.as_slice()[68..100];
                    println!("Output Commitment: {}", hex::encode(output_commitment));

                    best_samples.push(idx);

                    // store proof for aggregation
                    proofs.push(proof);
                }

                // update samples with the results of each chunk
                current_samples = best_samples
                    .iter()
                    .map(|&idx| current_samples[idx as usize].clone())
                    .collect::<Vec<_>>();
            }

            // all sub-chunks are processed, do one more final proof
            {
                println!("Generating proof for final samples.");
                let mut stdin = SP1Stdin::new();
                stdin.write(&current_samples);
                stdin.write(&query);
                let proof = client
                    .prove(&pk, stdin)
                    .compressed()
                    .run()
                    .expect("failed to generate proof");

                let query_commitment = &proof.public_values.as_slice()[4..36];
                println!("Query Commitment: {}", hex::encode(query_commitment));
                let samples_commitment = &proof.public_values.as_slice()[36..68];
                println!("Samples Commitment: {}", hex::encode(samples_commitment));
                let output_commitment = &proof.public_values.as_slice()[68..100];
                println!("Output Commitment: {}", hex::encode(output_commitment));

                // verify the proof to be sure
                client.verify(&proof, &vk).expect("failed to verify proof");

                proofs.push(proof);
            }

            // save all proofs & publics to file
            for (i, proof) in proofs.iter().enumerate() {
                let proof_data = bincode::serialize(proof).expect("failed to serialize proof");
                std::fs::write(args.path.with_extension(format!("{}.proof", i)), proof_data)
                    .expect("failed to save SP1 proof");

                println!("Saving public inputs.");
                std::fs::write(
                    args.path.with_extension(format!("{}.pub", i)),
                    proof.public_values.clone(),
                )
                .expect("failed to save SP1 public input");
            }

            // if enabled, aggregate into one final proof
            if args.aggregate {
                println!("Aggregating all {} proofs.", proofs.len());
                let mut stdin = SP1Stdin::new();
                let agg_inputs: Vec<AggregationInput> = proofs
                    .into_iter()
                    .map(|proof| AggregationInput {
                        proof,
                        vk: vk.clone(),
                    })
                    .collect();

                // write the verification keys to aggregator
                let vkeys_bytes = agg_inputs
                    .iter()
                    .map(|input| input.vk.hash_u32())
                    .collect::<Vec<_>>();
                stdin.write::<Vec<[u32; 8]>>(&vkeys_bytes);

                // write the public values to aggregator
                let public_values_bytes = agg_inputs
                    .iter()
                    .map(|input| input.proof.public_values.to_vec())
                    .collect::<Vec<_>>();
                stdin.write::<Vec<Vec<u8>>>(&public_values_bytes);

                // write the proofs
                //
                // Note: this data will not actually be read by the aggregation program, instead it will be
                // witnessed by the prover during the recursive aggregation process inside SP1 itself.
                for input in agg_inputs {
                    let SP1Proof::Compressed(proof) = input.proof.proof else {
                        panic!("expected compressed proof");
                    };
                    stdin.write_proof(proof, input.vk.vk);
                }

                println!("Proving the aggregated proof.");
                let proof = client
                    .prove(&agg_pk, stdin)
                    .compressed()
                    .run()
                    .expect("failed to generate aggregation proof");
                println!("Successfully generated aggregation proof!");

                // verify the proof
                client
                    .verify(&proof, &agg_vk)
                    .expect("failed to verify proof");
                println!("Successfully verified aggregation proof!");

                // create & save proof
                println!("Saving proof.");
                let proof_data = bincode::serialize(&proof).expect("failed to serialize proof");
                std::fs::write(args.path.with_extension("agg.proof"), proof_data)
                    .expect("failed to save SP1 Proof file");

                // save public input
                println!("Saving public inputs.");
                std::fs::write(args.path.with_extension("agg.pub"), proof.public_values)
                    .expect("failed to save SP1 public input");
            }
        }
    }
}
