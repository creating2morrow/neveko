#!/bin/bash
# Run from the neveko root 
cd neveko-auth && cargo +nightly fmt
cd ../neveko-contact && cargo +nightly fmt
cd ../neveko-core && cargo +nightly fmt
cd ../neveko-gui && cargo +nightly fmt
cd ../neveko-market && cargo +nightly fmt
cd ../neveko-message && cargo +nightly fmt
cd ../ && cargo +nightly fmt
