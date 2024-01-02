#!/bin/bash

# set the latest pre-commit script
cp -u scripts/pre-commit.sample .git/hooks/pre-commit

cd neveko-auth && cargo build
cd ../neveko-contact && cargo build
cd ../neveko-core && cargo build
cd ../neveko-gui && cargo build
cd ../neveko-market && cargo build
cd ../neveko-message && cargo build
cd ../ && RUST_LOG=debug cargo run $1
