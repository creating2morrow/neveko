#!/bin/bash
# Run from the neveko root 
cd neveko-auth && cargo clean
cd ../neveko-contact && cargo clean
cd ../neveko-core && cargo clean
cd ../neveko-gui && cargo clean
cd ../neveko-market && cargo clean
cd ../neveko-message && cargo clean
cd ../ && cargo clean
rm -rf .build/
