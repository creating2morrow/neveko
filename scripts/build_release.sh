#!/bin/bash
# Build nevmes release
# Run from the nevmes root
# usage: ./scripts/build_release x.x.x-ver

# Linux x86_64 output directory

LINUX_X86_64="x86_64-linux-gnu"
RELEASE_NAME="nevmes-$LINUX_X86_64-v$1"
LINUX_OUTPUT_DIR=".build/release/$RELEASE_NAME"

mkdir -p $LINUX_OUTPUT_DIR
cargo build --release
cp target/release/nevmes $LINUX_OUTPUT_DIR
cd nevmes-gui && cargo build --release && cp target/release/nevmes_gui ../$LINUX_OUTPUT_DIR
cp -r assets/ ../$LINUX_OUTPUT_DIR
cd ../
cd nevmes-auth && cargo build --release && cp target/release/nevmes_auth ../$LINUX_OUTPUT_DIR
cd ../
cd nevmes-contact && cargo build --release && cp target/release/nevmes_contact ../$LINUX_OUTPUT_DIR
cd ../
cd nevmes-message && cargo build --release && cp target/release/nevmes_message ../$LINUX_OUTPUT_DIR
cd ../
make the bzip for linux
cd .build/release/ && tar -cjf $RELEASE_NAME.tar.bz2 $RELEASE_NAME/ && mv $RELEASE_NAME.tar.bz2 ../../