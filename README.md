# Zero-Knowledge VectorDB

A verifiable HNSW query program, powered by [SP1](https://github.com/succinctlabs/sp1).

## Installation

You need the following:

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/getting-started/install.html)
- [Ollama](https://ollama.com/)

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
cargo run --release -- --execute
```

This will execute the program and display the output.

### Generate a Core Proof

To generate a core proof for your program:

```sh
cd script
cargo run --release -- --prove
```

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
