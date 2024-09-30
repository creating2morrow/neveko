#!/bin/bash
# Build neveko release
# Run from the neveko root
# usage: ./scripts/build_release vX.X.X-ver

# Linux x86_64 output directory
LINUX_X86_64="x86_64-linux-gnu"
RELEASE_NAME="neveko-$LINUX_X86_64-$1"
LINUX_OUTPUT_DIR=".build/release/$RELEASE_NAME"
mkdir -p $LINUX_OUTPUT_DIR
# monero version
MONERO_VERSION="monero-$LINUX_X86_64-v0.18.3.4"
# build jars for j4-i2p-rs
git clone --depth 1 https://github.com/kn0sys/i2p.i2p
cd i2p.i2p && ant buildRouter buildI2PTunnelJars buildSAM jbigi buildAddressbook
mkdir -p ../opt/j4-i2p-rs/jassets && cp build/* ../opt/j4-i2p-rs/jassets/
cd ../
# certificates for reseed
cp -r j4-i2p-rs/certificates $LINUX_OUTPUT_DIR
# download monero and extract monero wallet rpc
wget https://downloads.getmonero.org/cli/linux64
mv linux64 $MONERO_VERSION.tar.bz2
tar xvf $MONERO_VERSION.tar.bz2
mkdir $LINUX_OUTPUT_DIR/$MONERO_VERSION
cp $MONERO_VERSION/monero-wallet-rpc $LINUX_OUTPUT_DIR/$MONERO_VERSION
cp $MONERO_VERSION/monerod $LINUX_OUTPUT_DIR/$MONERO_VERSION
# build neveko-core
cargo build --release
# j4-i2p-rs dependencies
cp -r j4-i2p-rs/opt/j4-i2p-rs/deps opt/j4-i2p-rs
cp j4-i2p-rs/opt/j4-i2p-rs/jassets/j4rs-0.20.0-jar-with-dependencies.jar opt/j4-i2p-rs/jassets
cp -r opt/ $LINUX_OUTPUT_DIR
cp target/release/neveko $LINUX_OUTPUT_DIR
# build gui
cd neveko-gui && cargo build --release && cp target/release/neveko_gui ../$LINUX_OUTPUT_DIR
cp -r assets/ ../$LINUX_OUTPUT_DIR
cd ../
# build dev servers for API use
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
