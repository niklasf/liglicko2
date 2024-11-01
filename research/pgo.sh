#!/bin/sh -e

rm -rf target/pgo-profiles

cargo pgo build

export LLVM_PROFILE_FILE="$PWD/target/pgo-profiles/replay_encounters_%m_%p.profraw"
cat sample-encounters.csv | ./target/x86_64-unknown-linux-gnu/release/replay_encounters > /dev/null

cargo pgo optimize

cp ./target/x86_64-unknown-linux-gnu/release/replay_encounters replay_encounters
