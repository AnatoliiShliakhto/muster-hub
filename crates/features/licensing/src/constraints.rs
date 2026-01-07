//! # Machine Fingerprints (Compound Machine ID)
//!
//! ## Why "compound" IDs?
//!
//! A single machine fingerprint can be too strict: a minor hardware change (e.g., network adapter)
//! may invalidate an otherwise legitimate license. To support **fuzzy matching**, we derive multiple
//! independent fingerprints and require a **threshold** of matches.
//!
//! This implementation generates **three** component fingerprints:
//!
//! - **CPU ID** (`CPUID`)
//! - **MAC Address** (`MacAddress`)
//! - **System ID** (`SystemID`)
//!
//! Each component fingerprint is derived deterministically using `machineid_rs` with `SHA256`,
//! salted by a constant `KEY`.
//!
//! ## Data Format (Single String Encoding)
//!
//! The three fingerprints are packed into a single string called a **compound machine id**.
//!
//! Format (versioned):
//!
//! ```text
//! v1:<cpuid>|<mac>|<system_id>
//! ```
//!
//! - The `v1:` prefix allows future format upgrades (`v2:...`) without breaking old licenses.
//! - `|` is used as a separator because it does not appear in typical hex/base16 digests.
//!
//! ## Parsing
//!
//! Use [`parse_machine_id_compound`] to convert a compound id back into the 3 part ids.
//! This is intended for:
//!
//! - Verifying a license on the client/server,
//! - Computing the number of component matches for fuzzy hardware binding.
//!
//! ## How to use with `MachineConstraint::Threshold`
//!
//! Recommended meaning for `MachineConstraint::Threshold { ids, min_matches }`:
//!
//! - `ids`: a list of **allowed compound machine ids**, i.e., each entry corresponds to a single
//!   machine the license is bound to. (This supports multi-machine licenses.)
//! - `min_matches`: the minimum number of matching **components** required to accept the license
//!   for a given allowed machine.
//!
//! Verification strategy:
//!
//! 1. Compute current machine components: `current_machine_components()` -> `[CpuId, Mac, System]`
//! 2. For each allowed compound ID in `ids`:
//!    - parse it into 3 parts.
//!    - compute intersection count with current components.
//!    - Accept if `matches >= min_matches`.
//!
//! Example thresholds:
//!
//! - `min_matches = 1`: very permissive (any one-component match)
//! - `min_matches = 2`: recommended default (survives one component change)
//! - `min_matches = 3`: strict binding (all components must match)
//!
//! ## Security Notes
//!
//! - These identifiers are **not secrets**, but they are **identifiers**. Treat them as personal/device
//!   data and avoid logging them at info level.
//! - Salting (`KEY`) prevents trivial rainbow-table precomputation for the raw hardware values
//!   but does not make the identifier confidential.
//! - Machine identifiers can be spoofed on some platforms. For high-security enforcement, combine
//!   this with server-side checks and additional device attestation.
//!
//! ## Platform Considerations
//!
//! The availability and stability of components depends on OS and environment:
//!
//! - Virtual machines and containers may expose unstable or synthetic IDs.
//! - Some systems may have multiple MACs; `machineid_rs` selects one by its own rules.
//! - Permissions may restrict reading certain IDs.
//!
//! Always decide `min_matches` based on your support matrix and acceptable false rejects.
//!

use crate::error::LicenseError;
use machineid_rs::{Encryption, HWIDComponent, IdBuilder};

/// Constant salt used to derive deterministic machine fingerprints.
///
/// This value must remain stable across releases, otherwise all issued licenses
/// bound to machine IDs will become invalid.
///
/// Do **not** treat this as a secret key; it is a salt to reduce trivial precomputation.
/// If you need confidentiality, do not expose machine identifiers.
const KEY: &str = "muster-hub-license";

/// Separator used in the compound machine id encoding.
///
/// Chosen to avoid collisions with common digest encodings (hex/base16).
const SEP: char = '|';

/// Encoding prefix to support forward-compatible format upgrades.
const PREFIX: &str = "v1:";

/// Derives a deterministic fingerprint for a single hardware component.
///
/// This uses `machineid_rs` in SHA256 mode with the constant `KEY` salt.
///
/// # Arguments
/// - `component`: which hardware component to fingerprint.
///
/// # Returns
/// A deterministic component identifier string.
///
/// # Errors
/// Returns [`LicenseError::MachineIDGeneration`] if the underlying hardware component
/// cannot be read or if `machineid_rs` fails to build the id.
///
/// # Notes
/// This function returns a fingerprint, not the raw hardware value.
fn build_component(component: HWIDComponent) -> Result<String, LicenseError> {
    IdBuilder::new(Encryption::SHA256).add_component(component).build(KEY).map_err(|e| {
        LicenseError::MachineIDGeneration {
            message: e.to_string().into(),
            context: Some("machineid_rs build failed".into()),
        }
    })
}

/// Generates the compound machine id as a single versioned string.
///
/// Format:
///
/// ```text
/// v1:<cpuid>|<mac>|<system_id>
/// ```
///
/// # Returns
/// A single string containing three component fingerprints.
///
/// # Errors
/// Returns [`LicenseError::MachineIDGeneration`] if any of the three hardware
/// component fingerprints cannot be derived.
///
/// # When to use
/// - Storing machine binding in license files as a single string.
/// - Transmitting a stable identifier to a licensing service.
///
/// # Privacy
/// Avoid logging the returned value in plaintext.
pub fn generate_machine_id_compound() -> Result<String, LicenseError> {
    let cpuid = build_component(HWIDComponent::CPUID)?;
    let mac = build_component(HWIDComponent::MacAddress)?;
    let system_id = build_component(HWIDComponent::SystemID)?;
    Ok(format!("{PREFIX}{cpuid}{SEP}{mac}{SEP}{system_id}"))
}

/// Parses a compound machine id into its component fingerprints.
///
/// Expects a format:
///
/// ```text
/// v1:<cpuid>|<mac>|<system_id>
/// ```
///
/// # Arguments
/// - `s`: compound machine ID string.
///
/// # Returns
/// A vector of exactly three strings: `[CpuId, Mac, SystemId]`.
///
/// # Errors
/// Returns [`LicenseError::MachineIDGeneration`] if the prefix is invalid, the
/// separator count is wrong, or any component is empty.
///
/// # Forward compatibility
/// The parser is strict for `v1:`. If you introduce `v2:`, add a separate parser
/// and dispatch by prefix.
pub fn parse_machine_id_compound(s: &str) -> Result<Vec<String>, LicenseError> {
    let s = s.strip_prefix(PREFIX).ok_or_else(|| LicenseError::MachineIDGeneration {
        message: "Invalid machine id prefix".into(),
        context: Some("Expected v1: prefix".into()),
    })?;

    let parts: Vec<&str> = s.split(SEP).collect();
    if parts.len() != 3 || parts.iter().any(|p| p.is_empty()) {
        return Err(LicenseError::MachineIDGeneration {
            message: "Invalid compound machine id format".into(),
            context: Some("Expected 3 parts separated by '|'".into()),
        });
    }

    Ok(parts.into_iter().map(str::to_owned).collect())
}

/// Returns the current machine's component identifiers.
///
/// This is a convenience helper used during license validation to compute fuzzy matches.
///
/// # Returns
/// A vector of exactly three strings: `[cpuid, mac, system_id]`.
///
/// # Errors
/// Returns [`LicenseError::MachineIDGeneration`] if the compound id cannot be generated
/// or parsed.
pub fn current_machine_components() -> Result<Vec<String>, LicenseError> {
    parse_machine_id_compound(&generate_machine_id_compound()?)
}
