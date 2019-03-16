use std::env;

use futures::future::{Future, IntoFuture};
use url::Url;

use lobsters::client::Client;
use lobsters::models::StoryId;

fn main() {
    let username = env::var("LOBSTERS_USER").expect("LOBSTERS_USER must be set");
    let password = env::var("LOBSTERS_PASS").expect("LOBSTERS_PASS must be set");
    let base_url: Url = lobsters::client::LOBSTERS
        .parse()
        .expect("base url is invalid");

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    let client = Client::new(base_url.clone()).expect("error creating client");

    // let work = client.index(None);
    // let stories = rt.block_on(work).expect("error fetching stories");
    // dbg!(&stories);

    // https://lobste.rs/s/yon0hz/rebuilding_my_personal_infrastructure
    let story_id = StoryId("yon0hz".to_string());
    let work = client.story(&story_id);
    let story = rt.block_on(work).expect("error fetching story");
    dbg!(&story);

    client.save_cookies().expect("unable to save cookies");
}
