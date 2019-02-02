use serde::{Deserialize, Serialize};

use crate::channel::ChannelId;
use crate::file::FileId;
use crate::user::UserId;

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct PostId(String);

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePost {
    pub channel_id: ChannelId,
    pub message: String,
    pub root_id: Option<PostId>,
    pub file_ids: Option<Vec<FileId>>,
    // props
}

#[derive(Deserialize, Debug)]
pub struct PostCollection {
    pub order: Vec<PostId>,
    pub posts: HashMap<PostId, Post>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub id: PostId,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64, // TODO: map 0 -> None
    pub edit_at: i64,   // TODO: map 0 -> None
    pub user_id: UserId,
    pub channel_id: ChannelId,
    pub root_id: PostId,
    pub parent_id: PostId,
    pub original_id: PostId, // TODO: map "" -> None
    pub message: String,
    #[serde(rename = "type")]
    pub type_: String,
    // props: object,
    pub hashtags: String,
    pub pending_post_id: PostId,
}
