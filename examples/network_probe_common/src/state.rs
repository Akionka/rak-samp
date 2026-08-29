//! Shared probe state and copied observations.

use super::*;

pub(super) static STATE: Mutex<PluginState> = Mutex::new(PluginState::new());
pub(super) static INITIALIZATION_FINISHED: Condvar = Condvar::new();
pub(super) static STATUS: AtomicU32 = AtomicU32::new(0);
pub(super) static FAILURE: AtomicU32 = AtomicU32::new(SampClientSdkResult::Ok as u32);
pub(super) static SCALAR_OBSERVATION: Mutex<Option<ScalarObservation>> = Mutex::new(None);
pub(super) static PLAYER_POOL_OBSERVATION: Mutex<Option<PlayerPoolObservation>> = Mutex::new(None);
pub(super) static RECONNECT_OBSERVATION: Mutex<Option<ReconnectObservation>> = Mutex::new(None);
pub(super) static ENTITY_IDS: Mutex<Option<EntityIds>> = Mutex::new(None);
pub(super) static CHAT_COMMAND_INVOKED: AtomicBool = AtomicBool::new(false);
pub(super) static SYNC_PACKETS_OBSERVED: AtomicU32 = AtomicU32::new(0);
pub(super) static SYNC_PACKET_COUNTS: [AtomicU32; 8] = [const { AtomicU32::new(0) }; 8];
pub(super) static TEXT_LABEL_PHASE: Mutex<&'static str> = Mutex::new("none");
pub(super) static TEXT_LABEL_INITIAL_FIELDS: AtomicU32 = AtomicU32::new(0);
pub(super) static TEXT_LABEL_INITIAL_RESULT: AtomicU32 = AtomicU32::new(0);
pub(super) static TEXTDRAW_PHASE: Mutex<&'static str> = Mutex::new("none");
pub(super) static TEXTDRAW_SNAPSHOT_FIELDS: AtomicU32 = AtomicU32::new(0);
pub(super) static TEXTDRAW_SNAPSHOT_RESULT: AtomicU32 = AtomicU32::new(0);
pub(super) static VEHICLE_PHASE: Mutex<&'static str> = Mutex::new("none");
pub(super) static VEHICLE_PHASES: Mutex<VehiclePhases> = Mutex::new(VehiclePhases::new());
pub(super) static RECONNECT_REQUESTED: AtomicBool = AtomicBool::new(false);
pub(super) static INCOMING_REPLY_COUNT: AtomicU32 = AtomicU32::new(0);

pub(super) const SYNC_PACKET_AIM: u32 = 1 << 0;
pub(super) const SYNC_PACKET_ONFOOT: u32 = 1 << 1;
pub(super) const SYNC_PACKET_STATS: u32 = 1 << 2;
pub(super) const SYNC_PACKET_WEAPONS: u32 = 1 << 3;
pub(super) const SYNC_PACKET_VEHICLE: u32 = 1 << 4;
pub(super) const SYNC_PACKET_PASSENGER: u32 = 1 << 5;
pub(super) const SYNC_PACKET_UNOCCUPIED: u32 = 1 << 6;
pub(super) const SYNC_PACKET_TRAILER: u32 = 1 << 7;
pub(super) const SYNC_INDEX_AIM: usize = 0;
pub(super) const SYNC_INDEX_ONFOOT: usize = 1;
pub(super) const SYNC_INDEX_STATS: usize = 2;
pub(super) const SYNC_INDEX_WEAPONS: usize = 3;
pub(super) const SYNC_INDEX_VEHICLE: usize = 4;
pub(super) const SYNC_INDEX_PASSENGER: usize = 5;
pub(super) const SYNC_INDEX_UNOCCUPIED: usize = 6;
pub(super) const SYNC_INDEX_TRAILER: usize = 7;

pub(super) struct PluginState {
    pub(super) subscriptions: Option<SubscriptionSet>,
    pub(super) initializing: bool,
    pub(super) shutting_down: bool,
}

/// A bounded copied snapshot emitted only by this opt-in local validation probe.
pub(super) struct ScalarObservation {
    pub(super) game_state: i32,
    pub(super) address: Vec<u8>,
    pub(super) hostname: Vec<u8>,
    pub(super) port: u16,
}

/// The copied player-pool values observed only by this opt-in local probe.
pub(super) struct PlayerPoolObservation {
    pub(super) including_npcs: u16,
    pub(super) excluding_npcs: u16,
    pub(super) max_id: Option<u16>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct ReconnectObservation {
    pub(super) server_ready: bool,
    pub(super) local_ready: bool,
    pub(super) game_state: Option<i32>,
    pub(super) spawned: Option<bool>,
    pub(super) incoming_ready: bool,
}

#[derive(Clone, Copy)]
pub(super) struct ProbePhaseStatus {
    pub(super) text_label_phase: &'static str,
    pub(super) text_label_initial_fields: u32,
    pub(super) text_label_initial_result: u32,
    pub(super) textdraw_phase: &'static str,
    pub(super) textdraw_snapshot_fields: u32,
    pub(super) textdraw_snapshot_result: u32,
    pub(super) vehicle_phase: &'static str,
}

#[derive(Clone, Copy)]
pub(super) struct EntityIds {
    pub(super) object: u16,
    pub(super) vehicle: u16,
    pub(super) pickup: u16,
    pub(super) gangzone: u16,
}

#[derive(Clone, Copy)]
pub(super) struct VehiclePair {
    pub(super) vehicle: u16,
    pub(super) trailer: u16,
}

pub(super) struct VehiclePhases {
    pub(super) local_driver: Option<u16>,
    pub(super) local_passenger: Option<u16>,
    pub(super) local_trailer: Option<VehiclePair>,
    pub(super) cleanup: bool,
}

impl VehiclePhases {
    pub(super) const fn new() -> Self {
        Self {
            local_driver: None,
            local_passenger: None,
            local_trailer: None,
            cleanup: false,
        }
    }
}

impl PluginState {
    pub(super) const fn new() -> Self {
        Self {
            subscriptions: None,
            initializing: false,
            shutting_down: false,
        }
    }
}
