//! Host-only direct SA-MP native backend.

/// SA-MP client build identity used by the direct native backend.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SampVersion {
    R1,
    R2,
    R3_1,
    R4_2,
    R5_1,
    Dl,
}

impl SampVersion {
    /// Identifies a client build from its PE optional-header entry-point RVA.
    #[must_use]
    pub const fn from_entry_point(entry_point: u32) -> Option<Self> {
        match entry_point {
            0x31DF13 => Some(Self::R1),
            0x3195DD => Some(Self::R2),
            0x0CC4D0 => Some(Self::R3_1),
            0x0CBCB0 => Some(Self::R4_2),
            0x0CBC90 => Some(Self::R5_1),
            0x0FDB60 => Some(Self::Dl),
            _ => None,
        }
    }

    /// Returns the PE optional-header entry-point RVA for this build.
    #[must_use]
    pub const fn entry_point(self) -> u32 {
        match self {
            Self::R1 => 0x31DF13,
            Self::R2 => 0x3195DD,
            Self::R3_1 => 0x0CC4D0,
            Self::R4_2 => 0x0CBCB0,
            Self::R5_1 => 0x0CBC90,
            Self::Dl => 0x0FDB60,
        }
    }
}

pub mod colours;
mod connection;
pub mod hooks;
pub mod memory;
mod players;
mod pools;
mod singletons;
mod text_labels;
mod textdraws;
mod ui;

mod error;
mod requests;
mod snapshots;

pub mod profile;
pub mod profiles;

pub use error::DirectClientError;
pub use profile::NativeProfile;
pub use requests::{
    LocalChatMessageRequest, LocalChatMessageStyle, LocalDeathMessageRequest, LocalDialogRequest,
    LocalDialogStyle,
};
pub use snapshots::{
    AimSyncSnapshot, AnimationSnapshot, ChatEntrySnapshot, GangzoneSnapshot, InCarSyncSnapshot,
    LocalDialogResponseSnapshot, LocalDialogSnapshot, LocalPlayerSnapshot, OnFootSyncSnapshot,
    PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot, ServerInfoSnapshot,
    TextLabelSnapshot, TextdrawSnapshot, TrailerSyncSnapshot, Vector3,
};
