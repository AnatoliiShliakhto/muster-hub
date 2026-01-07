//! Core licensing models

use mhub_domain::Features;
use serde::{Deserialize, Serialize};

/// Signed license container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedLicense {
    pub data: LicenseData,
    #[serde(with = "bytes_as_base64")]
    pub signature: Vec<u8>,
}

/// License data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseData {
    pub customer: String,
    pub constraint: MachineConstraint,
    pub features: Features,
    #[serde(with = "bytes_as_base64")]
    pub salt: Vec<u8>,
    pub expires_at: i64,
}

/// Machine binding constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MachineConstraint {
    /// Any machine
    Any,
    /// Specific machine IDs
    MachineIds(Vec<String>),
}

mod bytes_as_base64 {
    use base64::{Engine as _, engine::general_purpose};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub(super) fn serialize<S: Serializer>(
        v: &Vec<u8>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        let mut buf = String::with_capacity((v.len() * 4).div_ceil(3));
        general_purpose::STANDARD_NO_PAD.encode_string(v, &mut buf);
        String::serialize(&buf, s)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Vec<u8>, D::Error> {
        match general_purpose::STANDARD_NO_PAD.decode(String::deserialize(d)?) {
            Ok(bytes) => Ok(bytes),
            Err(e) => {
                Err(serde::de::Error::custom(format!("Invalid Base64: {e}")))
            },
        }
    }
}
