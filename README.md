# Verifiable Nearest Neighbor Search

A **verifiable nearest-neighbor search** program, powered by [SP1](https://github.com/succinctlabs/sp1).

This project was built for [Aligned Builders Hackathon](https://devfolio.co/alignedhackathon/):

- [Project Link](https://devfolio.co/projects/verifiable-similarity-27b9)
- [Presentation](./vnns-presentation.pdf)

It has [won the 3rd place](https://x.com/alignedlayer/status/1862163883567559092)!

The requested information as per the [judging criteria](https://mirror.xyz/0x7794D1c55568270A81D8Bf39e1bcE96BEaC10901/JnG4agqhW0oiskZJgcFdi9SLKvqkTBrbXkuk1nT6lxk) has been given within the project link above.

## Setup

### Installation

You need the following:

- [Rust](https://rustup.rs/) for everything
- [SP1](https://docs.succinct.xyz/getting-started/install.html) for zkVM
- [Ollama](https://ollama.com/) for embeddings (optional)
- [Aligned SDK](https://docs.alignedlayer.com/introduction/1_try_aligned#quickstart) for proof verification

### Keystore

You can create a new wallet & store it directly in a random file with:

```sh
PRIV_KEY=$(cast wallet new | grep "Private key" | cut -d ' ' -f 3)

# will prompt for a password, can be left empty
cast wallet import --private-key $PRIV_KEY -k ./secrets/ $RANDOM.json
```

It will prompt for a password, and then output the keystore under [`secrets`](./secrets/) folder.

You can view the respective address of a keystore file with:

```sh
# will prompt for the password
cast wallet address --keystore ./secrets/wallet.json
```

### Aligned SDK

First, deposit some funds to Aligned layer (skip this if you already have done so):

```sh
./aligned deposit ./path/to/keystore.json
```

This will print a transaction hash, which can be viewed at `https://holesky.etherscan.io/tx/<hash-here>`.

You can check your balance with:

```sh
./aligned balance ./path/to/keystore.json
```

### Embeddings

> [!WARNING]
>
> You need to have Ollama running on `localhost:11434` (default) to run the commands in this section.

The repository comes with existing embeddings under the [`data`](./data/) folder, within the files with `.index.json` extension. For this project, each data has the following type:

```ts
{
  name: string;
  description: string;
}
```

You can create your own embeddings as follows:

```sh
cargo run --bin vnns-embedder index -p ./path/to/data.json
# will output ./path/to/data.index.json
```

### Generate Query Vector

To generate a query vector to be used within a proof, use the following command:

```sh
cargo run --bin vnns-embedder query -p ./path/to/data.json -t "your text to be converted here"
# will output ./path/to/data.query.json
```

This saves the vector itself within the JSON file, which the prover reads from disk.

## Usage

### Build

To build the VNNS program, run the following command:

```sh
cd program
cargo prove build --elf-name riscv32im-succinct-vnns-elf
```

To build the aggregator program:

```sh
cd aggregator
cargo prove build --elf-name riscv32im-succinct-aggregator-elf
```

### Execute

To run the program without generating a proof:

```sh
RUST_LOG=info cargo run --bin vnns-script --release -- --execute --path ./data/foods-small.json
```

This will execute the program and display the output.

### Prove

To generate a core proof for your program:

```sh
RUST_LOG=info cargo run --bin vnns-script --release -- --prove --path ./data/foods-small.json
```

This will generate many proofs (based on file size & batch size) and store them under the same directory as given in `path`. To see which text the result belongs to, copy the `Output Commitment` on the console, and look-up the item within the vector index that has the same hash with that commitment.

> [!TIP]
>
> You can configure the batch size with `--batch-size <number>` argument, default is 4.
> The batch size should be small especially if the vector is large (1000s of elements) because they are all of type `f32` and will consume a lot of resources within the zkVM.

> [!TIP]
>
> If `--aggregate` option is passed, it will aggregate and store the final proof as well with the extension `.agg.proof` and `.agg.pub`.

### Submit

Consider proofs generated for some data `./data.json`. You can submit all batches of proofs to Aligned Layer with:

```sh
./aligned.sh submit ./path/to/keystore.json ./data.json
```

To send the aggregated proof only, you can use:

```sh
./aligned.sh submit-agg ./path/to/keystore.json ./data.json
```

> [!NOTE]
>
> For each proof, it will ask for your keystore password.

## License

The project is MIT licensed as per SP1 project template.
