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

Run tests using `cargo +nightly test`.

## Documentation

```bash
cargo doc --open
```

## Explorer

### Flatpak Build for Linux (Flatpak)

Ensure you're in `./explorer`.

* Linux Mint: `apt install elfutils flatpak-builder`
* `./build-flatpak.sh`
* Flatpak is built to `explorer/flatpak-build/explorer.flatpak`


### Install on Linux (Flatpak)

* Download `explorer.flatpak`
* Install with `flatpak install path/to/explorer.flatpak`
* OR run it from the build folder with `flatpak-builder --run flatpak-build/explorer org.repositive.EventStoreExplorer.json explorer`
* Run with flatpak run org.repositive.EventStoreExplorer
* You can make the explorer use a different GTK theme by following [these instructions](https://www.linuxuprising.com/2018/05/how-to-get-flatpak-apps-to-use-correct.html), e.g. `flatpak install flathub org.gtk.Gtk3theme.Arc-Dark`
