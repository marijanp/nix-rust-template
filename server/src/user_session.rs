use openidconnect::{
    core::{CoreGenderClaim, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm},
    EmptyAdditionalClaims, IdToken, Nonce,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserSession {
    pub id: Uuid,
    pub id_token: UserIdToken,
    pub nonce: Nonce,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub type UserIdToken = IdToken<
    EmptyAdditionalClaims,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;
