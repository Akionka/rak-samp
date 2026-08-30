use core::fmt;

/// Failures specific to the direct, profile-gated client helpers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DirectClientError {
    NotReady,
    Busy,
    UnsupportedVersion,
    QueueFull,
}

/// Failure to attach the SDK to a compatible SA-MP client.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachError {
    UnsupportedPlatform,
    SampNotLoaded,
    GameExecutableUnavailable,
    UnsupportedGame { image_base: usize, sha256: [u8; 32] },
    UnsupportedClient { entry_point: u32 },
    ClientNotReady,
    AlreadyAttached,
    HookInstallFailed(&'static str),
}

impl fmt::Display for AttachError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                formatter.write_str("samp_client_sdk requires a 32-bit Windows process")
            }
            Self::SampNotLoaded => formatter.write_str("samp.dll is not loaded"),
            Self::GameExecutableUnavailable => {
                formatter.write_str("could not identify the GTA executable")
            }
            Self::UnsupportedGame { image_base, sha256 } => {
                write!(
                    formatter,
                    "unsupported GTA executable at 0x{image_base:08X}: "
                )?;
                for byte in sha256 {
                    write!(formatter, "{byte:02X}")?;
                }
                Ok(())
            }
            Self::UnsupportedClient { entry_point } => {
                write!(
                    formatter,
                    "unsupported samp.dll entry point RVA: 0x{entry_point:X}"
                )
            }
            Self::ClientNotReady => formatter.write_str("the SA-MP RakClient is not ready yet"),
            Self::AlreadyAttached => {
                formatter.write_str("a samp_client_sdk runtime is already attached")
            }
            Self::HookInstallFailed(detail) => {
                write!(formatter, "failed to install SA-MP hook: {detail}")
            }
        }
    }
}

impl std::error::Error for AttachError {}

/// Failure to send or locally emulate network traffic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SendError {
    ClientNotReady,
    QueueFull,
    PayloadTooLarge,
    NativeCallFailed,
    TimestampedPacketUnsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CodecError {
    ClientNotReady,
    InvalidArgument,
    PayloadTooLarge,
    NativeCallFailed,
}

impl fmt::Display for SendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClientNotReady => formatter.write_str("the SA-MP client hook is not ready"),
            Self::QueueFull => formatter.write_str("the game-thread command queue is full"),
            Self::PayloadTooLarge => {
                formatter.write_str("the payload does not fit into the native bit stream")
            }
            Self::NativeCallFailed => {
                formatter.write_str("the SA-MP client rejected the network operation")
            }
            Self::TimestampedPacketUnsupported => {
                formatter.write_str("timestamped packet sends are not supported")
            }
        }
    }
}

impl std::error::Error for SendError {}
