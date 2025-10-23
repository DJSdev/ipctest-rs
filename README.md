# ipctest-rs

A Rust workspace for exploring and testing inter-process communication (IPC) patterns.
This repository contains a Rust `core` binary, event generating modules under `events/`, and Protocol Buffers definitions under `protos/`.

## Build

```bash
# all
cargo build

# core
cargo build --bin core

# events
cargo build --bin events

# protobufs
cargo build --lib
```
