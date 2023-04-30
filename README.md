# NEVMES

NEVidebla-MESago (invisible message)

### gpg and i2p made simple for end-to-end encrypted, secure comms

## About

* send messages over the invisible internet
* vanity base32 addresses (advanced)
* automated mandatory gpg key encryption
* xmr payment integration

## Dev

* stack - rust (egui, rocket), lmdb, i2p-zero, monero(rpc, daemon), gpg
* install dependencies
    * ubuntu example: `sudo apt update -y && sudo apt upgrade -y`
    * `sudo apt install -y libssl-dev build-essential libgpgme-dev`
* download and run i2prouter start (optional: setup to run on boot similar tor daemon)
* `git clone https://github/com/creating2morrow/nevmes`
* `cd nevmes && ./scripts/build_all_and_run.sh "-- -h"`
* gui built with rust [egui](https://docs.rs/egui/latest/egui/)

## API

* remote/programmatic access
* secured by wallet signing
* jwt and jwp
* see [curl.md](./docs/curl.md)

## Binaries

* nevmes-auth - `internal` auth server
* nevmes-contact - `internal` add contacts server
* nevmes-core - application core logic
* nevmes-gui - primary user interface
* nevmes-message - `internal` message tx/read etc. server
* nevmes - `external` primary server for contact share, payment, message rx etc.
* [monerod](https://www.getmonero.org/downloads/#cli) - (not included) monero-wallet-rpc needs this
    * can be overriden with remote node
    * use the `--remote-node` flag
* [monero-wallet-rpc](https://www.getmonero.org/downloads/#cli) - (not included) interface for xmr wallet ops
* [i2p-zero](https://github.com/i2p-zero/i2p-zero/releases/tag/v1.20) - (not included) tunnel creation
* [i2p](https://geti2p.net/en/download) - http proxy (not included, *i2p-zero http proxy not working)

## Manual

[the manual](./docs/man.md)

## Known issues

* gui password and screen lock needs fixing up
* timeout out JWP payment approval screen with infinite loading
* message retry login (untested)
* test framework (in progress)
* docs on all `fn` and `structs`
* i2pd installer on home screen?
* and more daemon info and wallet functionality (multisig)
