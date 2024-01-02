#!/bin/bash
# Run from the neveko root 
cd neveko-auth && cargo check
cd ../neveko-contact && cargo check
cd ../neveko-core && cargo check
cd ../neveko-gui && cargo check
cd ../neveko-market && cargo check
cd ../neveko-message && cargo check
cd ../ && cargo check

