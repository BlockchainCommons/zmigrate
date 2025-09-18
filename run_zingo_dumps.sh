#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

SRC="tests/fixtures/zingo"
TARGET="dumps/zingo"

run_dump() {
  local wallet=$1
  local input="${SRC}/${wallet}.dat"
  local output="${TARGET}/${wallet}.txt"

  mkdir -p "$(dirname "${output}")"

  echo "Dumping ${input} -> ${output}"
  cargo run --quiet --features zingo -- --from zingo --to dump "${input}" "${output}"
}

wallets=(
  "regtest/aaaaaaaaaaaaaaaaaaaaaaaa-v26"
  "mainnet/hhcclaltpcckcsslpcnetblr-gf0aaf9347"
  "mainnet/hhcclaltpcckcsslpcnetblr-latest"
  # "mainnet/vtfcorfbcbpctcfupmegmwbp-v28" # large
  "regtest/aadaalacaadaalacaadaalac-orch-and-sapling"
  "regtest/aadaalacaadaalacaadaalac-orch-only"
  "regtest/hmvasmuvwmssvichcarbpoct-v27"
  "testnet/G93738061a"
  "testnet/Gab72a38b"
  "testnet/cbbhrwiilgbrababsshsmtpr-latest"
  "testnet/glory_goddess"
  "testnet/latest"
  "testnet/v26"
  # "testnet/v27" # large
  "testnet/v28"
)

for wallet in "${wallets[@]}"; do
  run_dump "${wallet}"
done
