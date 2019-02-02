use serde::{Deserialize, Serialize};

use crate::team;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelId(String);

#[derive(Serialize, Deserialize, Debug)]
pub struct Channel {
    pub id: ChannelId,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
    pub team_id: team::TeamId,
    #[serde(rename = "type")]
    pub type_: String,
    pub display_name: String,
    pub name: String,
    pub header: String,
    pub purpose: String,
    pub last_post_at: i64,
    pub total_msg_count: i64,
    // extra_update_at: integer <int64>
    // Deprecated in Mattermost 5.0 release
    pub creator_id: String,
}
