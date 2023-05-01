
## Dev

Note: gui depends on the binary if starting core from gui
otherwise run nevmes core on your own

`cd ../ && cargo build && cp target/debug/nevmes nevmes-gui/ && cd nevmes-gui && RUST_LOG=debug cargo run`