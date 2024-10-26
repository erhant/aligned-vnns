# Embedder

## Usage

Get into this directory:

```sh
cd embedder
```

Type `cargo run help` for available commands. You need to have Ollama running to host the free & open-source models.

```sh
# in a separate terminal, run Ollama
ollama serve

# pull an embedding model
ollama pull nomic-embed-text:latest
```

If you have a file at some path `./path/to/file.json` you can create an object with embeddings with the command below:

```sh
cargo run index -p ./path/to/file.json
```

We currently expect the given path to include a JSON array with `{name, description}` fields, both strings. To make a query and get its embedding, you can do:

```sh
cargo run query -t "your query here"
```

> [!NOTE]
>
> On MacOS, you can pipe the resulting vector to your clipboard as well:
>
> ```sh
> cargo run -t "your query here" | pbcopy
> ```

The vector here is your query vector.

### Proof Generation

To create a proof, at the root directory do:

```sh
RUST_LOG=debug zkRust prove-sp1 ./prover
RUST_LOG=debug zkRust prove-risc0 ./prover
```

This will create proofs under `proof_data` folder.

> [!NOTE]
>
> Proof generation may take 5-10 minutes or so.

> [!WARNING]
>
> `zkRust` expects `src/main.rs` with 3 functions: `main`, `input`, and `output` in that order. Changing the order may break the prover.

### Key Generation

You can create a new key & store it directly in a random file with:

```sh
# tested in MacOS
PRIV_KEY=$(cast wallet new | grep "Private key" | cut -d ' ' -f 3)

# will prompt for a password, can be left empty
cast wallet import --private-key $PRIV_KEY -k ./secrets/ $RANDOM.json
```

You can view the address of a keystore with:

```sh
# will prompt for the password
cast wallet address --keystore ./secrets/wallet.json
```

It will prompt for a password, and then output the keystore under [`secrets`](./secrets/) folder.

### Aligned Layer Verification

When you provide the keystore file to `zkRust` along with `--submit-to-aligned` flag, it will upload the generated proofs:

```sh
RUST_LOG=debug zkRust prove-sp1 ./prover --submit-to-aligned --keystore-path ./secrets/wallet.json
RUST_LOG=debug zkRust prove-risc0 ./prover --submit-to-aligned --keystore-path ./secrets/wallet.json
```

## Benchmarks

A small proof of Calculator language with SHA2 output, length and program output, took ~6 minutes with `--precompiles` flag.
