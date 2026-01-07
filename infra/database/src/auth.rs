use crate::error::DatabaseError;
use ed25519_dalek::SigningKey;
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

        fill(&mut seed).map_err(|e| DatabaseError::Internal {
            message: e.to_string().into(),
            context: Some("Failed to generate seed".into()),
        })?;

        let signing_key = SigningKey::from_bytes(&seed);
        let public_key_bytes = signing_key.verifying_key().to_bytes();
        let public_key_hex = hex::encode(public_key_bytes);
        let encoding_key = EncodingKey::from_ed_der(signing_key.to_bytes().as_ref());

        Ok(Self { encoding_key, public_key: public_key_hex })
    }

    pub(crate) async fn setup_database(&self, db: &Surreal<Any>) -> Result<(), DatabaseError> {
        db.query("DEFINE ACCESS OVERWRITE user ON DATABASE TYPE RECORD WITH JWT ALGORITHM EDDSA KEY $public_key;")
            .bind(("public_key", self.public_key.clone())).await?;
        Ok(())
    }
}
