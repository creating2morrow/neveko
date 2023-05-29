#!/bin/bash
# Run from the nevmes root 
cd nevmes-auth && cargo +nightly fmt
cd ../nevmes-contact && cargo +nightly fmt
cd ../nevmes-core && cargo +nightly fmt
cd ../nevmes-gui && cargo +nightly fmt
cd ../nevmes-market && cargo +nightly fmt
cd ../nevmes-message && cargo +nightly fmt
cd ../ && cargo +nightly fmt
