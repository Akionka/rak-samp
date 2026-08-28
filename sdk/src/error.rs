use crate::SampClientSdkResult;
use samp_protocol::{BitStreamError, EncodeError};
use std::fmt;

/// A synchronous failure while encoding or submitting a Protocol descriptor send.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolSendError {
    /// Protocol encoding or descriptor framing failed before Host submission.
    Encode(EncodeError<BitStreamError>),
    /// The Host rejected immediate submission or queued enqueueing.
    ///
    /// For a queued send, this variant does not report later asynchronous transport execution.
    Host(SampClientSdkResult),
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
