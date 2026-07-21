use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: Uuid,
    pub auth_user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub role: ProfileRole,
    pub trust_level: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileRole {
    Buyer,
    Seller,
    Runner,
    Admin,
}

impl ProfileRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buyer => "buyer",
            Self::Seller => "seller",
            Self::Runner => "runner",
            Self::Admin => "admin",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProfilePatch {
    pub display_name: Option<String>,
    pub avatar_url: Option<Option<String>>,
}

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("profile was not found")]
    NotFound,
    #[error("profile data is invalid")]
    Invalid,
    #[error("profile persistence failed")]
    Unavailable,
}

#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn bootstrap(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError>;

    async fn get_by_auth_user_id(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError>;

    async fn update(
        &self,
        auth_user_id: Uuid,
        patch: ProfilePatch,
    ) -> Result<UserProfile, ProfileError>;
}
