#!/bin/bash
# Run from the nevmes root 
cd nevmes-auth && cargo clean
cd ../nevmes-contact && cargo clean
cd ../nevmes-core && cargo clean
cd ../nevmes-gui && cargo clean
cd ../nevmes-market && cargo clean
cd ../nevmes-message && cargo clean
cd ../ && cargo clean
rm -rf .build/
