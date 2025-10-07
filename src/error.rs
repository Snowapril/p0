use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum DeviceError {
    #[error("Out of memory")]
    OutOfMemory,
    #[error("Device is lost")]
    Lost,
    #[error("Unexpected error variant (driver implementation is at fault)")]
    Unexpected,
    #[error("Current device is unavailable to run this engine")]
    Unavailable(String),
}
