use std::{
    sync::atomic::{AtomicBool, AtomicU8, AtomicU32},
    time::Duration,
};

pub(crate) const HOST_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
pub(crate) const ID_COUNT: usize = 256;
pub(crate) const ID_TIMESTAMP: u8 = 40;
pub(crate) const ID_STATS_UPDATE: u8 = 205;
pub(crate) const RPC_UPDATE_SCORES_AND_PINGS: u8 = 155;
pub(crate) const STATS_PAYLOAD_LEN: usize = 8;

pub(crate) type IdHistogram = [AtomicU32; ID_COUNT];

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SelfTestStatus {
    Pending,
    Rewritten,
    Passed,
    Failed,
    TimedOut,
    CallFailed,
    Disabled,
}

impl SelfTestStatus {
    pub(crate) const fn as_raw(self) -> u8 {
        self as u8
    }

    const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Pending),
            1 => Some(Self::Rewritten),
            2 => Some(Self::Passed),
            3 => Some(Self::Failed),
            4 => Some(Self::TimedOut),
            5 => Some(Self::CallFailed),
            6 => Some(Self::Disabled),
            _ => None,
        }
    }

    const fn is_finished(self) -> bool {
        matches!(
            self,
            Self::Passed | Self::Failed | Self::TimedOut | Self::CallFailed | Self::Disabled
        )
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Rewritten => "rewritten",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::TimedOut => "timed-out",
            Self::CallFailed => "call-failed",
            Self::Disabled => "disabled",
        }
    }
}

pub(crate) fn self_test_finished(status: u8) -> bool {
    match SelfTestStatus::from_raw(status) {
        Some(status) => status.is_finished(),
        None => false,
    }
}

pub(crate) fn self_test_label(status: u8) -> &'static str {
    match SelfTestStatus::from_raw(status) {
        Some(status) => status.label(),
        None => "invalid",
    }
}

pub(crate) struct Metrics {
    pub(crate) incoming_packets: AtomicU32,
    pub(crate) outgoing_packets: AtomicU32,
    pub(crate) incoming_rpcs: AtomicU32,
    pub(crate) outgoing_rpcs: AtomicU32,
    pub(crate) null_events: AtomicU32,
    pub(crate) timestamp_decode_errors: AtomicU32,
    pub(crate) player_skin_decoded: AtomicU32,
    pub(crate) typed_r1_rpc_callbacks: AtomicU32,
    pub(crate) typed_r1_packet_callbacks: AtomicU32,
    pub(crate) typed_r1_rpc_replaced: AtomicBool,
    pub(crate) typed_r1_packet_replaced: AtomicBool,
    pub(crate) incoming_packet_ids: IdHistogram,
    pub(crate) outgoing_packet_ids: IdHistogram,
    pub(crate) incoming_timestamp_inner_ids: IdHistogram,
    pub(crate) outgoing_timestamp_inner_ids: IdHistogram,
    pub(crate) incoming_rpc_ids: IdHistogram,
    pub(crate) outgoing_rpc_ids: IdHistogram,
}

impl Metrics {
    const fn new() -> Self {
        Self {
            incoming_packets: AtomicU32::new(0),
            outgoing_packets: AtomicU32::new(0),
            incoming_rpcs: AtomicU32::new(0),
            outgoing_rpcs: AtomicU32::new(0),
            null_events: AtomicU32::new(0),
            timestamp_decode_errors: AtomicU32::new(0),
            player_skin_decoded: AtomicU32::new(0),
            typed_r1_rpc_callbacks: AtomicU32::new(0),
            typed_r1_packet_callbacks: AtomicU32::new(0),
            typed_r1_rpc_replaced: AtomicBool::new(false),
            typed_r1_packet_replaced: AtomicBool::new(false),
            incoming_packet_ids: [const { AtomicU32::new(0) }; ID_COUNT],
            outgoing_packet_ids: [const { AtomicU32::new(0) }; ID_COUNT],
            incoming_timestamp_inner_ids: [const { AtomicU32::new(0) }; ID_COUNT],
            outgoing_timestamp_inner_ids: [const { AtomicU32::new(0) }; ID_COUNT],
            incoming_rpc_ids: [const { AtomicU32::new(0) }; ID_COUNT],
            outgoing_rpc_ids: [const { AtomicU32::new(0) }; ID_COUNT],
        }
    }
}

pub(crate) struct SelfTests {
    pub(crate) packet: AtomicU8,
    pub(crate) rpc: AtomicU8,
    pub(crate) dialog: AtomicU8,
    pub(crate) send_packet: AtomicU8,
    pub(crate) send_rpc: AtomicU8,
}

impl SelfTests {
    const fn new() -> Self {
        Self {
            packet: AtomicU8::new(SelfTestStatus::Pending.as_raw()),
            rpc: AtomicU8::new(SelfTestStatus::Pending.as_raw()),
            dialog: AtomicU8::new(SelfTestStatus::Pending.as_raw()),
            send_packet: AtomicU8::new(SelfTestStatus::Pending.as_raw()),
            send_rpc: AtomicU8::new(SelfTestStatus::Pending.as_raw()),
        }
    }
}

pub(crate) static STOP: AtomicBool = AtomicBool::new(false);
pub(crate) static METRICS: Metrics = Metrics::new();
pub(crate) static SELF_TESTS: SelfTests = SelfTests::new();
pub(crate) static STATS_PAYLOAD_READY: AtomicBool = AtomicBool::new(false);
pub(crate) static STATS_PAYLOAD: [AtomicU8; STATS_PAYLOAD_LEN] =
    [const { AtomicU8::new(0) }; STATS_PAYLOAD_LEN];
