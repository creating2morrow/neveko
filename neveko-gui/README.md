
## Dev

Note: gui depends on the binary if starting core from gui
otherwise run neveko core on your own

`cd ../ && cargo build && cp target/debug/neveko neveko-gui/ && cd neveko-gui && RUST_LOG=debug cargo run`