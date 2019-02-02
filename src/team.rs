use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamId(String);

#[derive(Serialize, Deserialize, Debug)]
pub struct Team {
    pub id: TeamId,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
    pub display_name: String,
    pub name: String,
    pub description: String,
    pub email: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub allowed_domains: String,
    pub invite_id: String,
    pub allow_open_invite: bool,
}

impl TeamId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
