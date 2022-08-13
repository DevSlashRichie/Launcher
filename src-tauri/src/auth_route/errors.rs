use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Failed OAuth {0}")]
    OAuthError(String),
    #[error("Invalid redirect URI")]
    InvalidRedirectUri,
}