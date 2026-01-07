//! Core licensing types

use serde::{Deserialize, Serialize};

/// Signed license container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedLicense {
    pub data: LicenseData,
    pub signature: Vec<u8>,
}

/// License data payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    pub customer: String,
    pub constraint: MachineConstraint,
    pub features: Vec<String>,
    pub secret: Vec<u8>,
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