use std::env;

use futures::future::{Future, IntoFuture};
use url::Url;

use lobsters::client::{AuthenticatedClient, UnauthenticatedClient};

fn main() {
    let username = env::var("LOBSTERS_USER").expect("LOBSTERS_USER must be set");
    let password = env::var("LOBSTERS_PASS").expect("LOBSTERS_PASS must be set");
    let base_url: Url = lobsters::client::LOBSTERS
        .parse()
        .expect("base url is invalid");

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    let work = AuthenticatedClient::new(base_url.clone())
        .into_future()
        .or_else(|_err| {
            println!("No cookie store, logging in...");
            // TODO: Inspect err and determine if this is an authentication err
            let client = UnauthenticatedClient::new(base_url).expect("URL is not https");
            client.login(username, password)
        });
    let client = rt.block_on(work).expect("error logging in");

    client.save_cookies().expect("unable to save cookies");

    let work = client.index(None);
    let stories = rt.block_on(work).expect("error fetching stories");

    dbg!(&stories);

    client.save_cookies().expect("unable to save cookies");
}
