#!/bin/bash

set -euo pipefail

# Ensure the Homebrew Berkeley DB tools are preferred over the macOS system
# binaries. Callers can override this behavior by exporting PATH ahead of time.
export PATH="/opt/homebrew/opt/berkeley-db/bin:${PATH}"

SRC="tests/fixtures/zcashd"
TARGET="dumps/zcashd"

# Fail fast if db_dump cannot parse the fixtures with the currently available
# Berkeley DB binary.
if ! db_dump "${SRC}/wallet0.dat" >/dev/null 2>&1; then
  echo "db_dump failed to parse ${SRC}/wallet0.dat. Ensure a Berkeley DB build" \
       "that understands Btree v10 is installed and on PATH." >&2
  exit 1
fi

run_dump() {
  local input=$1
  local output=$2

  mkdir -p "$(dirname "${output}")"

  echo "Dumping ${input} -> ${output}"
  cargo run --quiet -- --from zcashd --to format "${input}" "${output}"
}

process_group() {
  local version=$1
  shift

  local src_dir
  local target_dir

  if [[ "${version}" == "." ]]; then
    src_dir="${SRC}"
    target_dir="${TARGET}"
  else
    src_dir="${SRC}/${version}"
    target_dir="${TARGET}/${version}"
  fi

  for wallet in "$@"; do
    run_dump "${src_dir}/${wallet}" "${target_dir}/${wallet%.dat}.txt"
  done
}

process_group "golden-v5.6.0" \
  node0_wallet.dat node1_wallet.dat node2_wallet.dat node3_wallet.dat

process_group "sprout" \
  node0_wallet.dat node1_wallet.dat node2_wallet.dat node3_wallet.dat

process_group "tarnished-v5.6.0" \
  node0_wallet.dat node1_wallet.dat node2_wallet.dat node3_wallet.dat

process_group "." \
  wallet0.dat wallet1.dat wallet2.dat wallet3.dat \
  wallet4.dat wallet5.dat wallet6.dat wallet7.dat
