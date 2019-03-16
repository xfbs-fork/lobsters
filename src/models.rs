use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Story {
    pub short_id: String,
    pub short_id_url: String,
    pub created_at: String,
    pub title: String,
    pub url: String,
    pub score: i32,
    pub upvotes: u32,
    pub downvotes: u32,
    pub comment_count: u32,
    pub description: String,
    pub comments_url: String,
    pub submitter_user: Submitter,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub struct Submitter {
    pub username: String,
    pub created_at: String,
    pub is_admin: bool,
    pub about: String,
    pub is_moderator: bool,
    pub karma: i32,
    pub avatar_url: String,
    pub invited_by_user: String,
    pub github_username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Tag(String);
