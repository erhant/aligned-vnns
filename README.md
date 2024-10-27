# Zero-Knowledge VectorDB

A verifiable HNSW query program, powered by [SP1](https://github.com/succinctlabs/sp1).

> Built for Aligned Builders Hackathon, see [judging criteria](https://mirror.xyz/0x7794D1c55568270A81D8Bf39e1bcE96BEaC10901/JnG4agqhW0oiskZJgcFdi9SLKvqkTBrbXkuk1nT6lxk).

## Installation

You need the following:

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/getting-started/install.html)
- [Ollama](https://ollama.com/)
- [Aligned SDK](https://docs.alignedlayer.com/introduction/1_try_aligned#quickstart)

## Usage

### Build the Program

To build the program, run the following command:

```sh
cd program
cargo prove build
```

### Execute the Program

To run the program without generating a proof:

```sh
cd script
RUST_LOG=info cargo run --release -- --execute
```

This will execute the program and display the output.

### Generate a Core Proof

To generate a core proof for your program:

```sh
cd script
RUST_LOG=info cargo run --release -- --prove
```

This will save `sp1.proof` and `sp1.pub` within the [`script`](./script/) directory.

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
aligned deposit-to-batcher \
--rpc_url https://ethereum-holesky-rpc.publicnode.com \
--network holesky \
--keystore_path ./secrets/wallet.json \
--amount 0.1ether
```

This will print a transaction hash, which can be viewed at `https://holesky.etherscan.io/tx/<hash-here>`.

Confirm that you have enough balance

```sh
aligned get-user-balance \
--rpc_url https://ethereum-holesky-rpc.publicnode.com \
--network holesky \
--user_addr 0xB1ae88120FbF7F58348Fb9DC74a9cEb258f60c5E
```

### Sending a Proof

You can send a proof with:

```sh
aligned submit \
--proving_system SP1 \
--proof ./data/foods-smol.sp1.proof \
--public_input ./data/foods-smol.sp1.pub \
--vm_program ./elf/riscv32im-succinct-zkvm-elf \
--batcher_url wss://batcher.alignedlayer.com \
--keystore_path ./secrets/wallet.json \
--network holesky \
--rpc_url https://ethereum-holesky-rpc.publicnode.com
```

### Sending an Aggregated Proof

```sh
aligned submit \
--proving_system SP1 \
--proof ./data/foods-med.sp1.proof \
--public_input ./data/foods-med.sp1.pub \
--vm_program ./elf/riscv32im-succinct-aggregator-elf \
--batcher_url wss://batcher.alignedlayer.com \
--keystore_path ./secrets/wallet.json \
--network holesky \
--rpc_url https://ethereum-holesky-rpc.publicnode.com
```
