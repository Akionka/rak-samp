//! Profile-selected probe configuration.

use samp_client_sdk::SampClientSdkClientVersion;
use std::time::Duration;

#[cfg(feature = "r1-probe")]
macro_rules! profile_value {
    ($r5:expr, $r1:expr, $r3:expr, $dl:expr) => {
        $r1
    };
}

#[cfg(feature = "r3-probe")]
macro_rules! profile_value {
    ($r5:expr, $r1:expr, $r3:expr, $dl:expr) => {
        $r3
    };
}

#[cfg(feature = "dl-probe")]
macro_rules! profile_value {
    ($r5:expr, $r1:expr, $r3:expr, $dl:expr) => {
        $dl
    };
}

#[cfg(not(any(feature = "r1-probe", feature = "r3-probe", feature = "dl-probe")))]
macro_rules! profile_value {
    ($r5:expr, $r1:expr, $r3:expr, $dl:expr) => {
        $r5
    };
}

pub(super) const OUTBOUND_MARKER: &[u8] = profile_value!(
    b"R5_SDK_OUTBOUND_20260812",
    b"R1_SDK_OUTBOUND_20260816",
    b"R3_SDK_OUTBOUND_20260812",
    b"DL_SDK_OUTBOUND_20260812"
);
pub(super) const INCOMING_MARKER: &[u8] = profile_value!(
    b"R5_SDK_INCOMING_20260812",
    b"R1_SDK_INCOMING_20260816",
    b"R3_SDK_INCOMING_20260812",
    b"DL_SDK_INCOMING_20260812"
);
pub(super) const DIALOG_REQUEST_MARKER: &[u8] = profile_value!(
    b"R5_SDK_DIALOG_REQUEST_20260812",
    b"R1_SDK_DIALOG_REQUEST_20260816",
    b"R3_SDK_DIALOG_REQUEST_20260812",
    b"DL_SDK_DIALOG_REQUEST_20260812"
);
pub(super) const ENTITY_REQUEST_MARKER: &[u8] = profile_value!(
    b"R5_SDK_ENTITY_REQUEST_20260813",
    b"R1_SDK_ENTITY_REQUEST_20260816",
    b"R3_SDK_ENTITY_REQUEST_20260813",
    b"DL_SDK_ENTITY_REQUEST_20260813"
);
pub(super) const ENTITY_IDS_PREFIX: &[u8] = profile_value!(
    b"R5_SDK_ENTITY_IDS_",
    b"R1_SDK_ENTITY_IDS_",
    b"R3_SDK_ENTITY_IDS_",
    b"DL_SDK_ENTITY_IDS_"
);
pub(super) const CHAT_INPUT_TEXT_MARKER: &[u8] = profile_value!(
    b"R5_SDK_TEXT_CACHE_20260812",
    b"R1_SDK_TEXT_CACHE_20260816",
    b"R3_SDK_TEXT_CACHE_20260812",
    b"DL_SDK_TEXT_CACHE_20260812"
);
pub(super) const LOCAL_CHAT_MARKER: &[u8] = profile_value!(
    b"R5 SDK full UI validation",
    b"R1 SDK full UI validation",
    b"R3 SDK full UI validation",
    b"DL SDK full UI validation"
);
pub(super) const LOCAL_CHAT_PREFIX: &[u8] =
    profile_value!(b"R5 SDK", b"R1 SDK", b"R3 SDK", b"DL SDK");
pub(super) const LOCAL_COMMAND_NAME: &[u8] =
    profile_value!(b"r5sdkprobe", b"r1sdkprobe", b"r3sdkprobe", b"dlsdkprobe");
pub(super) const MISSING_COMMAND_NAME: &[u8] = profile_value!(
    b"r5_sdk_probe_missing_command",
    b"r1_sdk_probe_missing_command",
    b"r3_sdk_probe_missing_command",
    b"dl_sdk_probe_missing_command"
);
pub(super) const LOCAL_COMMAND_TEXT: &[u8] = profile_value!(
    b"/r5sdkprobe consolidated",
    b"/r1sdkprobe consolidated",
    b"/r3sdkprobe consolidated",
    b"/dlsdkprobe consolidated"
);
pub(super) const LOCAL_DIALOG_INPUT_TEXT: &[u8] = profile_value!(
    b"R5_INPUT_UPDATED",
    b"R1_INPUT_UPDATED",
    b"R3_INPUT_UPDATED",
    b"DL_INPUT_UPDATED"
);
pub(super) const LOCAL_CHAT_INPUT_MUTATION: &[u8] = profile_value!(
    b"R5_SDK_MUTATION",
    b"R1_SDK_MUTATION",
    b"R3_SDK_MUTATION",
    b"DL_SDK_MUTATION"
);
pub(super) const LOCAL_INPUT_DIALOG_TITLE: &[u8] = profile_value!(
    b"R5 input dialog",
    b"R1 input dialog",
    b"R3 input dialog",
    b"DL input dialog"
);
pub(super) const LOCAL_INPUT_DIALOG_BODY: &[u8] = profile_value!(
    b"R5 input body",
    b"R1 input body",
    b"R3 input body",
    b"DL input body"
);
pub(super) const LOCAL_LIST_DIALOG_TITLE: &[u8] = profile_value!(
    b"R5 list dialog",
    b"R1 list dialog",
    b"R3 list dialog",
    b"DL list dialog"
);
pub(super) const LOCAL_LABEL_TEXT: &[u8] = profile_value!(
    b"R5 label validation",
    b"R1 label validation",
    b"R3 label validation",
    b"DL label validation"
);
pub(super) const LOCAL_LABEL_UPDATED_TEXT: &[u8] = profile_value!(
    b"R5 label updated",
    b"R1 label updated",
    b"R3 label updated",
    b"DL label updated"
);
pub(super) const LOCAL_TEXTDRAW_TEXT: &[u8] = profile_value!(
    b"R5 textdraw validation",
    b"R1 textdraw validation",
    b"R3 textdraw validation",
    b"DL textdraw validation"
);
pub(super) const LOCAL_TEXTDRAW_UPDATED_TEXT: &[u8] = profile_value!(
    b"R5 textdraw updated",
    b"R1 textdraw updated",
    b"R3 textdraw updated",
    b"DL textdraw updated"
);
pub(super) const LOCAL_DRIVER_REQUEST: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_DRIVER_REQUEST",
    b"R1_SDK_LOCAL_DRIVER_REQUEST",
    b"R3_SDK_LOCAL_DRIVER_REQUEST",
    b"DL_SDK_LOCAL_DRIVER_REQUEST"
);
pub(super) const LOCAL_PASSENGER_REQUEST: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_PASSENGER_REQUEST",
    b"R1_SDK_LOCAL_PASSENGER_REQUEST",
    b"R3_SDK_LOCAL_PASSENGER_REQUEST",
    b"DL_SDK_LOCAL_PASSENGER_REQUEST"
);
pub(super) const LOCAL_TRAILER_REQUEST: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_TRAILER_REQUEST",
    b"R1_SDK_LOCAL_TRAILER_REQUEST",
    b"R3_SDK_LOCAL_TRAILER_REQUEST",
    b"DL_SDK_LOCAL_TRAILER_REQUEST"
);
pub(super) const VEHICLE_CLEANUP_REQUEST: &[u8] = profile_value!(
    b"R5_SDK_VEHICLE_CLEANUP",
    b"R1_SDK_VEHICLE_CLEANUP",
    b"R3_SDK_VEHICLE_CLEANUP",
    b"DL_SDK_VEHICLE_CLEANUP"
);
pub(super) const RECONNECT_COMMAND_NAME: &[u8] = profile_value!(
    b"r5sdkreconnect",
    b"r1sdkreconnect",
    b"r3sdkreconnect",
    b"dlsdkreconnect"
);
pub(super) const MAIN_PASS_MESSAGE: &[u8] = profile_value!(
    b"R5 main pass complete. Type /r5sdkreconnect for the final lifecycle pass.",
    b"R1 main pass complete. Type /r1sdkreconnect for the final lifecycle pass.",
    b"R3 main pass complete. Type /r3sdkreconnect for the final lifecycle pass.",
    b"DL main pass complete. Type /dlsdkreconnect for the final lifecycle pass."
);
pub(super) const HOST_CONNECTION_TIMEOUT: Duration = Duration::from_secs(5 * 60);
pub(super) const INITIALIZATION_TIMEOUT: Duration = Duration::from_secs(45);
pub(super) const SCALAR_CACHE_TIMEOUT: Duration = Duration::from_secs(45);
pub(super) const CHAT_INPUT_CACHE_TIMEOUT: Duration = Duration::from_secs(60);
pub(super) const SCOREBOARD_CACHE_TIMEOUT: Duration = Duration::from_secs(60);
pub(super) const DIALOG_ACTIVE_CACHE_TIMEOUT: Duration = Duration::from_secs(15);
pub(super) const INCOMING_READY_TIMEOUT: Duration = Duration::from_secs(60);
pub(super) const CALLBACK_TIMEOUT: Duration = Duration::from_secs(15);
pub(super) const RETRY_DELAY: Duration = Duration::from_millis(100);
pub(super) const STATUS_FILE: &str = profile_value!(
    "samp-client-sdk-r5-network-probe.status",
    "samp-client-sdk-r1-network-probe.status",
    "samp-client-sdk-r3-network-probe.status",
    "samp-client-sdk-dl-network-probe.status"
);
pub(super) const PROFILE_ENTRY_POINT_RVA: u32 =
    profile_value!(0x0CBC90, 0x31DF13, 0x0CC4D0, 0x0FDB60);
/// State observed after the first incoming RPC, before the local player is spawned.
pub(super) const PROFILE_INITIAL_GAME_STATE: i32 = profile_value!(15, 14, 15, 5);
/// State required after a reconnect has spawned the local player.
pub(super) const PROFILE_CONNECTED_STATE: i32 = profile_value!(14, 14, 14, 5);
pub(super) const PROFILE_SERVER_HOSTNAME: &[u8] = profile_value!(
    b"SA-MP",
    b"SDK R1 loopback probe",
    b"SA-MP",
    b"SDK DL loopback probe"
);
pub(super) const PROFILE_CLIENT_VERSION: SampClientSdkClientVersion = profile_value!(
    SampClientSdkClientVersion::R5_1,
    SampClientSdkClientVersion::R1,
    SampClientSdkClientVersion::R3_1,
    SampClientSdkClientVersion::Dl
);
pub(super) const LOCAL_DRIVER_READY_PREFIX: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_DRIVER_READY_",
    b"R1_SDK_LOCAL_DRIVER_READY_",
    b"R3_SDK_LOCAL_DRIVER_READY_",
    b"DL_SDK_LOCAL_DRIVER_READY_"
);
pub(super) const LOCAL_PASSENGER_READY_PREFIX: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_PASSENGER_READY_",
    b"R1_SDK_LOCAL_PASSENGER_READY_",
    b"R3_SDK_LOCAL_PASSENGER_READY_",
    b"DL_SDK_LOCAL_PASSENGER_READY_"
);
pub(super) const LOCAL_TRAILER_READY_PREFIX: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_TRAILER_READY_",
    b"R1_SDK_LOCAL_TRAILER_READY_",
    b"R3_SDK_LOCAL_TRAILER_READY_",
    b"DL_SDK_LOCAL_TRAILER_READY_"
);
pub(super) const VEHICLE_CLEANUP_READY_MARKER: &[u8] = profile_value!(
    b"R5_SDK_VEHICLE_CLEANUP_READY",
    b"R1_SDK_VEHICLE_CLEANUP_READY",
    b"R3_SDK_VEHICLE_CLEANUP_READY",
    b"DL_SDK_VEHICLE_CLEANUP_READY"
);
pub(super) const MAX_PE_HEADER_OFFSET: usize = 0x1000;
