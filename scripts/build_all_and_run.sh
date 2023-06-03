#!/bin/bash
cd neveko-auth && cargo build
cd ../neveko-contact && cargo build
cd ../neveko-core && cargo build
cd ../neveko-gui && cargo build
cd ../neveko-market && cargo build
cd ../neveko-message && cargo build
cd ../ && RUST_LOG=debug cargo run $1
