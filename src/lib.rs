#![warn(rust_2018_idioms)]

pub mod client;
pub mod error;
pub mod models;

/// URL of lobste.rs
///
/// Useful as `base_url` to `Client`
pub const URL: &str = "https://lobste.rs/";

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
