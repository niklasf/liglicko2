#!/bin/sh -e

cargo pgo build

export LLVM_PROFILE_FILE="$PWD/target/pgo-profiles/replay_encounters_%m_%p.profraw"
cat sample-encounters.csv | ./target/x86_64-unknown-linux-gnu/release/replay_encounters > /dev/null

cargo pgo optimize

echo "Built: ./target/x86_64-unknown-linux-gnu/release/replay_encounters"
