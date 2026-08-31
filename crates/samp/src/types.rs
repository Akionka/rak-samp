use modkit_abi::{
    SAMP_MAX_NICKNAME_BYTES, SAMP_MAX_PLAYERS, SAMP_MAX_SERVER_ADDRESS_BYTES,
    SAMP_MAX_SERVER_HOSTNAME_BYTES, SAMP_VERSION_DL, SAMP_VERSION_R1, SAMP_VERSION_R2,
    SAMP_VERSION_R3_1, SAMP_VERSION_R4_2, SAMP_VERSION_R5_1, SampLocalPlayerV1, SampPlayerInfoV1,
    SampServerInfoV1, SampVector3V1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientVersion {
    R1,
    R2,
    R3_1,
    R4_2,
    R5_1,
    Dl,
}

impl ClientVersion {
    pub(crate) const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            SAMP_VERSION_R1 => Some(Self::R1),
            SAMP_VERSION_R2 => Some(Self::R2),
            SAMP_VERSION_R3_1 => Some(Self::R3_1),
            SAMP_VERSION_R4_2 => Some(Self::R4_2),
            SAMP_VERSION_R5_1 => Some(Self::R5_1),
            SAMP_VERSION_DL => Some(Self::Dl),
            _ => None,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameState {
    None = 0,
    WaitingForConnection = 9,
    Connecting = 13,
    Connected = 14,
    AwaitingJoin = 15,
    Restarting = 18,
}

impl GameState {
    #[must_use]
    pub const fn raw(self) -> i32 {
        self as i32
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendRateKind {
    OnFoot = modkit_abi::SAMP_SEND_RATE_ON_FOOT,
    InVehicle = modkit_abi::SAMP_SEND_RATE_IN_CAR,
    Aim = modkit_abi::SAMP_SEND_RATE_AIM,
}

impl SendRateKind {
    #[must_use]
    pub const fn raw(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlayerId(u16);

impl PlayerId {
    #[must_use]
    pub const fn new(raw: u16) -> Option<Self> {
        if raw < SAMP_MAX_PLAYERS {
            Some(Self(raw))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VehicleId(u16);

impl VehicleId {
    #[must_use]
    pub const fn new(raw: u16) -> Option<Self> {
        if raw < modkit_abi::SAMP_MAX_VEHICLES {
            Some(Self(raw))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TextLabelId(u16);

impl TextLabelId {
    #[must_use]
    pub const fn new(raw: u16) -> Option<Self> {
        if raw < modkit_abi::SAMP_MAX_TEXT_LABELS {
            Some(Self(raw))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<SampVector3V1> for Vector3 {
    fn from(value: SampVector3V1) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextLabel {
    pub id: TextLabelId,
    pub text: Vec<u8>,
    pub colour: u32,
    pub position: Vector3,
    pub draw_distance: f32,
    pub behind_walls: bool,
    pub attached_player_id: Option<PlayerId>,
    pub attached_vehicle_id: Option<VehicleId>,
}

impl TextLabel {
    pub(crate) fn from_abi(
        value: modkit_abi::SampTextLabelV1,
    ) -> Result<Self, modkit_abi::ModResult> {
        let Some(id) = TextLabelId::new(value.id) else {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        };
        let text_len = usize::from(value.text_len);
        if text_len > modkit_abi::SAMP_MAX_TEXT_LABEL_TEXT_BYTES || value.behind_walls > 1 {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        }
        let attached_player_id =
            if value.attached_player_id == modkit_abi::SAMP_TEXT_LABEL_NO_ATTACHMENT {
                None
            } else {
                Some(
                    PlayerId::new(value.attached_player_id)
                        .ok_or(modkit_abi::MOD_NATIVE_CALL_FAILED)?,
                )
            };
        let attached_vehicle_id =
            if value.attached_vehicle_id == modkit_abi::SAMP_TEXT_LABEL_NO_ATTACHMENT {
                None
            } else {
                Some(
                    VehicleId::new(value.attached_vehicle_id)
                        .ok_or(modkit_abi::MOD_NATIVE_CALL_FAILED)?,
                )
            };
        Ok(Self {
            id,
            text: value.text[..text_len].to_vec(),
            colour: value.colour,
            position: value.position.into(),
            draw_distance: value.draw_distance,
            behind_walls: value.behind_walls != 0,
            attached_player_id,
            attached_vehicle_id,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerInfo {
    pub address: Vec<u8>,
    pub hostname: Vec<u8>,
    pub port: u16,
}

impl ServerInfo {
    pub(crate) fn from_abi(value: SampServerInfoV1) -> Result<Self, modkit_abi::ModResult> {
        let address_len = usize::from(value.address_len);
        let hostname_len = usize::from(value.hostname_len);
        if address_len > SAMP_MAX_SERVER_ADDRESS_BYTES
            || hostname_len > SAMP_MAX_SERVER_HOSTNAME_BYTES
        {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        }
        Ok(Self {
            address: value.address[..address_len].to_vec(),
            hostname: value.hostname[..hostname_len].to_vec(),
            port: value.port,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalPlayer {
    pub id: PlayerId,
    pub nickname: Vec<u8>,
    pub colour: u32,
    pub spawned: bool,
    pub special_action: u8,
    pub animation_id: u16,
    pub health: f32,
    pub armour: f32,
    pub position: Vector3,
    pub velocity: Vector3,
    pub vehicle_id: Option<u16>,
    pub score: i32,
    pub ping: u32,
}

impl LocalPlayer {
    pub(crate) fn from_abi(value: SampLocalPlayerV1) -> Result<Self, modkit_abi::ModResult> {
        let Some(id) = PlayerId::new(value.id) else {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        };
        let nickname_len = usize::from(value.nickname_len);
        if nickname_len > SAMP_MAX_NICKNAME_BYTES || value.spawned > 1 || value.has_vehicle > 1 {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        }
        Ok(Self {
            id,
            nickname: value.nickname[..nickname_len].to_vec(),
            colour: value.colour,
            spawned: value.spawned != 0,
            special_action: value.special_action,
            animation_id: value.animation_id,
            health: value.health,
            armour: value.armour,
            position: value.position.into(),
            velocity: value.velocity.into(),
            vehicle_id: (value.has_vehicle != 0).then_some(value.vehicle_id),
            score: value.score,
            ping: value.ping,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub is_local: bool,
    pub is_npc: bool,
    pub nickname: Vec<u8>,
    pub colour: u32,
    pub score: i32,
    pub ping: u32,
}

impl PlayerInfo {
    pub(crate) fn from_abi(value: SampPlayerInfoV1) -> Result<Option<Self>, modkit_abi::ModResult> {
        if value.exists > 1 || value.is_local > 1 || value.is_npc > 1 {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        }
        if value.exists == 0 {
            return Ok(None);
        }
        let Some(id) = PlayerId::new(value.id) else {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        };
        let nickname_len = usize::from(value.nickname_len);
        if nickname_len > SAMP_MAX_NICKNAME_BYTES {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        }
        Ok(Some(Self {
            id,
            is_local: value.is_local != 0,
            is_npc: value.is_npc != 0,
            nickname: value.nickname[..nickname_len].to_vec(),
            colour: value.colour,
            score: value.score,
            ping: value.ping,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_id_rejects_the_public_upper_bound() {
        assert_eq!(PlayerId::new(SAMP_MAX_PLAYERS - 1).unwrap().get(), 1_003);
        assert!(PlayerId::new(SAMP_MAX_PLAYERS).is_none());
    }

    #[test]
    fn snapshots_reject_invalid_lengths_and_booleans() {
        let server = SampServerInfoV1 {
            address_len: SAMP_MAX_SERVER_ADDRESS_BYTES as u16 + 1,
            ..SampServerInfoV1::default()
        };
        assert_eq!(
            ServerInfo::from_abi(server),
            Err(modkit_abi::MOD_NATIVE_CALL_FAILED)
        );

        let local = SampLocalPlayerV1 {
            spawned: 2,
            ..SampLocalPlayerV1::default()
        };
        assert_eq!(
            LocalPlayer::from_abi(local),
            Err(modkit_abi::MOD_NATIVE_CALL_FAILED)
        );
    }

    #[test]
    fn text_label_rejects_out_of_range_attached_vehicle() {
        let label = modkit_abi::SampTextLabelV1 {
            attached_player_id: modkit_abi::SAMP_TEXT_LABEL_NO_ATTACHMENT,
            attached_vehicle_id: modkit_abi::SAMP_MAX_VEHICLES,
            ..modkit_abi::SampTextLabelV1::default()
        };
        assert_eq!(
            TextLabel::from_abi(label),
            Err(modkit_abi::MOD_NATIVE_CALL_FAILED)
        );
    }
}
