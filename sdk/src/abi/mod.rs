//! Stable C ABI declarations and table layout.

use super::*;

mod control;
mod table;
mod values;

pub use control::{
    SampClientSdkChatCommandCallbackV1, SampClientSdkClientVersion, SampClientSdkCommandReceipt,
    SampClientSdkCommandResultV1, SampClientSdkDirection, SampClientSdkEncodedString,
    SampClientSdkEventCallbackV1, SampClientSdkEventV1, SampClientSdkHookAction,
    SampClientSdkHostStatus, SampClientSdkResult, SampClientSdkSendOptions,
    SampClientSdkSubscription, SampClientSdkTextLabelCreateResultV1,
};
pub use table::{SampClientSdkApiV1, SampClientSdkGetApiV1};
pub use values::{
    SampClientSdkActiveDialogV1, SampClientSdkAimSyncV1, SampClientSdkAnimationV1,
    SampClientSdkChatEntryV1, SampClientSdkChatInputTextV1, SampClientSdkDialogListItemV1,
    SampClientSdkDialogResponseV1, SampClientSdkDialogSnapshotV1, SampClientSdkGangzoneV1,
    SampClientSdkInCarSyncV1, SampClientSdkLocalPlayerV1, SampClientSdkOnFootSyncV1,
    SampClientSdkPassengerSyncV1, SampClientSdkPlayerInfoV1, SampClientSdkRemotePlayerStateV1,
    SampClientSdkServerInfoV1, SampClientSdkStreamedOutPlayerPositionV1, SampClientSdkTextDrawV1,
    SampClientSdkTextLabelV1, SampClientSdkTrailerSyncV1,
};

pub const ABI_VERSION_V1: u32 = 1;
pub const DEFAULT_HOST_MODULE: &[u8] = b"samp_client_sdk.asi\0";
