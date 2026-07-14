use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub user_id: Uuid,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("authentication required")]
    Missing,
    #[error("invalid or expired access token")]
    Invalid,
    #[error("authentication service unavailable")]
    Unavailable,
}

#[async_trait]
pub trait AuthVerifier: Send + Sync {
    async fn verify(&self, bearer_token: &str) -> Result<AuthUser, AuthError>;
}

#[derive(Debug, Clone)]
pub struct SupabaseAuthVerifier {
    client: reqwest::Client,
    user_endpoint: String,
    anon_key: String,
}

impl SupabaseAuthVerifier {
    pub fn new(supabase_url: &str, anon_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            user_endpoint: format!("{}/auth/v1/user", supabase_url.trim_end_matches('/')),
            anon_key,
        }
    }
}

#[derive(Deserialize)]
struct SupabaseUser {
    id: Uuid,
}

#[async_trait]
impl AuthVerifier for SupabaseAuthVerifier {
    async fn verify(&self, bearer_token: &str) -> Result<AuthUser, AuthError> {
        if bearer_token.is_empty() {
            return Err(AuthError::Missing);
        }
        let response = self
            .client
            .get(&self.user_endpoint)
            .header("apikey", &self.anon_key)
            .bearer_auth(bearer_token)
            .send()
            .await
            .map_err(|_| AuthError::Unavailable)?;
        match response.status() {
            StatusCode::OK => response
                .json::<SupabaseUser>()
                .await
                .map(|user| AuthUser { user_id: user.id })
                .map_err(|_| AuthError::Unavailable),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(AuthError::Invalid),
            _ => Err(AuthError::Unavailable),
        }
    }
}
