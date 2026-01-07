use crate::error::{DatabaseError, DatabaseErrorExt};
use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use ed25519_dalek::pkcs8::spki::der::zeroize::Zeroize;
use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};
use getrandom::fill;
use jsonwebtoken::EncodingKey;
use serde::Serialize;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

#[derive(Debug, Serialize)]
pub(crate) struct Claims<'a> {
    pub ns: &'a str,
    pub db: &'a str,
    pub ac: &'static str,
    pub id: String,
    pub exp: i64,
}

#[derive(Debug)]
pub(crate) struct AuthProvider {
    pub encoding_key: EncodingKey,
    pub public_key: String,
}

impl AuthProvider {
    pub(crate) fn init() -> Result<Self, DatabaseError> {
        let mut seed = [0u8; 32];

        fill(&mut seed).map_err(|e| {
            DatabaseError::internal()
                .with_message("Failed to generate seed")
                .with_context_fn(|| e.to_string())
        })?;

        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();

        let public_key = verifying_key.to_public_key_pem(LineEnding::LF).map_err(|e| {
            DatabaseError::internal()
                .with_message("Failed to encode public key")
                .with_context_fn(|| e.to_string())
        })?;

        let pkcs8_der = signing_key
            .to_pkcs8_der()
            .map_err(|e| {
                DatabaseError::internal()
                    .with_message("Failed to encode private key")
                    .with_context_fn(|| e.to_string())
            })?
            .as_bytes()
            .to_vec();

        let encoding_key = EncodingKey::from_ed_der(&pkcs8_der);
        seed.zeroize();

        Ok(Self { encoding_key, public_key })
    }

    pub(crate) async fn setup_database(&self, db: &Surreal<Any>) -> Result<(), DatabaseError> {
        db.query("DEFINE ACCESS OVERWRITE user ON DATABASE TYPE RECORD WITH JWT ALGORITHM EDDSA KEY $public_key;")
            .bind(("public_key", self.public_key.clone()))
            .await.with_context("database access definition failed").with_help("Check your database configuration.")?
            .check().with_context("failed to setup database access")?;
        Ok(())
    }
}
