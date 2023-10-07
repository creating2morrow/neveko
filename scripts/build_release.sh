#!/bin/bash
# Build neveko release
# Run from the neveko root
# usage: ./scripts/build_release vX.X.X-ver

# Linux x86_64 output directory

LINUX_X86_64="x86_64-linux-gnu"
RELEASE_NAME="neveko-$LINUX_X86_64-$1"
LINUX_OUTPUT_DIR=".build/release/$RELEASE_NAME"

mkdir -p $LINUX_OUTPUT_DIR
cargo build --release
cp target/release/neveko $LINUX_OUTPUT_DIR
cd neveko-gui && cargo build --release && cp target/release/neveko_gui ../$LINUX_OUTPUT_DIR
cp -r assets/ ../$LINUX_OUTPUT_DIR
cd ../
cd neveko-auth && cargo build --release && cp target/release/neveko_auth ../$LINUX_OUTPUT_DIR
cd ../
cd neveko-contact && cargo build --release && cp target/release/neveko_contact ../$LINUX_OUTPUT_DIR
cd ../
cd neveko-market && cargo build --release && cp target/release/neveko_market ../$LINUX_OUTPUT_DIR
cd ../
cd neveko-message && cargo build --release && cp target/release/neveko_message ../$LINUX_OUTPUT_DIR
cd ../
# make the bzip for linux
cd .build/release/ && tar -cjf $RELEASE_NAME.tar.bz2 $RELEASE_NAME/ && mv $RELEASE_NAME.tar.bz2 ../../
