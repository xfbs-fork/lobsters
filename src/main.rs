use std::env;

use futures::future::{self, Future};

use lobsters::{
    client::{Client, UnauthenticatedClient},
    error::Error,
};

fn main() {
    let username = env::var("LOBSTERS_USER").expect("LOBSTERS_USER must be set");
    let password = env::var("LOBSTERS_PASS").expect("LOBSTERS_PASS must be set");
    let base_url = lobsters::client::LOBSTERS
        .parse()
        .expect("base url is invalid");

    let client = UnauthenticatedClient::new(base_url).expect("URL is not https");

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    let work = client.login(username, password);
    let client = rt.block_on(work).expect("error logging in");

    client.save_cookies().expect("unable to save cookies");
}
