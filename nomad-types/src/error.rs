use crate::NomadIdentifier;

/// DB Error type
#[derive(thiserror::Error, Debug)]
pub enum NomadTypeError {
    /// Failed to perform conversion to 20 byte address
    #[error("Failed to convert 32 byte address into 20 byte address: {0}")]
    AddressConversionError(NomadIdentifier),
}
