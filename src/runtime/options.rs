use super::errors::SendError;

/// Reliability priority used by [`SendOptions`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketPriority {
    System,
    High,
    Medium,
    Low,
}

/// Delivery behavior used by [`SendOptions`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketReliability {
    Unreliable,
    UnreliableSequenced,
    Reliable,
    ReliableOrdered,
    ReliableSequenced,
}

/// RakNet delivery options for raw packet and RPC sends.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendOptions {
    pub priority: PacketPriority,
    pub reliability: PacketReliability,
    pub ordering_channel: u8,
    pub timestamp: bool,
}

impl Default for SendOptions {
    fn default() -> Self {
        Self {
            priority: PacketPriority::High,
            reliability: PacketReliability::ReliableOrdered,
            ordering_channel: 0,
            timestamp: false,
        }
    }
}

pub(super) fn validate_packet_options(options: SendOptions) -> Result<(), SendError> {
    if options.timestamp {
        Err(SendError::TimestampedPacketUnsupported)
    } else {
        Ok(())
    }
}
