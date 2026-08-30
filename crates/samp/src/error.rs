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
