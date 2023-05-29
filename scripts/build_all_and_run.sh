#!/bin/bash
cd nevmes-auth && cargo build
cd ../nevmes-contact && cargo build
cd ../nevmes-core && cargo build
cd ../nevmes-gui && cargo build
cd ../nevmes-market && cargo build
cd ../nevmes-message && cargo build
cd ../ && RUST_LOG=debug cargo run $1
