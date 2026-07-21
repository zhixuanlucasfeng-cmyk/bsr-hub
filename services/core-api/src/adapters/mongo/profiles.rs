use async_trait::async_trait;
use mongodb::{
    Collection,
    bson::{DateTime, doc},
    options::ReturnDocument,
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::ports::profile_repository::{
    ProfileError, ProfilePatch, ProfileRepository, ProfileRole, UserProfile,
};

use super::{bson_datetime, models::UserProfileDocument};

#[derive(Clone)]
pub struct MongoProfileRepository {
    profiles: Collection<UserProfileDocument>,
}

impl MongoProfileRepository {
    pub fn new(profiles: Collection<UserProfileDocument>) -> Self {
        Self { profiles }
    }

    fn default_display_name(auth_user_id: Uuid) -> String {
        format!("BSR member {}", &auth_user_id.to_string()[..8])
    }
}

#[async_trait]
impl ProfileRepository for MongoProfileRepository {
    async fn bootstrap(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError> {
        let now = OffsetDateTime::now_utc();
        let update = doc! {
            "$setOnInsert": {
                "_id": Uuid::new_v4().to_string(),
                "auth_user_id": auth_user_id.to_string(),
                "display_name": Self::default_display_name(auth_user_id),
                "avatar_url": null,
                "role": "buyer",
                "trust_level": 1_i32,
                "created_at": bson_datetime(now),
                "schema_version": 1_i32,
            },
            "$set": {
                "updated_at": bson_datetime(now),
            },
        };
        let document = self
            .profiles
            .find_one_and_update(doc! { "auth_user_id": auth_user_id.to_string() }, update)
            .upsert(true)
            .return_document(ReturnDocument::After)
            .await
            .map_err(|_| ProfileError::Unavailable)?
            .ok_or(ProfileError::Unavailable)?;
        profile_from_document(document)
    }

    async fn get_by_auth_user_id(&self, auth_user_id: Uuid) -> Result<UserProfile, ProfileError> {
        let document = self
            .profiles
            .find_one(doc! { "auth_user_id": auth_user_id.to_string() })
            .await
            .map_err(|_| ProfileError::Unavailable)?
            .ok_or(ProfileError::NotFound)?;
        profile_from_document(document)
    }

    async fn update(
        &self,
        auth_user_id: Uuid,
        patch: ProfilePatch,
    ) -> Result<UserProfile, ProfileError> {
        let mut set = doc! { "updated_at": bson_datetime(OffsetDateTime::now_utc()) };
        if let Some(display_name) = patch.display_name {
            let trimmed = display_name.trim();
            if trimmed.is_empty() || trimmed.chars().count() > 80 {
                return Err(ProfileError::Invalid);
            }
            set.insert("display_name", trimmed);
        }
        if let Some(avatar_url) = patch.avatar_url {
            set.insert("avatar_url", avatar_url);
        }
        let document = self
            .profiles
            .find_one_and_update(
                doc! { "auth_user_id": auth_user_id.to_string() },
                doc! { "$set": set },
            )
            .return_document(ReturnDocument::After)
            .await
            .map_err(|_| ProfileError::Unavailable)?
            .ok_or(ProfileError::NotFound)?;
        profile_from_document(document)
    }
}

fn profile_from_document(document: UserProfileDocument) -> Result<UserProfile, ProfileError> {
    Ok(UserProfile {
        id: Uuid::parse_str(&document.id).map_err(|_| ProfileError::Invalid)?,
        auth_user_id: Uuid::parse_str(&document.auth_user_id).map_err(|_| ProfileError::Invalid)?,
        display_name: document.display_name,
        avatar_url: document.avatar_url,
        role: role_from_str(&document.role)?,
        trust_level: document.trust_level,
        created_at: date_time(document.created_at)?,
        updated_at: date_time(document.updated_at)?,
    })
}

fn role_from_str(value: &str) -> Result<ProfileRole, ProfileError> {
    match value {
        "buyer" => Ok(ProfileRole::Buyer),
        "seller" => Ok(ProfileRole::Seller),
        "runner" => Ok(ProfileRole::Runner),
        "admin" => Ok(ProfileRole::Admin),
        _ => Err(ProfileError::Invalid),
    }
}

fn date_time(value: DateTime) -> Result<OffsetDateTime, ProfileError> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(value.timestamp_millis()) * 1_000_000)
        .map_err(|_| ProfileError::Invalid)
}
