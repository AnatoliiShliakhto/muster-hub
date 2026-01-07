use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Keyset {
    pub master_key: [u8; 32],
    pub public_key: [u8; 32],
}
