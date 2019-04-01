Lobsters client for Rust
========================

Build Status:

* Debian: [![builds.sr.ht Debian status](https://builds.sr.ht/~wezm/lobsters/debian.yml.svg)](https://builds.sr.ht/~wezm/lobsters/debian.yml?)
* FreeBSD: [![builds.sr.ht FreeBSD status](https://builds.sr.ht/~wezm/lobsters/freebsd.yml.svg)](https://builds.sr.ht/~wezm/lobsters/freebsd.yml?)

## What

This is a Rust crate that implements an asynchronous HTTP client for the
[Lobsters] website, and other websites running its code. Lobsters is a friendly
tech oriented link sharing community.

This crate allows the following to be performed with the client:

* Fetch stories
* Fetch comments on stories
* Post comments and replies

## Why

It did this mostly to practice building asynchronous HTTP clients in Rust and
gain more experience with the async ecosystem.

## How

Check out the binary that's part of the crate (main.rs) for sample usage.

<!--
## Installing

### From Binary Release

[Latest Release][release]

`lobsters` is a single small binary. To download the latest release do the following:

    curl -L https://releases.wezm.net/lobsters/lobsters-v0.3.0-arm-unknown-linux-gnueabihf.tar.gz | tar zxf -

The binary should be in your current directory and can be run as follows:

    ./lobsters

Feel free to move it elsewhere (`~/.local/bin` for example).
## From Source

**Note:** You will need the [Rust compiler installed][rust].

    git clone https://git.sr.ht/~wezm/lobsters
    cargo install --path lobsters
-->

## Known Limitations

* Support for 2FA login is not implemented

## License

This project is dual licenced under:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://git.sr.ht/~wezm/lobsters/tree/master/LICENSE-APACHE) OR
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://git.sr.ht/~wezm/lobsters/tree/master/LICENSE-MIT) OR
  <http://opensource.org/licenses/MIT>)

[Lobsters]: https://lobste.rs/
