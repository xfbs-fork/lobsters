#![warn(rust_2018_idioms)]
#![warn(missing_docs)]

//! # Lobsters client for Rust
//!
//! * [Repository](https://git.sr.ht/~wezm/lobsters)
//! * [Documentation](https://docs.rs/lobsters)
//!
//! ## What
//!
//! This is a Rust crate that implements an asynchronous HTTP client for the
//! [Lobsters] website, and other websites running its code. Lobsters is a friendly
//! tech oriented link sharing community.
//!
//! This crate allows the following to be performed with the client:
//!
//! * Fetch stories
//! * Fetch comments on stories
//! * Post comments and replies
//!
//! ## Why
//!
//! It did this mostly to practice building asynchronous HTTP clients in Rust and
//! gain more experience with the async ecosystem.
//!
//! ## How
//!
//! Check out the binary that's part of the crate (main.rs) for sample usage.
//!
//! ## License
//!
//! This project is dual licenced under:
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](https://git.sr.ht/~wezm/lobsters/tree/master/LICENSE-APACHE) **or**
//!   <http://www.apache.org/licenses/LICENSE-2.0>)
//! - MIT license ([LICENSE-MIT](https://git.sr.ht/~wezm/lobsters/tree/master/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
//!
//! [Lobsters]: https://lobste.rs/

pub mod client;
pub mod error;
pub mod models;

pub use client::Client;
pub use error::Error;
pub use url;

/// URL of lobste.rs. Useful as `base_url` to `Client`
pub const URL: &str = "https://lobste.rs/";
