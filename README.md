Lobsters Client and TUI in Rust
===============================

[![crates.io](https://img.shields.io/crates/v/lobsters.svg)](https://crates.io/crates/lobsters)
[![Documentation](https://docs.rs/lobsters/badge.svg)][crate-docs]

<img src="https://git.sr.ht/~wezm/lobsters/blob/master/screenshot.png" alt="Screenshot of lobsters in a terminal window" width="568" />

* [Issue Tracker][issues]
* [Download](https://git.sr.ht/~wezm/lobsters#download)
* [Build](https://git.sr.ht/~wezm/lobsters#building)
* [Testing](https://git.sr.ht/~wezm/lobsters#testing)
* [Contributing](https://git.sr.ht/~wezm/lobsters#contributing)

What
----

This is a pair of Rust crates. One implements an asynchronous HTTP client for
the [Lobsters] website, and other websites running its code. The other
implements a terminal user interface using the client. Lobsters is a friendly
programming oriented link sharing community.

This client crate allows the following actions to be performed:

* Fetch stories
* Fetch comments on stories
* Post comments and replies
* Login

Why
---

It did this mostly to practice building asynchronous HTTP clients in Rust and
gain more experience with the async ecosystem. Then I needed something to test
the client so I built the UI. You can [read more about building the client and
TUI on my blog][blog-post].

How
---

The `lobsters-cli` crate in this repo provides an example of the crate in use.
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

Download
--------

`lobsters` is a single binary available for a handful of platforms. To download
the latest release do the following:

* FreeBSD 12.0 x86_64:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-x86_64-unknown-freebsd.tar.gz | tar zxf -`
* Linux x86_64:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-x86_64-unknown-linux-musl.tar.gz | tar zxf -`
* macOS x86_64:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-x86_64-apple-darwin.tar.gz | tar zxf -`
* NetBSD 8.0 x86_64:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-x86_64-unknown-netbsd.tar.gz | tar zxf -`
* OpenBSD 6.5 x86_64:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-x86_64-unknown-openbsd.tar.gz | tar zxf -`
* Raspberry Pi:
  * `curl -L https://releases.wezm.net/lobsters/lobsters-0.1.0-arm-unknown-linux-gnueabihf.tar.gz | tar zxf -`

The binary should be in your current directory and can be run as follows:

    ./lobsters

Feel free to move it elsewhere (`~/.local/bin` for example).

Building
--------

Build Status:

* Debian: [![builds.sr.ht Debian status](https://builds.sr.ht/~wezm/lobsters/debian.yml.svg)](https://builds.sr.ht/~wezm/lobsters/debian.yml?)
* FreeBSD: [![builds.sr.ht FreeBSD status](https://builds.sr.ht/~wezm/lobsters/freebsd.yml.svg)](https://builds.sr.ht/~wezm/lobsters/freebsd.yml?)

### From Source

**Note:** You will need the [Rust compiler installed][rust].

    git clone https://git.sr.ht/~wezm/lobsters
    cargo install --path lobsters/lobsters-cli

### Cross-Compiling

There is a script that will build binaries for several platforms. You will need
an `arm-linux-gnueabihf` and `musl` toolchain installed as well as those [rustup]
targets installed.

    ./build-all-platforms

Known Limitations
-----------------

* Support for 2FA login is not implemented

Testing
-------

Run the test suite:

    cargo test

Contributing
------------

If you have code or patches you wish to contribute, the preferred mechanism is
a git pull request. Push your changes to a git repository somewhere (Sourcehut,
GitHub, GitLab, whatever). Ensure that contributions don't break [the
tests](https://git.sr.ht/~wezm/lobsters#testing) and add new ones when appropriate.

Assuming you have followed the [build steps](https://git.sr.ht/~wezm/lobsters#build)
above you would do the following to push to your own fork on Sourcehut, change
the git URL to match wherever your forked repo is:

    git remote rename origin upstream
    git remote add origin git@git.sr.ht:~yourname/lobsters
    git push -u origin master

Then generate the pull request:

    git fetch upstream master
    git request-pull -p upstream/master origin

And copy-paste the result into a plain-text email to wes@wezm.net.

You may alternately use a patch-based approach as described on
<https://git-send-email.io>.

License
-------

This project is dual licenced under:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://git.sr.ht/~wezm/lobsters/tree/master/LICENSE-APACHE) OR
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://git.sr.ht/~wezm/lobsters/tree/master/LICENSE-MIT) OR
  <http://opensource.org/licenses/MIT>)

[blog-post]: https://www.wezm.net/technical/2019/04/lobsters-tui/
[crate-docs]: https://docs.rs/lobsters
[issues]: https://todo.sr.ht/~wezm/lobsters
[Lobsters]: https://lobste.rs/
[rust]: https://rustup.rs/
[rustup]: https://www.rust-lang.org/tools/install
