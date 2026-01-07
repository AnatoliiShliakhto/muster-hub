use crate::{LicenseError, Result};
use machineid_rs::{Encryption, HWIDComponent, IdBuilder};

const KEY: &str = "muster-hub-license";

/// Generates a unique, deterministic machine identifier.
///
/// This function creates a stable hardware fingerprint by combining multiple
/// hardware components (CPU ID, MAC address, and System ID) and hashing them
/// using SHA256. The resulting identifier is suitable for machine-binding
/// license constraints.
///
/// # Hardware Components
/// The machine ID is derived from:
/// * **CPU ID**: Processor-specific identifier
/// * **MAC Address**: Primary network interface hardware address
/// * **System ID**: Operating system or motherboard identifier
///
/// # Returns
/// `Ok(String)` containing the hexadecimal SHA256 hash of the combined
/// hardware components.
///
/// # Errors
/// Returns [`LicenseError::MachineIDGenerationFailed`] if:
/// * Hardware component reading fails (e.g., insufficient permissions)
/// * The underlying [`IdBuilder`] encounters an error during ID generation
/// * The hashing operation fails
///
/// # Security Considerations
/// * The hardware components are hashed with a constant salt (`KEY`) to prevent
///   rainbow table attacks.
/// * The resulting ID should be transmitted securely (e.g., over TLS) when
///   communicating with license servers.
///
/// # Platform Support
/// Availability of hardware components depends on the operating system:
/// * **Windows**: All components typically available
/// * **Linux**: May require elevated privileges for some components
/// * **macOS**: System ID may have limited availability
///
/// # Examples
/// ```
/// use mhub_licensing::Result;
/// use mhub_licensing::constraints::generate_machine_id;
///
/// fn bind_license_to_machine() -> Result<String> {
///     let machine_id = generate_machine_id()?;
///     println!("Machine ID: {}", machine_id);
///     Ok(machine_id)
/// }
/// ```
///
/// # See Also
/// * [`MachineConstraint`] Enum that uses machine IDs for license binding
/// * [`validate_license`] Function that validates machine-bound licenses
pub fn generate_machine_id() -> Result<String> {
    IdBuilder::new(Encryption::SHA256)
        .add_component(HWIDComponent::CPUID)
        .add_component(HWIDComponent::MacAddress)
        .add_component(HWIDComponent::SystemID)
        .build(KEY)
        .map_err(|e| LicenseError::MachineIDGeneration {
            message: e.to_string().into(),
            context: None,
        })
}
