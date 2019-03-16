use std::env;

use futures::future::{Future, IntoFuture};

use lobsters::url::Url;
use lobsters::client::Client;
use lobsters::models::{NewComment, StoryId};

fn main() {
    let username = env::var("LOBSTERS_USER").expect("LOBSTERS_USER must be set");
    let password = env::var("LOBSTERS_PASS").expect("LOBSTERS_PASS must be set");
    let base_url: Url = lobsters::URL.parse().expect("base url is invalid");
    let localhost: Url = "http://localhost:3000/"
        .parse()
        .expect("base url is invalid");

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    let client = Client::new(localhost.clone()).expect("error creating client");

    // let work = client.index(None);
    // let stories = rt.block_on(work).expect("error fetching stories");
    // dbg!(&stories);

    // https://lobste.rs/s/yon0hz/rebuilding_my_personal_infrastructure
    let story_id = StoryId("d6qvya".to_string());
    let work = client.story(&story_id);
    let story = rt.block_on(work).expect("error fetching story");
    dbg!(&story);

    let comment = NewComment {
        story_id: story_id,
        comment: "Hello from Rust!".to_string(),
        hat_id: None,
        parent_comment_short_id: None,
    };

    // let login = client.login(username, password);
    let create_comment = client.post_comment(comment);
    // let work = login.and_then(|_res| create_comment);
    let comment_url = rt.block_on(create_comment).expect("error posting comment");

    if let Some(comment_url) = comment_url {
        println!("Comment posted: {}", comment_url);
    }

    client.save_cookies().expect("unable to save cookies");
}
