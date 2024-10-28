# Zero-Knowledge VectorDB

A verifiable nearest-neighbor search program, powered by [SP1](https://github.com/succinctlabs/sp1).

> Built for [Aligned Builders Hackathon](https://devfolio.co/alignedhackathon/).
>
> - [Judging](https://mirror.xyz/0x7794D1c55568270A81D8Bf39e1bcE96BEaC10901/JnG4agqhW0oiskZJgcFdi9SLKvqkTBrbXkuk1nT6lxk).

## Setup

### Installation

You need the following:

- [Rust](https://rustup.rs/) for everything
- [SP1](https://docs.succinct.xyz/getting-started/install.html) for zkVM
- [Ollama](https://ollama.com/) for embeddings
- [Aligned SDK](https://docs.alignedlayer.com/introduction/1_try_aligned#quickstart) for proof verification

### Wallet Generation

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

### Setting up Aligned SDK

First, deposit some funds to Aligned layer (skip this if you already have done so):

```sh
./aligned deposit ./path/to/keystore.json
```

This will print a transaction hash, which can be viewed at `https://holesky.etherscan.io/tx/<hash-here>`.

You can check your balance with:

```sh
./aligned balance ./path/to/keystore.json
```

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

This will generate many proofs (based on file size & batch size) and store them under the same directory as given in `path`.

If `--aggregate` option is passed, it will aggregate and store the final proof as well with the extension `.agg.proof` and `.agg.pub`.

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
