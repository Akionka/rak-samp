use modkit_sdk::{ConnectError as HostConnectError, ServiceError};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectError {
    Host(HostConnectError),
    Service(ServiceError),
}

impl fmt::Display for ConnectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(error) => write!(formatter, "host connection failed: {error}"),
            Self::Service(error) => write!(formatter, "SA-MP service resolution failed: {error:?}"),
        }
    }
}

impl std::error::Error for ConnectError {}

/// A synchronous failure while encoding or submitting a Protocol descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolSendError {
    /// Protocol encoding or framing failed before Host submission.
    Encode(samp_protocol::EncodeError<samp_protocol::BitStreamError>),
    /// The Host rejected immediate submission or bounded queue insertion.
    Host(modkit_abi::ModResult),
}

impl fmt::Display for ProtocolSendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(error) => write!(formatter, "Protocol send encoding failed: {error}"),
            Self::Host(result) => write!(formatter, "Host rejected Protocol send: {result:?}"),
        }
    }
}

impl std::error::Error for ProtocolSendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::Host(_) => None,
        }
    }
}
