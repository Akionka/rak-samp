//! Static RakNet and SA-MP identifier names.
//!
//! These helpers are a Rust-friendly equivalent of SF.lua's
//! `raknetGetRpcName` and `raknetGetPacketName`. They are pure lookups: no
//! host discovery, client pointer, or network operation is involved.

use crate::SampClientSdkEncodedString;

const MAX_BIT_STREAM_BITS: usize = 16 * 1024 * 1024 * u8::BITS as usize;

/// A bounded, owned RakNet-compatible bit stream for plugin-side construction.
///
/// Bits are stored most-significant-bit first in each byte. Numeric values use
/// little-endian byte order, matching the supported Windows x86 client. This
/// type owns only Rust memory; it never represents a native `RakNet::BitStream`
/// pointer and is safe to keep on a plugin worker thread.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BitStream {
    bytes: Vec<u8>,
    bit_len: usize,
    read_offset: usize,
}

/// A checked bit-stream operation failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BitStreamError {
    /// The requested range does not fit in the stream.
    OutOfBounds {
        requested_bits: usize,
        available_bits: usize,
    },
    /// The supplied bit length does not fit in its byte buffer.
    InvalidBitLength { bit_len: usize, byte_len: usize },
    /// The bounded plugin-side stream would grow beyond its safe limit.
    PayloadTooLarge { requested_bits: usize },
}

impl BitStream {
    /// Creates an empty stream, corresponding to SF.lua's `raknetNewBitStream`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bytes: Vec::new(),
            bit_len: 0,
            read_offset: 0,
        }
    }

    /// Creates a byte-aligned stream.
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self, BitStreamError> {
        let bytes = bytes.into();
        let bit_len =
            bytes
                .len()
                .checked_mul(u8::BITS as usize)
                .ok_or(BitStreamError::PayloadTooLarge {
                    requested_bits: usize::MAX,
                })?;
        Self::from_bits(bytes, bit_len)
    }

    /// Creates a stream from left-aligned meaningful bits in `bytes`.
    pub fn from_bits(bytes: impl Into<Vec<u8>>, bit_len: usize) -> Result<Self, BitStreamError> {
        let bytes = bytes.into();
        let available_bits = bytes.len().saturating_mul(u8::BITS as usize);
        if bit_len > available_bits {
            return Err(BitStreamError::InvalidBitLength {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        if bit_len > MAX_BIT_STREAM_BITS {
            return Err(BitStreamError::PayloadTooLarge {
                requested_bits: bit_len,
            });
        }
        let mut stream = Self {
            bytes,
            bit_len,
            read_offset: 0,
        };
        stream.trim_unused_bits();
        Ok(stream)
    }

    /// Returns the left-aligned storage used by the host send ABI.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the number of meaningful bits (`raknetBitStreamGetNumberOfBitsUsed`).
    #[must_use]
    pub const fn len_bits(&self) -> usize {
        self.bit_len
    }

    /// Returns the number of bytes containing meaningful bits.
    #[must_use]
    pub const fn len_bytes(&self) -> usize {
        self.bit_len.div_ceil(u8::BITS as usize)
    }

    /// Returns unread bits from the current read cursor.
    #[must_use]
    pub const fn remaining_bits(&self) -> usize {
        self.bit_len.saturating_sub(self.read_offset)
    }

    /// Returns the read cursor offset in bits.
    #[must_use]
    pub const fn read_offset_bits(&self) -> usize {
        self.read_offset
    }

    /// Returns the checked read cursor offset in bits.
    #[must_use]
    pub const fn read_offset(&self) -> usize {
        self.read_offset_bits()
    }

    /// Returns the write cursor offset in bits.
    #[must_use]
    pub const fn write_offset_bits(&self) -> usize {
        self.bit_len
    }

    /// Returns the checked write cursor offset in bits.
    #[must_use]
    pub const fn write_offset(&self) -> usize {
        self.write_offset_bits()
    }

    /// Clears data and both cursors, corresponding to `raknetResetBitStream`.
    pub fn reset(&mut self) {
        self.bytes.clear();
        self.bit_len = 0;
        self.read_offset = 0;
    }

    /// Clears the owned stream and both cursors.
    pub fn clear(&mut self) {
        self.reset();
    }

    /// Clears written data and resets the read cursor safely.
    ///
    /// Native `ResetWritePointer` can leave an invalid read cursor; this owned
    /// representation deliberately cannot.
    pub fn reset_write_pointer(&mut self) {
        self.reset();
    }

    /// Clears the write cursor and owned contents.
    pub fn reset_write(&mut self) {
        self.reset_write_pointer();
    }

    /// Resets the read cursor, corresponding to `raknetBitStreamResetReadPointer`.
    pub fn reset_read_pointer(&mut self) {
        self.read_offset = 0;
    }

    /// Resets the checked read cursor.
    pub fn reset_read(&mut self) {
        self.reset_read_pointer();
    }

    /// Sets the read cursor to a checked bit offset.
    pub fn set_read_offset(&mut self, offset_bits: usize) -> Result<(), BitStreamError> {
        if offset_bits > self.bit_len {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: offset_bits,
                available_bits: self.bit_len,
            });
        }
        self.read_offset = offset_bits;
        Ok(())
    }

    /// Truncates the stream to a checked write offset.
    ///
    /// Unlike the native raw-pointer function, this cannot advance into
    /// uninitialized backing storage.
    pub fn set_write_offset(&mut self, offset_bits: usize) -> Result<(), BitStreamError> {
        if offset_bits > self.bit_len {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: offset_bits,
                available_bits: self.bit_len,
            });
        }
        self.bit_len = offset_bits;
        self.read_offset = self.read_offset.min(offset_bits);
        self.bytes.truncate(self.len_bytes());
        self.trim_unused_bits();
        Ok(())
    }

    /// Advances the read cursor by a checked number of bits.
    pub fn ignore_bits(&mut self, bit_len: usize) -> Result<(), BitStreamError> {
        let remaining_bits = self.remaining_bits();
        if bit_len > remaining_bits {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: bit_len,
                available_bits: remaining_bits,
            });
        }
        self.read_offset += bit_len;
        Ok(())
    }

    /// Reads one bit.
    pub fn read_bool(&mut self) -> Result<bool, BitStreamError> {
        if self.read_offset == self.bit_len {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: 1,
                available_bits: 0,
            });
        }
        let value = self.bit_at(self.read_offset);
        self.read_offset += 1;
        Ok(value)
    }

    /// Writes one bit.
    pub fn write_bool(&mut self, value: bool) -> Result<(), BitStreamError> {
        self.ensure_additional_capacity(1)?;
        self.write_bit_unchecked(value);
        Ok(())
    }

    /// Reads an exact number of bits. A partial final output byte is right-aligned,
    /// matching SF.lua's `ReadBits(..., true)` behavior.
    pub fn read_bits(&mut self, bit_len: usize) -> Result<Vec<u8>, BitStreamError> {
        let remaining_bits = self.remaining_bits();
        if bit_len > remaining_bits {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: bit_len,
                available_bits: remaining_bits,
            });
        }
        let mut output = Vec::with_capacity(bit_len.div_ceil(u8::BITS as usize));
        let mut unread = bit_len;
        while unread != 0 {
            let group_bits = unread.min(u8::BITS as usize);
            let mut value = 0_u8;
            for _ in 0..group_bits {
                value = (value << 1) | u8::from(self.read_bool()?);
            }
            output.push(value);
            unread -= group_bits;
        }
        Ok(output)
    }

    /// Writes bits from `bytes`. A partial final byte must be right-aligned,
    /// matching SF.lua's `WriteBits(..., true)` behavior.
    pub fn write_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), BitStreamError> {
        let available_bits = bytes.len().saturating_mul(u8::BITS as usize);
        if bit_len > available_bits {
            return Err(BitStreamError::InvalidBitLength {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        self.ensure_additional_capacity(bit_len)?;
        let mut remaining = bit_len;
        let mut byte_index = 0;
        while remaining != 0 {
            let group_bits = remaining.min(u8::BITS as usize);
            let source = bytes[byte_index];
            for bit_index in 0..group_bits {
                let shift = if group_bits == u8::BITS as usize {
                    u8::BITS as usize - 1 - bit_index
                } else {
                    group_bits - 1 - bit_index
                };
                self.write_bit_unchecked(source & (1 << shift) != 0);
            }
            remaining -= group_bits;
            byte_index += 1;
        }
        Ok(())
    }

    /// Reads an exact byte buffer.
    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, BitStreamError> {
        let bit_len =
            len.checked_mul(u8::BITS as usize)
                .ok_or(BitStreamError::PayloadTooLarge {
                    requested_bits: usize::MAX,
                })?;
        self.read_bits(bit_len)
    }

    /// Writes an exact byte buffer.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), BitStreamError> {
        self.write_bits(bytes, bytes.len().saturating_mul(u8::BITS as usize))
    }

    /// Reads a byte string without assuming Unicode or a NUL terminator.
    pub fn read_string(&mut self, len: usize) -> Result<Vec<u8>, BitStreamError> {
        self.read_bytes(len)
    }

    /// Writes a byte string without appending a NUL terminator.
    pub fn write_string(&mut self, value: &[u8]) -> Result<(), BitStreamError> {
        self.write_bytes(value)
    }

    /// Reads a signed 8-bit integer.
    pub fn read_i8(&mut self) -> Result<i8, BitStreamError> {
        Ok(self.read_bytes(1)?[0] as i8)
    }

    /// Reads a signed 16-bit little-endian integer.
    pub fn read_i16(&mut self) -> Result<i16, BitStreamError> {
        Ok(i16::from_le_bytes(self.read_fixed()?))
    }

    /// Reads a signed 32-bit little-endian integer.
    pub fn read_i32(&mut self) -> Result<i32, BitStreamError> {
        Ok(i32::from_le_bytes(self.read_fixed()?))
    }

    /// Reads a little-endian IEEE-754 `f32`.
    pub fn read_f32(&mut self) -> Result<f32, BitStreamError> {
        Ok(f32::from_le_bytes(self.read_fixed()?))
    }

    /// Writes a signed 8-bit integer.
    pub fn write_i8(&mut self, value: i8) -> Result<(), BitStreamError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Reads one unsigned 8-bit integer.
    pub fn read_u8(&mut self) -> Result<u8, BitStreamError> {
        self.read_i8().map(|value| value as u8)
    }

    /// Writes one unsigned 8-bit integer.
    pub fn write_u8(&mut self, value: u8) -> Result<(), BitStreamError> {
        self.write_i8(value as i8)
    }

    /// Writes a signed 16-bit little-endian integer.
    pub fn write_i16(&mut self, value: i16) -> Result<(), BitStreamError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Reads one unsigned 16-bit integer.
    pub fn read_u16(&mut self) -> Result<u16, BitStreamError> {
        self.read_i16().map(|value| value as u16)
    }

    /// Writes one unsigned 16-bit integer.
    pub fn write_u16(&mut self, value: u16) -> Result<(), BitStreamError> {
        self.write_i16(value as i16)
    }

    /// Writes a signed 32-bit little-endian integer.
    pub fn write_i32(&mut self, value: i32) -> Result<(), BitStreamError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Reads one unsigned 32-bit integer.
    pub fn read_u32(&mut self) -> Result<u32, BitStreamError> {
        self.read_i32().map(|value| value as u32)
    }

    /// Writes one unsigned 32-bit integer.
    pub fn write_u32(&mut self, value: u32) -> Result<(), BitStreamError> {
        self.write_i32(value as i32)
    }

    /// Writes a little-endian IEEE-754 `f32`.
    pub fn write_f32(&mut self, value: f32) -> Result<(), BitStreamError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Appends another owned stream's meaningful bits.
    pub fn write_stream(&mut self, source: &Self) -> Result<(), BitStreamError> {
        self.ensure_additional_capacity(source.bit_len)?;
        for bit_offset in 0..source.bit_len {
            self.write_bit_unchecked(source.bit_at(bit_offset));
        }
        Ok(())
    }

    /// Appends a string encoded by [`crate::HostApi::encode_string`].
    pub fn write_encoded_string(
        &mut self,
        value: &SampClientSdkEncodedString,
    ) -> Result<(), BitStreamError> {
        self.write_left_aligned_bits(value.as_bytes(), value.len_bits())
    }

    fn read_fixed<const N: usize>(&mut self) -> Result<[u8; N], BitStreamError> {
        let bytes = self.read_bytes(N)?;
        let mut output = [0; N];
        output.copy_from_slice(&bytes);
        Ok(output)
    }

    fn ensure_additional_capacity(&self, additional_bits: usize) -> Result<(), BitStreamError> {
        let requested_bits =
            self.bit_len
                .checked_add(additional_bits)
                .ok_or(BitStreamError::PayloadTooLarge {
                    requested_bits: usize::MAX,
                })?;
        if requested_bits > MAX_BIT_STREAM_BITS {
            return Err(BitStreamError::PayloadTooLarge { requested_bits });
        }
        Ok(())
    }

    fn write_left_aligned_bits(
        &mut self,
        bytes: &[u8],
        bit_len: usize,
    ) -> Result<(), BitStreamError> {
        let available_bits = bytes.len().saturating_mul(u8::BITS as usize);
        if bit_len > available_bits {
            return Err(BitStreamError::InvalidBitLength {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        self.ensure_additional_capacity(bit_len)?;
        for bit_offset in 0..bit_len {
            let byte = bytes[bit_offset / u8::BITS as usize];
            let bit = byte & (0x80 >> (bit_offset % u8::BITS as usize)) != 0;
            self.write_bit_unchecked(bit);
        }
        Ok(())
    }

    fn write_bit_unchecked(&mut self, value: bool) {
        let byte_index = self.bit_len / u8::BITS as usize;
        let bit_index = self.bit_len % u8::BITS as usize;
        if bit_index == 0 {
            self.bytes.push(0);
        }
        if value {
            self.bytes[byte_index] |= 0x80 >> bit_index;
        }
        self.bit_len += 1;
    }

    fn bit_at(&self, bit_offset: usize) -> bool {
        let byte = self.bytes[bit_offset / u8::BITS as usize];
        byte & (0x80 >> (bit_offset % u8::BITS as usize)) != 0
    }

    fn trim_unused_bits(&mut self) {
        self.bytes.truncate(self.len_bytes());
        if let Some(last) = self.bytes.last_mut() {
            let used = self.bit_len % u8::BITS as usize;
            if used != 0 {
                *last &= u8::MAX << (u8::BITS as usize - used);
            }
        }
    }
}

/// Returns the SF.lua SA-MP RPC label for `id`, if the catalog defines one.
#[must_use]
pub const fn rpc_name(id: u8) -> Option<&'static str> {
    Some(match id {
        11 => "SetPlayerName",
        12 => "SetPlayerPos",
        13 => "SetPlayerPosFindZ",
        14 => "SetPlayerHealth",
        15 => "TogglePlayerControllable",
        16 => "PlaySound",
        17 => "SetPlayerWorldBounds",
        18 => "GivePlayerMoney",
        19 => "SetPlayerFacingAngle",
        20 => "ResetPlayerMoney",
        21 => "ResetPlayerWeapons",
        22 => "GivePlayerWeapon",
        23 => "ClickPlayer",
        24 => "SetVehicleParamsEx",
        25 => "ClientJoin",
        26 => "EnterVehicle",
        27 => "EnterEditObject",
        28 => "CancelEdit",
        29 => "SetPlayerTime",
        30 => "ToggleClock",
        31 => "ScriptCash",
        32 => "WorldPlayerAdd",
        33 => "SetPlayerShopName",
        34 => "SetPlayerSkillLevel",
        35 => "SetPlayerDrunkLevel",
        36 => "Create3DTextLabel",
        37 => "DisableCheckpoint",
        38 => "SetRaceCheckpoint",
        39 => "DisableRaceCheckpoint",
        40 => "GameModeRestart",
        41 => "PlayAudioStream",
        42 => "StopAudioStream",
        43 => "RemoveBuildingForPlayer",
        44 => "CreateObject",
        45 => "SetObjectPos",
        46 => "SetObjectRot",
        47 => "DestroyObject",
        50 => "ServerCommand",
        52 => "Spawn",
        53 => "Death",
        54 => "NPCJoin",
        55 => "DeathMessage",
        56 => "SetPlayerMapIcon",
        57 => "RemoveVehicleComponent",
        58 => "Update3DTextLabel",
        59 => "ChatBubble",
        60 => "UpdateSystemTime",
        61 => "ShowDialog",
        62 => "DialogResponse",
        63 => "DestroyPickup",
        64 => "WeaponPickupDestroy",
        65 => "LinkVehicleToInterior",
        66 => "SetPlayerArmour",
        67 => "SetPlayerArmedWeapon",
        68 => "SetSpawnInfo",
        69 => "SetPlayerTeam",
        70 => "PutPlayerInVehicle",
        71 => "RemovePlayerFromVehicle",
        72 => "SetPlayerColor",
        73 => "DisplayGameText",
        74 => "ForceClassSelection",
        75 => "AttachObjectToPlayer",
        76 => "InitMenu",
        77 => "ShowMenu",
        78 => "HideMenu",
        79 => "CreateExplosion",
        80 => "ShowPlayerNameTagForPlayer",
        81 => "AttachCameraToObject",
        82 => "InterpolateCamera",
        83 => "ClickTextDraw",
        84 => "SetObjectMaterial",
        85 => "GangZoneStopFlash",
        86 => "ApplyAnimation",
        87 => "ClearAnimations",
        88 => "SetPlayerSpecialAction",
        89 => "SetPlayerFightingStyle",
        90 => "SetPlayerVelocity",
        91 => "SetVehicleVelocity",
        92 => "SetPlayerDrunkVisuals",
        93 => "ClientMessage",
        94 => "SetWorldTime",
        95 => "CreatePickup",
        96 => "SCMEvent",
        98 => "SetVehicleTireStatus",
        99 => "MoveObject",
        101 => "Chat",
        102 => "SrvNetStats",
        103 => "ClientCheck",
        104 => "EnableStuntBonusForPlayer",
        105 => "TextDrawSetString",
        106 => "DamageVehicle",
        107 => "SetCheckpoint",
        108 => "GangZoneCreate",
        112 => "PlayCrimeReport",
        113 => "SetPlayerAttachedObject",
        115 => "GiveTakeDamage",
        116 => "EditAttachedObject",
        117 => "EditObject",
        118 => "SetInteriorId",
        119 => "MapMarker",
        120 => "GangZoneDestroy",
        121 => "GangZoneFlash",
        122 => "StopObject",
        123 => "SetNumberPlate",
        124 => "TogglePlayerSpectating",
        126 => "PlayerSpectatePlayer",
        127 => "PlayerSpectateVehicle",
        128 => "RequestClass",
        129 => "RequestSpawn",
        131 => "PickedUpPickup",
        132 => "MenuSelect",
        133 => "SetPlayerWantedLevel",
        134 => "ShowTextDraw",
        135 => "TextDrawHideForPlayer",
        136 => "VehicleDestroyed",
        137 => "ServerJoin",
        138 => "ServerQuit",
        139 => "InitGame",
        140 => "MenuQuit",
        144 => "RemovePlayerMapIcon",
        145 => "SetPlayerAmmo",
        146 => "SetPlayerGravity",
        147 => "SetVehicleHealth",
        148 => "AttachTrailerToVehicle",
        149 => "DetachTrailerFromVehicle",
        150 => "SetPlayerDrunkHandling",
        151 => "DestroyPickups",
        152 => "SetWeather",
        153 => "SetPlayerSkin",
        154 => "ExitVehicle",
        155 => "UpdateScoresPingsIPs",
        156 => "SetPlayerInterior",
        157 => "SetPlayerCameraPos",
        158 => "SetPlayerCameraLookAt",
        159 => "SetVehiclePos",
        160 => "SetVehicleZAngle",
        161 => "SetVehicleParamsForPlayer",
        162 => "SetCameraBehindPlayer",
        163 => "WorldPlayerRemove",
        164 => "WorldVehicleAdd",
        165 => "WorldVehicleRemove",
        166 => "WorldPlayerDeath",
        _ => return None,
    })
}

/// Returns the SF.lua RakNet packet label for `id`, if the catalog defines one.
#[must_use]
pub const fn packet_name(id: u8) -> Option<&'static str> {
    Some(match id {
        6 => "INTERNAL_PING",
        7 => "PING",
        8 => "PING_OPEN_CONNECTIONS",
        9 => "CONNECTED_PONG",
        10 => "REQUEST_STATIC_DATA",
        11 => "CONNECTION_REQUEST",
        12 => "AUTH_KEY",
        14 => "BROADCAST_PINGS",
        15 => "SECURED_CONNECTION_RESPONSE",
        16 => "SECURED_CONNECTION_CONFIRMATION",
        17 => "RPC_MAPPING",
        19 => "SET_RANDOM_NUMBER_SEED",
        20 => "RPC",
        21 => "RPC_REPLY",
        23 => "DETECT_LOST_CONNECTIONS",
        24 => "OPEN_CONNECTION_REQUEST",
        25 => "OPEN_CONNECTION_REPLY",
        26 => "CONNECTION_COOKIE",
        28 => "RSA_PUBLIC_KEY_MISMATCH",
        29 => "CONNECTION_ATTEMPT_FAILED",
        30 => "NEW_INCOMING_CONNECTION",
        31 => "NO_FREE_INCOMING_CONNECTIONS",
        32 => "DISCONNECTION_NOTIFICATION",
        33 => "CONNECTION_LOST",
        34 => "CONNECTION_REQUEST_ACCEPTED",
        35 => "INITIALIZE_ENCRYPTION",
        36 => "CONNECTION_BANNED",
        37 => "INVALID_PASSWORD",
        38 => "MODIFIED_PACKET",
        39 => "PONG",
        40 => "TIMESTAMP",
        41 => "RECEIVED_STATIC_DATA",
        42 => "REMOTE_DISCONNECTION_NOTIFICATION",
        43 => "REMOTE_CONNECTION_LOST",
        44 => "REMOTE_NEW_INCOMING_CONNECTION",
        45 => "REMOTE_EXISTING_CONNECTION",
        46 => "REMOTE_STATIC_DATA",
        56 => "ADVERTISE_SYSTEM",
        200 => "VEHICLE_SYNC",
        201 => "RCON_COMMAND",
        202 => "RCON_RESPONCE",
        203 => "AIM_SYNC",
        204 => "WEAPONS_UPDATE",
        205 => "STATS_UPDATE",
        206 => "BULLET_SYNC",
        207 => "PLAYER_SYNC",
        208 => "MARKERS_SYNC",
        209 => "UNOCCUPIED_SYNC",
        210 => "TRAILER_SYNC",
        211 => "PASSENGER_SYNC",
        212 => "SPECTATOR_SYNC",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{BitStream, BitStreamError, packet_name, rpc_name};

    #[test]
    fn rpc_names_match_the_sf_lua_catalog() {
        assert_eq!(rpc_name(61), Some("ShowDialog"));
        assert_eq!(rpc_name(62), Some("DialogResponse"));
        assert_eq!(rpc_name(139), Some("InitGame"));
        assert_eq!(rpc_name(0), None);
    }

    #[test]
    fn packet_names_match_the_sf_lua_catalog() {
        assert_eq!(packet_name(20), Some("RPC"));
        assert_eq!(packet_name(207), Some("PLAYER_SYNC"));
        assert_eq!(packet_name(255), None);
    }

    #[test]
    fn bit_stream_round_trips_unaligned_values_and_right_aligned_partial_bits() {
        let mut stream = BitStream::new();
        stream.write_bool(true).unwrap();
        stream.write_i8(-2).unwrap();
        stream.write_i16(-0x1234).unwrap();
        stream.write_i32(-123_456).unwrap();
        stream.write_f32(1.5).unwrap();
        stream.write_bits(&[0b0000_0101], 3).unwrap();

        assert_eq!(stream.len_bits(), 1 + 8 + 16 + 32 + 32 + 3);
        assert_eq!(stream.as_bytes().last(), Some(&0b1101_0000));
        assert!(stream.read_bool().unwrap());
        assert_eq!(stream.read_i8().unwrap(), -2);
        assert_eq!(stream.read_i16().unwrap(), -0x1234);
        assert_eq!(stream.read_i32().unwrap(), -123_456);
        assert_eq!(stream.read_f32().unwrap(), 1.5);
        assert_eq!(stream.read_bits(3).unwrap(), vec![0b0000_0101]);
        assert_eq!(stream.remaining_bits(), 0);
    }

    #[test]
    fn bit_stream_checks_cursors_and_never_exposes_uninitialized_bits() {
        let mut stream = BitStream::from_bytes([0xAB, 0xCD]).unwrap();
        stream.set_read_offset(4).unwrap();
        assert_eq!(stream.read_bits(4).unwrap(), vec![0x0B]);
        stream.ignore_bits(8).unwrap();
        assert_eq!(
            stream.read_bits(1),
            Err(BitStreamError::OutOfBounds {
                requested_bits: 1,
                available_bits: 0,
            })
        );
        stream.set_write_offset(12).unwrap();
        assert_eq!(stream.as_bytes(), &[0xAB, 0xC0]);
        assert_eq!(
            stream.set_write_offset(13),
            Err(BitStreamError::OutOfBounds {
                requested_bits: 13,
                available_bits: 12,
            })
        );
    }

    #[test]
    fn bit_stream_unsigned_and_cursor_aliases_preserve_the_owned_wire_data() {
        let mut stream = BitStream::new();
        stream.write_u8(0xFE).unwrap();
        stream.write_u16(0xFEDC).unwrap();
        stream.write_u32(0xFEDC_BA98).unwrap();
        assert_eq!(stream.write_offset_bits(), 56);

        stream.reset_read();
        assert_eq!(stream.read_u8(), Ok(0xFE));
        assert_eq!(stream.read_u16(), Ok(0xFEDC));
        assert_eq!(stream.read_u32(), Ok(0xFEDC_BA98));
        assert_eq!(stream.remaining_bits(), 0);

        stream.reset_write();
        assert_eq!(stream.len_bits(), 0);
        stream.write_string(b"owned").unwrap();
        stream.clear();
        assert_eq!(stream.len_bytes(), 0);
    }

    #[test]
    fn bit_stream_appends_meaningful_bits_from_another_stream() {
        let mut source = BitStream::new();
        source.write_bits(&[0b0000_0110], 3).unwrap();
        let mut destination = BitStream::new();
        destination.write_bool(true).unwrap();
        destination.write_stream(&source).unwrap();

        assert_eq!(destination.len_bits(), 4);
        assert_eq!(destination.as_bytes(), &[0b1110_0000]);
        assert_eq!(destination.read_bits(4).unwrap(), vec![0b0000_1110]);
    }
}
