Lobsters client for Rust
========================

Build Status:

* Debian: [![builds.sr.ht Debian status](https://builds.sr.ht/~wezm/lobsters/debian.yml.svg)](https://builds.sr.ht/~wezm/lobsters/debian.yml?)
* FreeBSD: [![builds.sr.ht FreeBSD status](https://builds.sr.ht/~wezm/lobsters/freebsd.yml.svg)](https://builds.sr.ht/~wezm/lobsters/freebsd.yml?)

<img src="https://git.sr.ht/~wezm/lobsters/blob/master/screenshot.png" alt="Screenshot of lobsters in a terminal window" width="568" />

## What

This is a pair of Rust crates. One implements an asynchronous HTTP client for
the [Lobsters] website, and other websites running its code. The other
implements a terminal user interface using the client. Lobsters is a friendly
programming oriented link sharing community.

This client crate allows the following to be performed with the client:

* Fetch stories
* Fetch comments on stories
* Post comments and replies
* Login

## Why

It did this mostly to practice building asynchronous HTTP clients in Rust and
gain more experience with the async ecosystem. Then I needed something to test
the client and ended up building the UI.
<!-- [I wrote more about it on my blog] -->

## How

The lobsters-cli crate in this repo provides an example of the crate in use.
You can try out out by downloading a pre-compiled binary, available below.

### Keyboard bindings

The TUI uses the following key bindings:

* `j` or `↓` — Move cursor down
* `k` or `↑` — Move cursor up
* `h` or `←` — Scroll view left
* `l` or `→` — Scroll view right
* `Enter` — Open story URL in browser
* `c` — Open story comments in browser
* `q` or `Esc` — Quit

## Installing

### From Binary Release

`lobsters` is a single binary available for a handful of platforms. To download
the latest release do the following:

* FreeBSD x86_64:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-x86_64-unknown-freebsd.tar.gz | tar zxf -`
* Linux x86_64:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-x86_64-unknown-linux-musl.tar.gz | tar zxf -`
* macOS x86_64:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-x86_64-apple-darwin.tar.gz | tar zxf -`
* NetBSD x86_64:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-x86_64-unknown-netbsd.tar.gz | tar zxf -`
* Raspberry Pi:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-arm-unknown-linux-gnueabihf.tar.gz | tar zxf -`

The binary should be in your current directory and can be run as follows:

    ./lobsters

Feel free to move it elsewhere (`~/.local/bin` for example).

## Building

### From Source

**Note:** You will need the [Rust compiler installed][rust].

    git clone https://git.sr.ht/~wezm/lobsters
    cargo install --path lobsters

### Cross-Compiling

There is a script that will build binaries for several platforms. You will need
an `arm-linux-gnueabihf` and `musl` toolchain installed as well as those rustup
targets installed.

    ./build-all-platforms

## Known Limitations

* Support for 2FA login is not implemented

## License

This project is dual licenced under:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://git.sr.ht/~wezm/lobsters/tree/master/LICENSE-APACHE) OR
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://git.sr.ht/~wezm/lobsters/tree/master/LICENSE-MIT) OR
  <http://opensource.org/licenses/MIT>)

[Lobsters]: https://lobste.rs/
[rust]: https://rustup.rs/
