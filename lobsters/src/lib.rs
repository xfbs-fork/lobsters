#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

//! Lobsters Client
//! ===============
//!
//! * [Issue Tracker][issues]
//!
//! What
//! ----
//!
//! This is a pair of Rust crates. One implements an asynchronous HTTP client for
//! the [Lobsters] website, and other websites running its code. The other
//! implements a terminal user interface using the client. Lobsters is a friendly
//! programming oriented link sharing community.
//!
//! This client crate allows the following actions to be performed:
//!
//! * Fetch stories
//! * Fetch comments on stories
//! * Post comments and replies
//! * Login
//!
//! Why
//! ---
//!
//! It did this mostly to practice building asynchronous HTTP clients in Rust and
//! gain more experience with the async ecosystem. Then I needed something to test
//! the client so I built the UI. You can [read more about building the client and
//! TUI on my blog][blog-post].
//!
//! Known Limitations
//! -----------------
//!
//! * Support for 2FA login is not implemented
//!
//! Testing
//! -------
//!
//! Run the test suite:
//!
//!     cargo test
//!
//! Contributing
//! ------------
//!
//! If you have code or patches you wish to contribute, the preferred mechanism is
//! a git pull request. Push your changes to a git repository somewhere (Sourcehut,
//! GitHub, GitLab, whatever). Ensure that contributions don't break [the
//! tests](https://git.sr.ht/~wezm/lobsters#testing) and add new ones when appropriate.
//!
//! Assuming you have followed the [build steps](https://git.sr.ht/~wezm/lobsters#build)
//! above you would do the following to push to your own fork on Sourcehut, change
//! the git URL to match wherever your forked repo is:
//!
//!     git remote rename origin upstream
//!     git remote add origin git@git.sr.ht:~yourname/lobsters
//!     git push -u origin master
//!
//! Then generate the pull request:
//!
//!     git fetch upstream master
//!     git request-pull -p upstream/master origin
//!
//! And copy-paste the result into a plain-text email to wes@wezm.net.
//!
//! You may alternately use a patch-based approach as described on
//! <https://git-send-email.io>.
//!
//! License
//! -------
//!
//! This project is dual licenced under:
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](https://git.sr.ht/~wezm/lobsters/tree/master/LICENSE-APACHE) OR
//!   <http://www.apache.org/licenses/LICENSE-2.0>)
//! - MIT license ([LICENSE-MIT](https://git.sr.ht/~wezm/lobsters/tree/master/LICENSE-MIT) OR
//!   <http://opensource.org/licenses/MIT>)
//!
//! [blog-post]: https://www.wezm.net/technical/2019/04/lobsters-tui/
//! [crate-docs]: https://docs.rs/lobsters
//! [issues]: https://todo.sr.ht/~wezm/lobsters
//! [Lobsters]: https://lobste.rs/
//! [rust]: https://rustup.rs/
//! [rustup]: https://www.rust-lang.org/tools/install

pub mod client;
pub mod error;
pub mod models;

pub use client::Client;
pub use error::Error;
pub use url;

/// URL of lobste.rs. Useful as `base_url` to `Client`
pub const URL: &str = "https://lobste.rs/";
