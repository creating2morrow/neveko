#!/bin/bash
# Build nevmes release
# Run from the nevmes root
# usage: ./scripts/build_release x.x.x-ver

OUTPUT_DIR=".build/release/nevmes-v$1"
mkdir -p $OUTPUT_DIR
cargo build --release
cp target/release/nevmes $OUTPUT_DIR
cd nevmes-gui && cargo build --release && cp target/release/nevmes_gui ../$OUTPUT_DIR
cp -r assets/ ../$OUTPUT_DIR
cd ../
cd nevmes-auth && cargo build --release && cp target/release/nevmes_auth ../$OUTPUT_DIR
cd ../
cd nevmes-contact && cargo build --release && cp target/release/nevmes_contact ../$OUTPUT_DIR
cd ../
cd nevmes-message && cargo build --release && cp target/release/nevmes_message ../$OUTPUT_DIR
cd ../
