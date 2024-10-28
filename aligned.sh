#!/bin/sh

NETWORK=holesky
RPC_URL=https://ethereum-holesky-rpc.publicnode.com

# Deposits 0.1 ether to the batcher contract
deposit() {
  local keystore_path=$1
  aligned deposit-to-batcher \
    --rpc_url $RPC_URL \
    --network $NETWORK \
    --keystore_path "$keystore_path" \
    --amount 0.1ether
}

# Reads the keystore and prints balance
balance() {
  local keystore_path=$1
  local address=$(cast wallet address --keystore "$keystore_path")

  aligned get-user-balance \
    --rpc_url $RPC_URL \
    --network $NETWORK \
    --user_addr "$address"
}

submitagg() {
  local keystore_path=$1
  local json_path=$2
  local proof_path="${json_path%.json}.agg.proof"
  local pub_path="${json_path%.json}.agg.pub"

  if [ ! -f "$proof_path" ] || [ ! -f "$pub_path" ]; then
    echo "Required files not found: $proof_path or $pub_path"
    exit 1
  fi

  # submit using aggregator elf
  aligned submit \
    --proving_system SP1 \
    --proof "$proof_path" \
    --public_input "$pub_path" \
    --vm_program ./elf/riscv32im-succinct-aggregator-elf \
    --batcher_url wss://batcher.alignedlayer.com \
    --keystore_path "$keystore_path" \
    --network $NETWORK \
    --rpc_url $RPC_URL
}

submit() {
  local keystore_path=$1
  local json_path=$2

  local index=0
  while true; do
    local proof_path="${json_path%.json}.$index.proof"
    local pub_path="${json_path%.json}.$index.pub"

    if [ ! -f "$proof_path" ] || [ ! -f "$pub_path" ]; then
      echo "Submitted $index proofs in total."
      break
    fi

    echo "Submitting proof $index"
    # submit using VNNS elf
    aligned submit \
      --proving_system SP1 \
      --proof "$proof_path" \
      --public_input "$pub_path" \
      --vm_program ./elf/riscv32im-succinct-vnns-elf \
      --batcher_url wss://batcher.alignedlayer.com \
      --keystore_path "$keystore_path" \
      --network $NETWORK \
      --rpc_url $RPC_URL

    index=$((index + 1))
  done
}

### Main ###

if [ $# -lt 2 ]; then
  echo "Usage: $0 <function_name> <keystore_path>"
  exit 1
fi

function_name=$1
case $function_name in
  deposit)
    deposit $2
    ;;
  balance)
    balance $2
    ;;
  submit-agg)
    submitagg $2 $3
    ;;
  submit)
    submit $2 $3
    ;;
  *)
    echo "Invalid function name: $function_name"
    exit 1
    ;;
esac
