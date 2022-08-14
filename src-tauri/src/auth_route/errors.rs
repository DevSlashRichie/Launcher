use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {

    #[error("Failed OAuth: {0}")]
    OAuthError(String),

    #[error("Failed XBL: {0}")]
    XBLError(String),

    #[error("Failed XSTS: {0}")]
    XSTSError(String),

    #[error("Failed Minecraft Token: {0}")]
    MinecraftTokenError(String),

    #[error("Invalid redirect URI")]
    InvalidRedirectUri,

    #[error("State payload mismatch")]
    InvalidState,

}