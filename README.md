# Event Store in Rust

[![Build Status](https://travis-ci.org/repositive/event-store-rs.svg?branch=master)](https://travis-ci.org/repositive/event-store-rs)

Event store, but in Rust

## Install

Add the following to Cargo.toml in your project:

```bash
event-store = { git = "https://github.com/repositive/event-store-rs.git" }
event-store-derive = { git = "https://github.com/repositive/event-store-rs.git" }
event-store-derive-internals = { git = "https://github.com/repositive/event-store-rs.git" }
```

## Setup

Currently requires Rust Nightly. Install it with `rustup toolchain add nightly`.

Run tests with `cargo +nightly test`.

## Documentation

```bash
cargo doc --open
```
