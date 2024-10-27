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

use zkvdb_embedder::{Data, EmbeddedData};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const PROGRAM_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM aggregator.
pub const AGGREGATOR_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-aggregator-elf");

struct AggregationInput {
    pub proof: SP1ProofWithPublicValues,
    pub vk: SP1VerifyingKey,
}

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,

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
        std::fs::read(args.path.with_extension("index.json")).expect("Failed to read the file");
    let samples_data: Vec<EmbeddedData<Data>> =
        serde_json::from_slice(&samples_bytes).expect("Failed to parse JSON");
    let samples = samples_data
        .into_iter()
        .map(|data| data.embeddings)
        .collect::<Vec<Vec<f32>>>();

    // Read query from file
    let query_bytes =
        std::fs::read(args.path.with_extension("query.json")).expect("Failed to read the file");
    let query: Vec<f32> = serde_json::from_slice(&query_bytes).expect("Failed to parse JSON");

    match exec_type {
        ExecutionType::Execute => {
            // pass everything at once for execution
            let mut stdin = SP1Stdin::new();
            stdin.write(&samples);
            stdin.write(&query);

            println!("Executing program.");
            // Execute the program

            let (output, report) = client.execute(PROGRAM_ELF, stdin).run().unwrap();
            println!("Program executed successfully.");

            // Read the output.
            let idx = u32::from_ne_bytes(
                output.as_slice()[0..4]
                    .try_into()
                    .expect("Failed to read u32 from output"),
            );
            println!("Closest idx: {}", idx);

            // Read the commitments
            let query_commitment = &output.as_slice()[4..36];
            println!("Query Commitment: {:?}", query_commitment.len());
            let samples_commitment = &output.as_slice()[36..68];
            println!("Samples Commitment: {:?}", samples_commitment.len());

            let expected_idx = zkvdb_lib::compute_best_sample(&samples, &query);
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
            for chunk in samples.chunks(CHUNK_SIZE) {
                println!("Generating proof for chunk.");
                let mut stdin = SP1Stdin::new();
                stdin.write(&chunk);
                stdin.write(&query);

                let proof = client
                    .prove(&pk, stdin)
                    .compressed()
                    .run()
                    .expect("failed to generate proof");
                proofs.push(proof);
            }

            // aggregate all proofs
            println!("Aggregating all {} proofs.", proofs.len());
            let mut stdin = SP1Stdin::new();
            let inputs: Vec<AggregationInput> = proofs
                .into_iter()
                .map(|proof| AggregationInput {
                    proof,
                    vk: vk.clone(),
                })
                .collect();

            // Write the verification keys.
            let vkeys = inputs
                .iter()
                .map(|input| input.vk.hash_u32())
                .collect::<Vec<_>>();
            stdin.write::<Vec<[u32; 8]>>(&vkeys);

            // Write the public values.
            let public_values = inputs
                .iter()
                .map(|input| input.proof.public_values.to_vec())
                .collect::<Vec<_>>();
            stdin.write::<Vec<Vec<u8>>>(&public_values);

            // Write the proofs.
            //
            // Note: this data will not actually be read by the aggregation program, instead it will be
            // witnessed by the prover during the recursive aggregation process inside SP1 itself.
            for input in inputs {
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
            std::fs::write(args.path.with_extension("sp1.proof"), proof_data)
                .expect("failed to save SP1 Proof file");

            // save public input
            println!("Saving public inputs.");
            std::fs::write(args.path.with_extension("sp1.pub"), proof.public_values)
                .expect("failed to save SP1 public input");
        }
    }
}
