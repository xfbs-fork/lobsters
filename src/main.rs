use std::env;

use futures::future::{Future, IntoFuture};
use url::Url;

use lobsters::client::Client;

fn main() {
    let username = env::var("LOBSTERS_USER").expect("LOBSTERS_USER must be set");
    let password = env::var("LOBSTERS_PASS").expect("LOBSTERS_PASS must be set");
    let base_url: Url = lobsters::client::LOBSTERS
        .parse()
        .expect("base url is invalid");

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    let client = Client::new(base_url.clone()).expect("error creating client");
    let work = client.index(None);
    let stories = rt.block_on(work).expect("error fetching stories");

    dbg!(&stories);

    client.save_cookies().expect("unable to save cookies");
}
