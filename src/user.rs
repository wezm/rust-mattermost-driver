use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserId(String);

#[derive(Serialize, Deserialize, Debug)]
pub enum UserParam {
    Me,
    Id(UserId),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: UserId,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub nickname: String,
    pub email: String,
    pub email_verified: bool,
    pub auth_service: String,
    pub roles: String,
    pub locale: String,
    pub notify_props: UserNotifyProps,
    //pub props: object,
    pub last_password_update: i64,
    pub last_picture_update: i64,
    pub failed_attempts: Option<i64>,
    pub mfa_active: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum NotifyOption {
    All,
    Mention,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct UserNotifyProps {
    #[serde(with = "string_boolean")]
    pub email: bool,
    pub push: NotifyOption,
    pub desktop: NotifyOption,
    #[serde(with = "string_boolean")]
    pub desktop_sound: bool,
    pub mention_keys: String,
    #[serde(with = "string_boolean")]
    pub channel: bool,
    #[serde(with = "string_boolean")]
    pub first_name: bool,
}

impl UserParam {
    pub fn as_str(&self) -> &str {
        match self {
            UserParam::Me => "me",
            UserParam::Id(UserId(ref id)) => id.as_str(),
        }
    }
}

mod string_boolean {
    use serde::{Deserialize, Deserializer, Serializer};

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    enum StringBoolean {
        True,
        False,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        match StringBoolean::deserialize(deserializer)? {
            StringBoolean::True => Ok(true),
            StringBoolean::False => Ok(false),
        }
    }

    pub fn serialize<S>(is_true: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if *is_true {
            serializer.serialize_str("true")
        } else {
            serializer.serialize_str("false")
        }
    }
}
