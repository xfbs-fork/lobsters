use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct User {
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
pub struct StoryId(pub String);

#[derive(Debug, Deserialize)]
pub struct Story {
    pub short_id: StoryId,
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
    pub submitter_user: User,
    pub tags: Vec<Tag>,
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Deserialize)]
pub struct CommentId(pub String);

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub short_id: CommentId,
    pub short_id_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_deleted: bool,
    pub is_moderated: bool,
    pub score: i32,
    pub upvotes: u32,
    pub downvotes: u32,
    pub comment: String,
    pub url: String,
    pub indent_level: u32,
    pub commenting_user: User,
}

#[derive(Debug, Deserialize)]
pub struct Tag(pub String);
