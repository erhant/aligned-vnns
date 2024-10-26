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

use alloy_sol_types::SolType;
use clap::Parser;
use sp1_sdk::{ProverClient, SP1Stdin};
use std::path::PathBuf;

use zkvdb_embedder::{Data, EmbeddedData};
use zkvdb_lib::PublicValuesStruct;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const PROGRAM_ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

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

    #[clap(short, long, default_value = "2")]
    top_k: usize,
}

fn main() {
    sp1_sdk::utils::setup_logger();

    let args = Args::parse();
    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

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

    // Read k from args
    let k: usize = args.top_k;
    assert!(
        k <= samples.len(),
        "k must be less than or equal to the number of samples"
    );

    // Pass inputs to zkVM
    let mut stdin = SP1Stdin::new();
    stdin.write(&samples);
    stdin.write(&query);
    stdin.write(&k);
    println!("Inputs prepared.");

    if args.execute {
        println!("Executing program.");
        // Execute the program
        let (output, report) = client.execute(PROGRAM_ELF, stdin).run().unwrap();
        println!("Program executed successfully.");

        // Read the output.
        let decoded = PublicValuesStruct::abi_decode(output.as_slice(), true).unwrap();
        let PublicValuesStruct { idx } = decoded;
        println!("Closest idx: {}", idx);

        let expected_dest = zkvdb_lib::similarity_search(samples, query, k);
        assert_eq!(dest, expected_dest);
        println!("Values are correct!");

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        println!("Proving program.");
        // setup the program for proving.
        let (pk, vk) = client.setup(PROGRAM_ELF);

        // generate the proof
        let proof = client
            .prove(&pk, stdin)
            .compressed()
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // verify the proof
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");

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
