# NEVEKO

NEVidebla-EKOnomia (invisible economy)

[![cargo-build](https://github.com/creating2morrow/neveko/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/creating2morrow/neveko/actions/workflows/rust.yml)
[![cargo-audit](https://github.com/creating2morrow/neveko/actions/workflows/audit.yml/badge.svg?branch=main)](https://github.com/creating2morrow/neveko/actions/workflows/audit.yml)

![home](./assets/home.png)

### i2p made simple for E2EE marketplace, messaging and more

## About

* send messages over the invisible internet
* vanity base32 addresses (advanced)
* automated mandatory message encipher
* self-hosted i2p marketplace
* xmr multisig / payment integration

## Dev

* stack - rust (egui, rocket), lmdb, i2p, monero(rpc, daemon)
* install dependencies
    * ubuntu example: `sudo apt update -y && sudo apt upgrade -y`
    * `sudo apt install -y libssl-dev build-essential`
* `git clone --recursive https://github.com/creating2morrow/neveko`
* `cd neveko && ./scripts/build_all_and_run.sh "-- -h"`
* Example to start neveko with remote stagenet node / i2p proxy remote for development:
    * `./scripts/build_all_and_run.sh "-- --monero-location monero-x86_64-linux-gnu-v0.18.3.4 --monero-rpc-host http://127.0.0.1:18083 --monero-rpc-daemon http://xmr3kaacphwkk4z2gp35bdl47lrrnzimmyqj4oliauqrjzqecofa.b32.i2p:18081 --monero-rpc-username user --monero-rpc-cred pass --remote-node --i2p-advanced --i2p-tunnels-json /home/user/neveko/i2p-manual/config --i2p-proxy-host http://x.x.x.x:xxxx --i2p-socks-proxy-host http://x.x.x.x:xxxx"`
    * the `--monero-location` flag is needed even when using a remote node because
      neveko has its own monero-wallet-rpc instance
    * remote nodes are forced over the `--i2p-socks-proxy-host`
* Recommended neveko-core startup with full node:
    * ` ./scripts/build_all_and_run.sh "-- --monero-blockchain-dir=/home/user/.bitmonero --monero-location monero-x86_64-linux-gnu-v0.18.3.4 --monero-blockchain-dir /home/user/.bitmonero"`
    * monerod doesn't need to be running because neveko will start it and monero-wallet-rpc
    * gui will automatically detect monerod, rpc if neveko core is started first
* Neveko doesn't write logs to file. Use the command below to write to a log file:
  ```bash 
    {NEVEKO_START_CMDS} > neveko.log 2>&1
  ```
  * just remember to put cli password in the original window, not the log file window
  * https://stackoverflow.com/questions/6674327/redirect-all-output-to-file-in-bash
* gui built with rust [egui](https://docs.rs/egui/latest/egui/)
* copy the `certificates` directory from `j4-i2p-rs` to `neveko` root
* see [j4-i2p-rs](https://github.com/kn0sys/j4-i2p-rs) for more information on embedded i2p
* darknet release server links are located at: http://neveko.i2p/index.txt

## Contributing and Releasing

```bash
| branch |                 |tag and release|
  dev     -----------------|-------------------------------------------->
  v0.1.0  -----------tag v0.1.0 (delete branch)
  v0.2.0                   |-------------------------------------------->
  main    -------------------------------------------------------------->
```

* code on dev branch
* run `./scripts/fmtall.sh` before committing
* pull request to dev branch
* todo => `TODO(name): detailed work`
* docs on all `pub fn` and `pub struct`
* merge dev to vX.X.X
* merge vX.X.X to main
* tag release v.X.X.X every friday (if stable changes)
* release binaries from the `cargo-build-release` workflow with notes
* create next v.X.X+1.X branch and delete old release branch
* release bug fixes as appropriate to v.X.X.X+1 branch when ready

## Workflows

|name                | branch   | purpose                                     |
|--                  |--        |--                                           |
|cargo-build         | main,dev | ensure code compilation and build success   |
|cargo-audit         | main,dev | run security audit against RustSec database |
|cargo-build-release | `v0.*`   | publish production ready binaries           |

## API

* remote/programmatic access
* secured by wallet signing
* jwt and jwp
* see [curl.md](./docs/curl.md)

## Binaries

* neveko-auth - `internal` auth server
* neveko-contact - `internal` add contacts server
* neveko-gui - primary user interface
* neveko-market - `internal` marketplace admin server
* neveko-message - `internal` message tx/read etc. server
* neveko - `external` primary server for contact share, payment, market, message rx etc.
* [monerod](https://www.getmonero.org/downloads/#cli) - monero-wallet-rpc needs this
    * can be overriden with remote node
    * use the `--remote-node` flag
* [monero-wallet-rpc](https://www.getmonero.org/downloads/#cli) - interface for xmr wallet ops

most of the complex logic stays in neveko-core, exported from [lib.rs](./neveko-core/src/lib.rs)

## Manual

[the manual](./docs/man.md)

## Donations

This is a research project as of v0.1.0-beta but if anything here is useful donations are much appreciated!
Features and bug fixes aren't guaranteed by donations but they will supply coffee for devs on
sleepless nights!

87TzQS4g6mN4oAcEhcnEHGCxw9bFwMXR8WHJEEZoCd7tPHgcH3NsiCF5FSWSkKYVa7EYJjuosPZBiNAh9LqHaRSiBUhsAcC
