//! Typed SA-MP RPC helpers modeled after MoonLoader's `samp.events`.
//!
//! Register one raw incoming or outgoing RPC callback through [`crate::HostApi`], then invoke a
//! named helper from that callback. A helper ignores every other RPC ID, decodes its own payload,
//! and converts [`RpcAction`] back to the host ABI action. `Replace` clears and serializes the
//! complete RPC payload, just as returning a value table does in `samp.events`.
//!
//! Text fields deliberately use `Vec<u8>`: SA-MP text is not guaranteed to be UTF-8. Use
//! [`std::str::from_utf8`] only when the server's encoding is known.

use crate::{HostApi, RakRsEventV1, RakRsHookAction, RakRsResult};
use core::{fmt, marker::PhantomData};

/// Maximum supported length for a `string32` field in the initial helper set.
///
/// This covers the documented 4096-byte SA-MP dialog/info limit while preventing a malformed
/// server packet from requesting an unbounded allocation in a plugin.
pub const MAX_STRING32_BYTES: usize = 4096;
/// Maximum decoded text bytes accepted by `encodedString4096` helpers.
pub const MAX_ENCODED_STRING_BYTES: usize = 4095;

/// A three-dimensional SA-MP coordinate or velocity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// The typed decision returned by an RPC helper callback.
#[derive(Clone, Debug, PartialEq)]
pub enum RpcAction<T> {
    /// Preserve the original RPC payload.
    Continue,
    /// Do not pass the RPC to SA-MP.
    Block,
    /// Replace the complete RPC payload with this typed value.
    Replace(T),
}

/// An error while decoding or rewriting a callback-local RPC event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventError {
    /// The host rejected an operation on the callback event.
    Host(RakRsResult),
    /// A `string32` length exceeded the event's documented safe limit.
    LengthExceedsLimit { length: usize, limit: usize },
    /// A field cannot be encoded in the SA-MP representation used by this helper.
    ValueOutOfRange { value: usize, maximum: usize },
    /// The opaque event pointer provided to the callback was null.
    NullEvent,
    /// An exact bit length exceeded the supplied byte buffer.
    InvalidBitLength { bit_len: usize, byte_len: usize },
}

impl fmt::Display for EventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(result) => {
                write!(formatter, "rak-rs host event operation failed: {result:?}")
            }
            Self::LengthExceedsLimit { length, limit } => {
                write!(
                    formatter,
                    "RPC string length {length} exceeds the {limit}-byte limit"
                )
            }
            Self::ValueOutOfRange { value, maximum } => {
                write!(formatter, "value {value} exceeds the maximum {maximum}")
            }
            Self::NullEvent => formatter.write_str("rak-rs supplied a null callback event"),
            Self::InvalidBitLength { bit_len, byte_len } => write!(
                formatter,
                "bit length {bit_len} exceeds the {byte_len}-byte payload"
            ),
        }
    }
}

impl std::error::Error for EventError {}

/// A callback-local view over an opaque host event.
///
/// Construct this only with [`Event::from_callback`] inside the raw callback registered with the
/// host. It may not be retained after that callback returns.
pub struct Event<'callback> {
    api: HostApi,
    raw: *mut RakRsEventV1,
    _callback: PhantomData<&'callback mut RakRsEventV1>,
}

impl<'callback> Event<'callback> {
    /// Creates an event view for the raw pointer supplied to a host callback.
    ///
    /// # Safety
    ///
    /// `raw` must be the event pointer received from a currently executing `rak_rs` callback and
    /// the returned value must not outlive that callback.
    pub unsafe fn from_callback(api: HostApi, raw: *mut RakRsEventV1) -> Result<Self, EventError> {
        if raw.is_null() {
            return Err(EventError::NullEvent);
        }
        Ok(Self {
            api,
            raw,
            _callback: PhantomData,
        })
    }

    /// Returns the SA-MP packet or RPC identifier.
    #[must_use]
    pub fn id(&self) -> u8 {
        unsafe { (self.api.raw().event_id)(self.raw) }
    }

    /// Moves the read cursor to the start of the event payload.
    pub fn reset_read(&mut self) -> Result<(), EventError> {
        self.host_result(unsafe { (self.api.raw().event_reset_read)(self.raw) })
    }

    /// Removes the current payload before serializing a replacement.
    pub fn clear(&mut self) -> Result<(), EventError> {
        self.host_result(unsafe { (self.api.raw().event_clear)(self.raw) })
    }

    /// Atomically replaces this byte-aligned payload.
    pub fn replace_bytes(&mut self, value: &[u8]) -> Result<(), EventError> {
        self.host_result(unsafe {
            (self.api.raw().event_replace_bytes)(self.raw, value.as_ptr(), value.len())
        })
    }

    /// Atomically replaces this payload with an exact, possibly partial-byte bit length.
    pub fn replace_bits(&mut self, value: &[u8], bit_len: usize) -> Result<(), EventError> {
        if bit_len > value.len().saturating_mul(u8::BITS as usize) {
            return Err(EventError::InvalidBitLength {
                bit_len,
                byte_len: value.len(),
            });
        }
        self.host_result(unsafe {
            (self.api.raw().event_replace_bits)(self.raw, value.as_ptr(), value.len(), bit_len)
        })
    }

    /// Returns the number of unread payload bits.
    #[must_use]
    pub fn remaining_bits(&self) -> usize {
        unsafe { (self.api.raw().event_remaining_bits)(self.raw) }
    }

    /// Reads exact bits into a left-aligned byte buffer.
    pub fn read_bits(&mut self, bit_len: usize) -> Result<Vec<u8>, EventError> {
        let mut bytes = vec![0_u8; bit_len.div_ceil(u8::BITS as usize)];
        self.host_result(unsafe {
            (self.api.raw().event_read_bits)(self.raw, bytes.as_mut_ptr(), bit_len)
        })?;
        Ok(bytes)
    }

    /// Decodes one string with the current SA-MP client's RakNet compressor.
    pub fn read_encoded_string(&mut self, capacity: usize) -> Result<Vec<u8>, EventError> {
        if capacity == 0 {
            return Err(EventError::ValueOutOfRange {
                value: 0,
                maximum: 0,
            });
        }
        let mut bytes = vec![0_u8; capacity];
        let mut length = 0;
        self.host_result(unsafe {
            (self.api.raw().event_read_encoded_string)(
                self.raw,
                bytes.as_mut_ptr(),
                bytes.len(),
                &raw mut length,
            )
        })?;
        if length > bytes.len() {
            return Err(EventError::Host(RakRsResult::NativeCallFailed));
        }
        bytes.truncate(length);
        Ok(bytes)
    }

    /// Reads an unsigned byte.
    pub fn read_u8(&mut self) -> Result<u8, EventError> {
        let mut value = 0;
        self.host_result(unsafe { (self.api.raw().event_read_u8)(self.raw, &raw mut value) })?;
        Ok(value)
    }

    /// Reads an unsigned 16-bit integer.
    pub fn read_u16(&mut self) -> Result<u16, EventError> {
        let mut value = 0;
        self.host_result(unsafe { (self.api.raw().event_read_u16)(self.raw, &raw mut value) })?;
        Ok(value)
    }

    /// Reads an unsigned 32-bit integer.
    pub fn read_u32(&mut self) -> Result<u32, EventError> {
        let mut value = 0;
        self.host_result(unsafe { (self.api.raw().event_read_u32)(self.raw, &raw mut value) })?;
        Ok(value)
    }

    /// Reads an IEEE-754 single-precision float.
    pub fn read_f32(&mut self) -> Result<f32, EventError> {
        let mut value = 0.0;
        self.host_result(unsafe { (self.api.raw().event_read_f32)(self.raw, &raw mut value) })?;
        Ok(value)
    }

    /// Reads exactly `length` bytes.
    pub fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>, EventError> {
        let mut bytes = vec![0; length];
        self.host_result(unsafe {
            (self.api.raw().event_read_bytes)(self.raw, bytes.as_mut_ptr(), bytes.len())
        })?;
        Ok(bytes)
    }

    /// Reads a `uint8`-length-prefixed byte string.
    pub fn read_string8(&mut self) -> Result<Vec<u8>, EventError> {
        let length = usize::from(self.read_u8()?);
        self.read_bytes(length)
    }

    /// Reads a bounded `uint32`-length-prefixed byte string.
    pub fn read_string32(&mut self, limit: usize) -> Result<Vec<u8>, EventError> {
        let length = self.read_u32()? as usize;
        if length > limit {
            return Err(EventError::LengthExceedsLimit { length, limit });
        }
        self.read_bytes(length)
    }

    /// Writes an unsigned byte.
    pub fn write_u8(&mut self, value: u8) -> Result<(), EventError> {
        self.host_result(unsafe { (self.api.raw().event_write_u8)(self.raw, value) })
    }

    /// Writes an unsigned 16-bit integer.
    pub fn write_u16(&mut self, value: u16) -> Result<(), EventError> {
        self.host_result(unsafe { (self.api.raw().event_write_u16)(self.raw, value) })
    }

    /// Writes an unsigned 32-bit integer.
    pub fn write_u32(&mut self, value: u32) -> Result<(), EventError> {
        self.host_result(unsafe { (self.api.raw().event_write_u32)(self.raw, value) })
    }

    /// Writes an IEEE-754 single-precision float.
    pub fn write_f32(&mut self, value: f32) -> Result<(), EventError> {
        self.host_result(unsafe { (self.api.raw().event_write_f32)(self.raw, value) })
    }

    /// Writes bytes without a length prefix.
    pub fn write_bytes(&mut self, value: &[u8]) -> Result<(), EventError> {
        self.host_result(unsafe {
            (self.api.raw().event_write_bytes)(self.raw, value.as_ptr(), value.len())
        })
    }

    /// Writes a `uint8`-length-prefixed byte string.
    pub fn write_string8(&mut self, value: &[u8]) -> Result<(), EventError> {
        if value.len() > u8::MAX as usize {
            return Err(EventError::ValueOutOfRange {
                value: value.len(),
                maximum: u8::MAX as usize,
            });
        }
        self.write_u8(value.len() as u8)?;
        self.write_bytes(value)
    }

    /// Writes a bounded `uint32`-length-prefixed byte string.
    pub fn write_string32(&mut self, value: &[u8], limit: usize) -> Result<(), EventError> {
        if value.len() > limit {
            return Err(EventError::ValueOutOfRange {
                value: value.len(),
                maximum: limit,
            });
        }
        self.write_u32(value.len() as u32)?;
        self.write_bytes(value)
    }

    fn host_result(&self, result: RakRsResult) -> Result<(), EventError> {
        if result == RakRsResult::Ok {
            Ok(())
        } else {
            Err(EventError::Host(result))
        }
    }
}

struct PayloadWriter {
    bytes: Vec<u8>,
    bit_len: usize,
}

impl PayloadWriter {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            bit_len: 0,
        }
    }

    fn finish(self) -> Vec<u8> {
        debug_assert_eq!(self.bit_len, self.bytes.len() * u8::BITS as usize);
        self.bytes
    }

    fn finish_bits(self) -> EncodedPayload {
        EncodedPayload {
            bytes: self.bytes,
            bit_len: self.bit_len,
        }
    }

    fn u8(&mut self, value: u8) {
        self.bytes(&[value]);
    }

    fn u16(&mut self, value: u16) {
        self.bytes(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }

    fn f32(&mut self, value: f32) {
        self.u32(value.to_bits());
    }

    fn bytes(&mut self, value: &[u8]) {
        self.bits(value, value.len() * u8::BITS as usize);
    }

    fn bits(&mut self, value: &[u8], bit_len: usize) {
        debug_assert!(bit_len <= value.len() * u8::BITS as usize);
        for bit_offset in 0..bit_len {
            self.bit(value[bit_offset / 8] & (0x80 >> (bit_offset % 8)) != 0);
        }
    }

    fn bit(&mut self, value: bool) {
        let byte_index = self.bit_len / u8::BITS as usize;
        let bit_index = self.bit_len % u8::BITS as usize;
        if byte_index == self.bytes.len() {
            self.bytes.push(0);
        }
        if value {
            self.bytes[byte_index] |= 0x80 >> bit_index;
        }
        self.bit_len += 1;
    }

    fn string8(&mut self, value: &[u8]) -> Result<(), EventError> {
        if value.len() > u8::MAX as usize {
            return Err(EventError::ValueOutOfRange {
                value: value.len(),
                maximum: u8::MAX as usize,
            });
        }
        self.u8(value.len() as u8);
        self.bytes(value);
        Ok(())
    }

    fn string32(&mut self, value: &[u8]) -> Result<(), EventError> {
        if value.len() > MAX_STRING32_BYTES {
            return Err(EventError::ValueOutOfRange {
                value: value.len(),
                maximum: MAX_STRING32_BYTES,
            });
        }
        self.u32(value.len() as u32);
        self.bytes(value);
        Ok(())
    }

    fn vector3(&mut self, value: Vector3) {
        self.f32(value.x);
        self.f32(value.y);
        self.f32(value.z);
    }

    fn encoded_string(&mut self, api: HostApi, value: &[u8]) -> Result<(), EventError> {
        if value.len() > MAX_ENCODED_STRING_BYTES {
            return Err(EventError::ValueOutOfRange {
                value: value.len(),
                maximum: MAX_ENCODED_STRING_BYTES,
            });
        }
        let encoded = api.encode_string(value).map_err(EventError::Host)?;
        self.bits(encoded.as_bytes(), encoded.len_bits());
        Ok(())
    }
}

/// A complete callback replacement with an exact bit length.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedPayload {
    bytes: Vec<u8>,
    bit_len: usize,
}

impl EncodedPayload {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, EventError> {
        let bit_len =
            bytes
                .len()
                .checked_mul(u8::BITS as usize)
                .ok_or(EventError::ValueOutOfRange {
                    value: bytes.len(),
                    maximum: usize::MAX / u8::BITS as usize,
                })?;
        Ok(Self { bytes, bit_len })
    }

    pub fn from_bits(bytes: Vec<u8>, bit_len: usize) -> Result<Self, EventError> {
        if bit_len > bytes.len().saturating_mul(u8::BITS as usize) {
            return Err(EventError::InvalidBitLength {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        Ok(Self { bytes, bit_len })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn len_bits(&self) -> usize {
        self.bit_len
    }
}

#[derive(Clone, Copy)]
enum RpcEncoder<T> {
    Bytes(fn(T) -> Result<Vec<u8>, EventError>),
    Bits(fn(HostApi, T) -> Result<EncodedPayload, EventError>),
}

/// A typed RPC descriptor with its SA-MP RPC ID and read/write layout.
#[derive(Clone, Copy)]
pub struct Rpc<T> {
    id: u8,
    decode: fn(&mut Event<'_>) -> Result<T, EventError>,
    encode: RpcEncoder<T>,
}

impl<T> Rpc<T> {
    /// Creates a descriptor for one RPC ID.
    pub const fn new(
        id: u8,
        decode: fn(&mut Event<'_>) -> Result<T, EventError>,
        encode: fn(T) -> Result<Vec<u8>, EventError>,
    ) -> Self {
        Self {
            id,
            decode,
            encode: RpcEncoder::Bytes(encode),
        }
    }

    /// Creates a descriptor whose replacement serializer can use host codecs and exact bits.
    pub const fn new_bits(
        id: u8,
        decode: fn(&mut Event<'_>) -> Result<T, EventError>,
        encode: fn(HostApi, T) -> Result<EncodedPayload, EventError>,
    ) -> Self {
        Self {
            id,
            decode,
            encode: RpcEncoder::Bits(encode),
        }
    }

    /// Returns this descriptor's SA-MP RPC ID.
    #[must_use]
    pub const fn id(self) -> u8 {
        self.id
    }

    /// Serializes one complete payload without mutating a callback event.
    pub fn encode(self, api: HostApi, value: T) -> Result<EncodedPayload, EventError> {
        match self.encode {
            RpcEncoder::Bytes(encode) => EncodedPayload::from_bytes(encode(value)?),
            RpcEncoder::Bits(encode) => encode(api, value),
        }
    }

    /// Handles this RPC when `event` has the matching ID.
    ///
    /// A non-matching event passes through without invoking `handler`. Decode failures are returned
    /// to the plugin so it can fail open and report the incompatible payload if appropriate.
    pub fn handle(
        self,
        event: &mut Event<'_>,
        handler: impl FnOnce(T) -> RpcAction<T>,
    ) -> Result<RakRsHookAction, EventError> {
        if event.id() != self.id {
            return Ok(RakRsHookAction::Continue);
        }
        event.reset_read()?;
        let value = (self.decode)(event)?;
        match handler(value) {
            RpcAction::Continue => Ok(RakRsHookAction::Continue),
            RpcAction::Block => Ok(RakRsHookAction::Block),
            RpcAction::Replace(value) => {
                let payload = self.encode(event.api, value)?;
                event.replace_bits(payload.as_bytes(), payload.len_bits())?;
                Ok(RakRsHookAction::Continue)
            }
        }
    }
}

/// Calls a typed descriptor from a raw callback event.
///
/// # Safety
///
/// `raw` must be the event pointer supplied to the currently executing callback. On an error,
/// return [`RakRsHookAction::Continue`] so malformed traffic remains fail-open.
pub unsafe fn handle<T>(
    api: HostApi,
    raw: *mut RakRsEventV1,
    rpc: Rpc<T>,
    handler: impl FnOnce(T) -> RpcAction<T>,
) -> Result<RakRsHookAction, EventError> {
    let mut event = unsafe { Event::from_callback(api, raw) }?;
    rpc.handle(&mut event, handler)
}

/// Incoming server-to-client RPC helpers.
pub mod incoming {
    use super::*;

    /// MoonLoader's `onServerMessage` payload (RPC 93).
    #[derive(Clone, Debug, PartialEq)]
    pub struct ServerMessage {
        pub color: u32,
        pub text: Vec<u8>,
    }

    /// MoonLoader's `onDisplayGameText` payload (RPC 73).
    #[derive(Clone, Debug, PartialEq)]
    pub struct GameText {
        pub style: i32,
        pub time_ms: i32,
        pub text: Vec<u8>,
    }

    /// MoonLoader's `onShowDialog` payload (RPC 61).
    #[derive(Clone, Debug, PartialEq)]
    pub struct ShowDialog {
        pub dialog_id: u16,
        pub style: u8,
        pub title: Vec<u8>,
        pub button1: Vec<u8>,
        pub button2: Vec<u8>,
        pub text: Vec<u8>,
    }

    /// MoonLoader's `onPlaySound` payload (RPC 16).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlaySound {
        pub sound_id: i32,
        pub position: Vector3,
    }

    /// MoonLoader's `onSetCheckpoint` payload (RPC 107).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Checkpoint {
        pub position: Vector3,
        pub radius: f32,
    }

    /// MoonLoader's `onChatMessage` payload (RPC 101).
    #[derive(Clone, Debug, PartialEq)]
    pub struct ChatMessage {
        pub player_id: u16,
        pub text: Vec<u8>,
    }

    /// MoonLoader's `onPlayerChatBubble` payload (RPC 59).
    #[derive(Clone, Debug, PartialEq)]
    pub struct ChatBubble {
        pub player_id: u16,
        pub color: u32,
        pub draw_distance: f32,
        pub duration_ms: i32,
        pub text: Vec<u8>,
    }

    /// MoonLoader's `onPlayerJoin` payload (RPC 137).
    #[derive(Clone, Debug, PartialEq)]
    pub struct PlayerJoin {
        pub player_id: u16,
        pub color: u32,
        pub is_npc: bool,
        pub nickname: Vec<u8>,
    }

    /// MoonLoader's `onPlayerQuit` payload (RPC 138).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerQuit {
        pub player_id: u16,
        pub reason: u8,
    }

    /// MoonLoader's `onSetPlayerName` payload (RPC 11).
    #[derive(Clone, Debug, PartialEq)]
    pub struct PlayerName {
        pub player_id: u16,
        pub name: Vec<u8>,
        pub success: bool,
    }

    /// MoonLoader's `onSetPlayerTime` payload (RPC 29).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerTime {
        pub hour: u8,
        pub minute: u8,
    }

    /// MoonLoader's `onSetWorldBounds` payload (RPC 17).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct WorldBounds {
        pub max_x: f32,
        pub min_x: f32,
        pub max_y: f32,
        pub min_y: f32,
    }

    /// MoonLoader's `onGivePlayerWeapon` payload (RPC 22).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerWeapon {
        pub weapon_id: i32,
        pub ammo: i32,
    }

    /// MoonLoader's `onSetPlayerTeam` payload (RPC 69).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerTeam {
        pub player_id: u16,
        pub team_id: u8,
    }

    /// MoonLoader's `onPutPlayerInVehicle` payload (RPC 70).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PutPlayerInVehicle {
        pub vehicle_id: u16,
        pub seat_id: u8,
    }

    /// MoonLoader's `onSetVehiclePosition` payload (RPC 159).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehiclePosition {
        pub vehicle_id: u16,
        pub position: Vector3,
    }

    /// MoonLoader's `onSetVehicleAngle` payload (RPC 160).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleAngle {
        pub vehicle_id: u16,
        pub angle: f32,
    }

    /// MoonLoader's `onSetVehicleHealth` payload (RPC 147).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleHealth {
        pub vehicle_id: u16,
        pub health: f32,
    }

    /// The `onServerMessage` descriptor.
    pub const SERVER_MESSAGE: Rpc<ServerMessage> =
        Rpc::new(93, decode_server_message, encode_server_message);
    /// The `onDisplayGameText` descriptor.
    pub const DISPLAY_GAME_TEXT: Rpc<GameText> = Rpc::new(73, decode_game_text, encode_game_text);
    /// The `onShowDialog` descriptor.
    pub const SHOW_DIALOG: Rpc<ShowDialog> =
        Rpc::new_bits(61, decode_show_dialog, encode_show_dialog);
    /// The `onSetPlayerPos` descriptor.
    pub const SET_PLAYER_POS: Rpc<Vector3> = Rpc::new(12, decode_vector3, encode_vector3);
    /// The `onSetPlayerPosFindZ` descriptor.
    pub const SET_PLAYER_POS_FIND_Z: Rpc<Vector3> = Rpc::new(13, decode_vector3, encode_vector3);
    /// The `onSetPlayerHealth` descriptor.
    pub const SET_PLAYER_HEALTH: Rpc<f32> = Rpc::new(14, decode_f32, encode_f32);
    /// The `onSetPlayerArmour` descriptor.
    pub const SET_PLAYER_ARMOUR: Rpc<f32> = Rpc::new(66, decode_f32, encode_f32);
    /// The `onSetPlayerFacingAngle` descriptor.
    pub const SET_PLAYER_FACING_ANGLE: Rpc<f32> = Rpc::new(19, decode_f32, encode_f32);
    /// The `onTogglePlayerControllable` descriptor.
    pub const TOGGLE_PLAYER_CONTROLLABLE: Rpc<bool> = Rpc::new(15, decode_bool8, encode_bool8);
    /// The `onPlaySound` descriptor.
    pub const PLAY_SOUND: Rpc<PlaySound> = Rpc::new(16, decode_play_sound, encode_play_sound);
    /// The `onSetCheckpoint` descriptor.
    pub const SET_CHECKPOINT: Rpc<Checkpoint> = Rpc::new(107, decode_checkpoint, encode_checkpoint);
    /// The `onChatMessage` descriptor.
    pub const CHAT_MESSAGE: Rpc<ChatMessage> =
        Rpc::new(101, decode_chat_message, encode_chat_message);
    /// The `onPlayerChatBubble` descriptor.
    pub const CHAT_BUBBLE: Rpc<ChatBubble> = Rpc::new(59, decode_chat_bubble, encode_chat_bubble);
    /// The `onPlayerJoin` descriptor.
    pub const PLAYER_JOIN: Rpc<PlayerJoin> = Rpc::new(137, decode_player_join, encode_player_join);
    /// The `onPlayerQuit` descriptor.
    pub const PLAYER_QUIT: Rpc<PlayerQuit> = Rpc::new(138, decode_player_quit, encode_player_quit);
    /// The `onSetPlayerName` descriptor.
    pub const SET_PLAYER_NAME: Rpc<PlayerName> =
        Rpc::new(11, decode_player_name, encode_player_name);
    /// The `onSetPlayerTime` descriptor.
    pub const SET_PLAYER_TIME: Rpc<PlayerTime> =
        Rpc::new(29, decode_player_time, encode_player_time);
    /// The `onSetWorldBounds` descriptor.
    pub const SET_WORLD_BOUNDS: Rpc<WorldBounds> =
        Rpc::new(17, decode_world_bounds, encode_world_bounds);
    /// The `onGivePlayerMoney` descriptor.
    pub const GIVE_PLAYER_MONEY: Rpc<i32> = Rpc::new(18, decode_i32, encode_i32);
    /// The `onGivePlayerWeapon` descriptor.
    pub const GIVE_PLAYER_WEAPON: Rpc<PlayerWeapon> =
        Rpc::new(22, decode_player_weapon, encode_player_weapon);
    /// The `onSetWorldTime` descriptor.
    pub const SET_WORLD_TIME: Rpc<u8> = Rpc::new(94, decode_u8, encode_u8);
    /// The `onSetWeather` descriptor.
    pub const SET_WEATHER: Rpc<u8> = Rpc::new(152, decode_u8, encode_u8);
    /// The `onSetInterior` descriptor.
    pub const SET_INTERIOR: Rpc<u8> = Rpc::new(156, decode_u8, encode_u8);
    /// The `onSetPlayerArmedWeapon` descriptor.
    pub const SET_PLAYER_ARMED_WEAPON: Rpc<i32> = Rpc::new(67, decode_i32, encode_i32);
    /// The `onSetPlayerWantedLevel` descriptor.
    pub const SET_PLAYER_WANTED_LEVEL: Rpc<u8> = Rpc::new(133, decode_u8, encode_u8);
    /// The `onSetPlayerTeam` descriptor.
    pub const SET_PLAYER_TEAM: Rpc<PlayerTeam> =
        Rpc::new(69, decode_player_team, encode_player_team);
    /// The `onPutPlayerInVehicle` descriptor.
    pub const PUT_PLAYER_IN_VEHICLE: Rpc<PutPlayerInVehicle> = Rpc::new(
        70,
        decode_put_player_in_vehicle,
        encode_put_player_in_vehicle,
    );
    /// The `onPlayerStreamOut` descriptor.
    pub const PLAYER_STREAM_OUT: Rpc<u16> = Rpc::new(163, decode_u16, encode_u16);
    /// The `onVehicleStreamOut` descriptor.
    pub const VEHICLE_STREAM_OUT: Rpc<u16> = Rpc::new(165, decode_u16, encode_u16);
    /// The `onSetVehiclePosition` descriptor.
    pub const SET_VEHICLE_POSITION: Rpc<VehiclePosition> =
        Rpc::new(159, decode_vehicle_position, encode_vehicle_position);
    /// The `onSetVehicleAngle` descriptor (`vehicle_id`, then angle).
    pub const SET_VEHICLE_ANGLE: Rpc<VehicleAngle> =
        Rpc::new(160, decode_vehicle_angle, encode_vehicle_angle);
    /// The `onSetVehicleHealth` descriptor.
    pub const SET_VEHICLE_HEALTH: Rpc<VehicleHealth> =
        Rpc::new(147, decode_vehicle_health, encode_vehicle_health);
    /// The `onResetPlayerMoney` descriptor.
    pub const RESET_PLAYER_MONEY: Rpc<()> = Rpc::new(20, decode_empty, encode_empty);
    /// The `onResetPlayerWeapons` descriptor.
    pub const RESET_PLAYER_WEAPONS: Rpc<()> = Rpc::new(21, decode_empty, encode_empty);
    /// The `onDisableCheckpoint` descriptor.
    pub const DISABLE_CHECKPOINT: Rpc<()> = Rpc::new(37, decode_empty, encode_empty);
    /// The `onDisableRaceCheckpoint` descriptor.
    pub const DISABLE_RACE_CHECKPOINT: Rpc<()> = Rpc::new(39, decode_empty, encode_empty);
    /// The `onGamemodeRestart` descriptor.
    pub const GAMEMODE_RESTART: Rpc<()> = Rpc::new(40, decode_empty, encode_empty);
    /// The `onStopAudioStream` descriptor.
    pub const STOP_AUDIO_STREAM: Rpc<()> = Rpc::new(42, decode_empty, encode_empty);
    /// The `onRemovePlayerFromVehicle` descriptor.
    pub const REMOVE_PLAYER_FROM_VEHICLE: Rpc<()> = Rpc::new(71, decode_empty, encode_empty);
    /// The `onForceClassSelection` descriptor.
    pub const FORCE_CLASS_SELECTION: Rpc<()> = Rpc::new(74, decode_empty, encode_empty);
    /// The `onSetCameraBehind` descriptor.
    pub const SET_CAMERA_BEHIND: Rpc<()> = Rpc::new(162, decode_empty, encode_empty);

    /// Handles `onServerMessage` from an incoming raw RPC callback.
    ///
    /// # Safety
    ///
    /// See [`super::handle`].
    pub unsafe fn on_server_message(
        api: HostApi,
        raw: *mut RakRsEventV1,
        handler: impl FnOnce(ServerMessage) -> RpcAction<ServerMessage>,
    ) -> Result<RakRsHookAction, EventError> {
        unsafe { handle(api, raw, SERVER_MESSAGE, handler) }
    }

    macro_rules! rpc_helper {
        ($name:ident, $value:ty, $rpc:ident, $event_name:literal) => {
            #[doc = concat!("Handles MoonLoader's `", $event_name, "` from an incoming raw RPC callback.")]
            ///
            /// # Safety
            ///
            /// See [`super::handle`].
            pub unsafe fn $name(
                api: HostApi,
                raw: *mut RakRsEventV1,
                handler: impl FnOnce($value) -> RpcAction<$value>,
            ) -> Result<RakRsHookAction, EventError> {
                unsafe { handle(api, raw, $rpc, handler) }
            }
        };
    }

    rpc_helper!(
        on_display_game_text,
        GameText,
        DISPLAY_GAME_TEXT,
        "onDisplayGameText"
    );
    rpc_helper!(on_show_dialog, ShowDialog, SHOW_DIALOG, "onShowDialog");
    rpc_helper!(on_set_player_pos, Vector3, SET_PLAYER_POS, "onSetPlayerPos");
    rpc_helper!(
        on_set_player_pos_find_z,
        Vector3,
        SET_PLAYER_POS_FIND_Z,
        "onSetPlayerPosFindZ"
    );
    rpc_helper!(
        on_set_player_health,
        f32,
        SET_PLAYER_HEALTH,
        "onSetPlayerHealth"
    );
    rpc_helper!(
        on_set_player_armour,
        f32,
        SET_PLAYER_ARMOUR,
        "onSetPlayerArmour"
    );
    rpc_helper!(
        on_set_player_facing_angle,
        f32,
        SET_PLAYER_FACING_ANGLE,
        "onSetPlayerFacingAngle"
    );
    rpc_helper!(
        on_toggle_player_controllable,
        bool,
        TOGGLE_PLAYER_CONTROLLABLE,
        "onTogglePlayerControllable"
    );
    rpc_helper!(on_play_sound, PlaySound, PLAY_SOUND, "onPlaySound");
    rpc_helper!(
        on_set_checkpoint,
        Checkpoint,
        SET_CHECKPOINT,
        "onSetCheckpoint"
    );
    rpc_helper!(on_chat_message, ChatMessage, CHAT_MESSAGE, "onChatMessage");
    rpc_helper!(
        on_player_chat_bubble,
        ChatBubble,
        CHAT_BUBBLE,
        "onPlayerChatBubble"
    );
    rpc_helper!(on_player_join, PlayerJoin, PLAYER_JOIN, "onPlayerJoin");
    rpc_helper!(on_player_quit, PlayerQuit, PLAYER_QUIT, "onPlayerQuit");
    rpc_helper!(
        on_set_player_name,
        PlayerName,
        SET_PLAYER_NAME,
        "onSetPlayerName"
    );
    rpc_helper!(
        on_set_player_time,
        PlayerTime,
        SET_PLAYER_TIME,
        "onSetPlayerTime"
    );
    rpc_helper!(
        on_set_world_bounds,
        WorldBounds,
        SET_WORLD_BOUNDS,
        "onSetWorldBounds"
    );
    rpc_helper!(
        on_give_player_money,
        i32,
        GIVE_PLAYER_MONEY,
        "onGivePlayerMoney"
    );
    rpc_helper!(
        on_give_player_weapon,
        PlayerWeapon,
        GIVE_PLAYER_WEAPON,
        "onGivePlayerWeapon"
    );
    rpc_helper!(on_set_world_time, u8, SET_WORLD_TIME, "onSetWorldTime");
    rpc_helper!(on_set_weather, u8, SET_WEATHER, "onSetWeather");
    rpc_helper!(on_set_interior, u8, SET_INTERIOR, "onSetInterior");
    rpc_helper!(
        on_set_player_armed_weapon,
        i32,
        SET_PLAYER_ARMED_WEAPON,
        "onSetPlayerArmedWeapon"
    );
    rpc_helper!(
        on_set_player_wanted_level,
        u8,
        SET_PLAYER_WANTED_LEVEL,
        "onSetPlayerWantedLevel"
    );
    rpc_helper!(
        on_set_player_team,
        PlayerTeam,
        SET_PLAYER_TEAM,
        "onSetPlayerTeam"
    );
    rpc_helper!(
        on_put_player_in_vehicle,
        PutPlayerInVehicle,
        PUT_PLAYER_IN_VEHICLE,
        "onPutPlayerInVehicle"
    );
    rpc_helper!(
        on_player_stream_out,
        u16,
        PLAYER_STREAM_OUT,
        "onPlayerStreamOut"
    );
    rpc_helper!(
        on_vehicle_stream_out,
        u16,
        VEHICLE_STREAM_OUT,
        "onVehicleStreamOut"
    );
    rpc_helper!(
        on_set_vehicle_position,
        VehiclePosition,
        SET_VEHICLE_POSITION,
        "onSetVehiclePosition"
    );
    rpc_helper!(
        on_set_vehicle_angle,
        VehicleAngle,
        SET_VEHICLE_ANGLE,
        "onSetVehicleAngle"
    );
    rpc_helper!(
        on_set_vehicle_health,
        VehicleHealth,
        SET_VEHICLE_HEALTH,
        "onSetVehicleHealth"
    );
    rpc_helper!(
        on_reset_player_money,
        (),
        RESET_PLAYER_MONEY,
        "onResetPlayerMoney"
    );
    rpc_helper!(
        on_reset_player_weapons,
        (),
        RESET_PLAYER_WEAPONS,
        "onResetPlayerWeapons"
    );
    rpc_helper!(
        on_disable_checkpoint,
        (),
        DISABLE_CHECKPOINT,
        "onDisableCheckpoint"
    );
    rpc_helper!(
        on_disable_race_checkpoint,
        (),
        DISABLE_RACE_CHECKPOINT,
        "onDisableRaceCheckpoint"
    );
    rpc_helper!(
        on_gamemode_restart,
        (),
        GAMEMODE_RESTART,
        "onGamemodeRestart"
    );
    rpc_helper!(
        on_stop_audio_stream,
        (),
        STOP_AUDIO_STREAM,
        "onStopAudioStream"
    );
    rpc_helper!(
        on_remove_player_from_vehicle,
        (),
        REMOVE_PLAYER_FROM_VEHICLE,
        "onRemovePlayerFromVehicle"
    );
    rpc_helper!(
        on_force_class_selection,
        (),
        FORCE_CLASS_SELECTION,
        "onForceClassSelection"
    );
    rpc_helper!(
        on_set_camera_behind,
        (),
        SET_CAMERA_BEHIND,
        "onSetCameraBehind"
    );

    fn decode_server_message(event: &mut Event<'_>) -> Result<ServerMessage, EventError> {
        Ok(ServerMessage {
            color: event.read_u32()?,
            text: event.read_string32(MAX_STRING32_BYTES)?,
        })
    }

    fn encode_server_message(value: ServerMessage) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.color);
        writer.string32(&value.text)?;
        Ok(writer.finish())
    }

    fn decode_game_text(event: &mut Event<'_>) -> Result<GameText, EventError> {
        Ok(GameText {
            style: event.read_u32()? as i32,
            time_ms: event.read_u32()? as i32,
            text: event.read_string32(MAX_STRING32_BYTES)?,
        })
    }

    fn encode_game_text(value: GameText) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.style as u32);
        writer.u32(value.time_ms as u32);
        writer.string32(&value.text)?;
        Ok(writer.finish())
    }

    fn decode_show_dialog(event: &mut Event<'_>) -> Result<ShowDialog, EventError> {
        Ok(ShowDialog {
            dialog_id: event.read_u16()?,
            style: event.read_u8()?,
            title: event.read_string8()?,
            button1: event.read_string8()?,
            button2: event.read_string8()?,
            text: event.read_encoded_string(MAX_ENCODED_STRING_BYTES + 1)?,
        })
    }

    fn encode_show_dialog(api: HostApi, value: ShowDialog) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.dialog_id);
        writer.u8(value.style);
        writer.string8(&value.title)?;
        writer.string8(&value.button1)?;
        writer.string8(&value.button2)?;
        writer.encoded_string(api, &value.text)?;
        Ok(writer.finish_bits())
    }

    fn decode_vector3(event: &mut Event<'_>) -> Result<Vector3, EventError> {
        Ok(Vector3 {
            x: event.read_f32()?,
            y: event.read_f32()?,
            z: event.read_f32()?,
        })
    }

    fn encode_vector3(value: Vector3) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.vector3(value);
        Ok(writer.finish())
    }

    fn decode_f32(event: &mut Event<'_>) -> Result<f32, EventError> {
        event.read_f32()
    }

    fn encode_f32(value: f32) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.f32(value);
        Ok(writer.finish())
    }

    fn decode_bool8(event: &mut Event<'_>) -> Result<bool, EventError> {
        Ok(event.read_u8()? != 0)
    }

    fn encode_bool8(value: bool) -> Result<Vec<u8>, EventError> {
        Ok(vec![u8::from(value)])
    }

    fn decode_play_sound(event: &mut Event<'_>) -> Result<PlaySound, EventError> {
        Ok(PlaySound {
            sound_id: event.read_u32()? as i32,
            position: decode_vector3(event)?,
        })
    }

    fn encode_play_sound(value: PlaySound) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.sound_id as u32);
        writer.vector3(value.position);
        Ok(writer.finish())
    }

    fn decode_checkpoint(event: &mut Event<'_>) -> Result<Checkpoint, EventError> {
        Ok(Checkpoint {
            position: decode_vector3(event)?,
            radius: event.read_f32()?,
        })
    }

    fn encode_checkpoint(value: Checkpoint) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.vector3(value.position);
        writer.f32(value.radius);
        Ok(writer.finish())
    }

    fn decode_chat_message(event: &mut Event<'_>) -> Result<ChatMessage, EventError> {
        Ok(ChatMessage {
            player_id: event.read_u16()?,
            text: event.read_string8()?,
        })
    }

    fn encode_chat_message(value: ChatMessage) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.string8(&value.text)?;
        Ok(writer.finish())
    }

    fn decode_chat_bubble(event: &mut Event<'_>) -> Result<ChatBubble, EventError> {
        Ok(ChatBubble {
            player_id: event.read_u16()?,
            color: event.read_u32()?,
            draw_distance: event.read_f32()?,
            duration_ms: event.read_u32()? as i32,
            text: event.read_string8()?,
        })
    }

    fn encode_chat_bubble(value: ChatBubble) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u32(value.color);
        writer.f32(value.draw_distance);
        writer.u32(value.duration_ms as u32);
        writer.string8(&value.text)?;
        Ok(writer.finish())
    }

    fn decode_player_join(event: &mut Event<'_>) -> Result<PlayerJoin, EventError> {
        Ok(PlayerJoin {
            player_id: event.read_u16()?,
            color: event.read_u32()?,
            is_npc: event.read_u8()? != 0,
            nickname: event.read_string8()?,
        })
    }

    fn encode_player_join(value: PlayerJoin) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u32(value.color);
        writer.u8(u8::from(value.is_npc));
        writer.string8(&value.nickname)?;
        Ok(writer.finish())
    }

    fn decode_player_quit(event: &mut Event<'_>) -> Result<PlayerQuit, EventError> {
        Ok(PlayerQuit {
            player_id: event.read_u16()?,
            reason: event.read_u8()?,
        })
    }

    fn encode_player_quit(value: PlayerQuit) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u8(value.reason);
        Ok(writer.finish())
    }

    fn decode_player_name(event: &mut Event<'_>) -> Result<PlayerName, EventError> {
        Ok(PlayerName {
            player_id: event.read_u16()?,
            name: event.read_string8()?,
            success: event.read_u8()? != 0,
        })
    }

    fn encode_player_name(value: PlayerName) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.string8(&value.name)?;
        writer.u8(u8::from(value.success));
        Ok(writer.finish())
    }

    fn decode_player_time(event: &mut Event<'_>) -> Result<PlayerTime, EventError> {
        Ok(PlayerTime {
            hour: event.read_u8()?,
            minute: event.read_u8()?,
        })
    }

    fn encode_player_time(value: PlayerTime) -> Result<Vec<u8>, EventError> {
        Ok(vec![value.hour, value.minute])
    }

    fn decode_world_bounds(event: &mut Event<'_>) -> Result<WorldBounds, EventError> {
        Ok(WorldBounds {
            max_x: event.read_f32()?,
            min_x: event.read_f32()?,
            max_y: event.read_f32()?,
            min_y: event.read_f32()?,
        })
    }

    fn encode_world_bounds(value: WorldBounds) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.f32(value.max_x);
        writer.f32(value.min_x);
        writer.f32(value.max_y);
        writer.f32(value.min_y);
        Ok(writer.finish())
    }

    fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
        Ok(event.read_u32()? as i32)
    }

    fn encode_i32(value: i32) -> Result<Vec<u8>, EventError> {
        Ok(value.to_le_bytes().to_vec())
    }

    fn decode_player_weapon(event: &mut Event<'_>) -> Result<PlayerWeapon, EventError> {
        Ok(PlayerWeapon {
            weapon_id: decode_i32(event)?,
            ammo: decode_i32(event)?,
        })
    }

    fn encode_player_weapon(value: PlayerWeapon) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.weapon_id as u32);
        writer.u32(value.ammo as u32);
        Ok(writer.finish())
    }

    fn decode_u8(event: &mut Event<'_>) -> Result<u8, EventError> {
        event.read_u8()
    }

    fn encode_u8(value: u8) -> Result<Vec<u8>, EventError> {
        Ok(vec![value])
    }

    fn decode_player_team(event: &mut Event<'_>) -> Result<PlayerTeam, EventError> {
        Ok(PlayerTeam {
            player_id: event.read_u16()?,
            team_id: event.read_u8()?,
        })
    }

    fn encode_player_team(value: PlayerTeam) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u8(value.team_id);
        Ok(writer.finish())
    }

    fn decode_put_player_in_vehicle(
        event: &mut Event<'_>,
    ) -> Result<PutPlayerInVehicle, EventError> {
        Ok(PutPlayerInVehicle {
            vehicle_id: event.read_u16()?,
            seat_id: event.read_u8()?,
        })
    }

    fn encode_put_player_in_vehicle(value: PutPlayerInVehicle) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.u8(value.seat_id);
        Ok(writer.finish())
    }

    fn decode_u16(event: &mut Event<'_>) -> Result<u16, EventError> {
        event.read_u16()
    }

    fn encode_u16(value: u16) -> Result<Vec<u8>, EventError> {
        Ok(value.to_le_bytes().to_vec())
    }

    fn decode_vehicle_position(event: &mut Event<'_>) -> Result<VehiclePosition, EventError> {
        Ok(VehiclePosition {
            vehicle_id: event.read_u16()?,
            position: decode_vector3(event)?,
        })
    }

    fn encode_vehicle_position(value: VehiclePosition) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.vector3(value.position);
        Ok(writer.finish())
    }

    fn decode_vehicle_angle(event: &mut Event<'_>) -> Result<VehicleAngle, EventError> {
        Ok(VehicleAngle {
            vehicle_id: event.read_u16()?,
            angle: event.read_f32()?,
        })
    }

    fn encode_vehicle_angle(value: VehicleAngle) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.f32(value.angle);
        Ok(writer.finish())
    }

    fn decode_vehicle_health(event: &mut Event<'_>) -> Result<VehicleHealth, EventError> {
        Ok(VehicleHealth {
            vehicle_id: event.read_u16()?,
            health: event.read_f32()?,
        })
    }

    fn encode_vehicle_health(value: VehicleHealth) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.f32(value.health);
        Ok(writer.finish())
    }

    fn decode_empty(_event: &mut Event<'_>) -> Result<(), EventError> {
        Ok(())
    }

    fn encode_empty(_value: ()) -> Result<Vec<u8>, EventError> {
        Ok(Vec::new())
    }
}

/// Outgoing client-to-server RPC helpers.
pub mod outgoing {
    use super::*;

    /// MoonLoader's `onSendDialogResponse` payload (RPC 62).
    #[derive(Clone, Debug, PartialEq)]
    pub struct DialogResponse {
        pub dialog_id: u16,
        pub button: u8,
        pub list_item: u16,
        pub input: Vec<u8>,
    }

    /// MoonLoader's `onSendEnterVehicle` payload (RPC 26).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct EnterVehicle {
        pub vehicle_id: u16,
        pub passenger: bool,
    }

    /// MoonLoader's `onSendDeathNotification` payload (RPC 53).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DeathNotification {
        pub reason: u8,
        pub killer_id: u16,
    }

    /// MoonLoader's `onSendClickPlayer` payload (RPC 23).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ClickPlayer {
        pub player_id: u16,
        pub source: u8,
    }

    /// The `onSendChat` descriptor.
    pub const SEND_CHAT: Rpc<Vec<u8>> = Rpc::new(101, decode_string8, encode_string8);
    /// The `onSendCommand` descriptor.
    pub const SEND_COMMAND: Rpc<Vec<u8>> = Rpc::new(50, decode_string32, encode_string32);
    /// The `onSendDialogResponse` descriptor.
    pub const SEND_DIALOG_RESPONSE: Rpc<DialogResponse> =
        Rpc::new(62, decode_dialog_response, encode_dialog_response);
    /// The `onSendEnterVehicle` descriptor.
    pub const SEND_ENTER_VEHICLE: Rpc<EnterVehicle> =
        Rpc::new(26, decode_enter_vehicle, encode_enter_vehicle);
    /// The `onSendExitVehicle` descriptor.
    pub const SEND_EXIT_VEHICLE: Rpc<u16> = Rpc::new(154, decode_u16, encode_u16);
    /// The `onSendSpawn` descriptor.
    pub const SEND_SPAWN: Rpc<()> = Rpc::new(52, decode_empty, encode_empty);
    /// The `onSendDeathNotification` descriptor.
    pub const SEND_DEATH_NOTIFICATION: Rpc<DeathNotification> =
        Rpc::new(53, decode_death_notification, encode_death_notification);
    /// The `onSendMapMarker` descriptor.
    pub const SEND_MAP_MARKER: Rpc<Vector3> = Rpc::new(119, decode_vector3, encode_vector3);
    /// The `onSendClickPlayer` descriptor.
    pub const SEND_CLICK_PLAYER: Rpc<ClickPlayer> =
        Rpc::new(23, decode_click_player, encode_click_player);
    /// The `onSendInteriorChange` descriptor.
    pub const SEND_INTERIOR_CHANGE: Rpc<u8> = Rpc::new(118, decode_u8, encode_u8);
    /// The `onSendRequestClass` descriptor.
    pub const SEND_REQUEST_CLASS: Rpc<i32> = Rpc::new(128, decode_i32, encode_i32);
    /// The `onSendRequestSpawn` descriptor.
    pub const SEND_REQUEST_SPAWN: Rpc<()> = Rpc::new(129, decode_empty, encode_empty);
    /// The `onSendMenuSelect` descriptor.
    pub const SEND_MENU_SELECT: Rpc<u8> = Rpc::new(132, decode_u8, encode_u8);
    /// The `onSendVehicleDestroyed` descriptor.
    pub const SEND_VEHICLE_DESTROYED: Rpc<u16> = Rpc::new(136, decode_u16, encode_u16);
    /// The `onSendClickTextDraw` descriptor.
    pub const SEND_CLICK_TEXT_DRAW: Rpc<u16> = Rpc::new(83, decode_u16, encode_u16);
    /// The `onSendUpdateScoresAndPings` descriptor.
    pub const SEND_UPDATE_SCORES_AND_PINGS: Rpc<()> = Rpc::new(155, decode_empty, encode_empty);

    /// Handles `onSendChat` from an outgoing raw RPC callback.
    ///
    /// # Safety
    ///
    /// See [`super::handle`].
    pub unsafe fn on_send_chat(
        api: HostApi,
        raw: *mut RakRsEventV1,
        handler: impl FnOnce(Vec<u8>) -> RpcAction<Vec<u8>>,
    ) -> Result<RakRsHookAction, EventError> {
        unsafe { handle(api, raw, SEND_CHAT, handler) }
    }

    macro_rules! rpc_helper {
        ($name:ident, $value:ty, $rpc:ident, $event_name:literal) => {
            #[doc = concat!("Handles MoonLoader's `", $event_name, "` from an outgoing raw RPC callback.")]
            ///
            /// # Safety
            ///
            /// See [`super::handle`].
            pub unsafe fn $name(
                api: HostApi,
                raw: *mut RakRsEventV1,
                handler: impl FnOnce($value) -> RpcAction<$value>,
            ) -> Result<RakRsHookAction, EventError> {
                unsafe { handle(api, raw, $rpc, handler) }
            }
        };
    }

    rpc_helper!(on_send_command, Vec<u8>, SEND_COMMAND, "onSendCommand");
    rpc_helper!(
        on_send_dialog_response,
        DialogResponse,
        SEND_DIALOG_RESPONSE,
        "onSendDialogResponse"
    );
    rpc_helper!(
        on_send_enter_vehicle,
        EnterVehicle,
        SEND_ENTER_VEHICLE,
        "onSendEnterVehicle"
    );
    rpc_helper!(
        on_send_exit_vehicle,
        u16,
        SEND_EXIT_VEHICLE,
        "onSendExitVehicle"
    );
    rpc_helper!(on_send_spawn, (), SEND_SPAWN, "onSendSpawn");
    rpc_helper!(
        on_send_death_notification,
        DeathNotification,
        SEND_DEATH_NOTIFICATION,
        "onSendDeathNotification"
    );
    rpc_helper!(
        on_send_map_marker,
        Vector3,
        SEND_MAP_MARKER,
        "onSendMapMarker"
    );
    rpc_helper!(
        on_send_click_player,
        ClickPlayer,
        SEND_CLICK_PLAYER,
        "onSendClickPlayer"
    );
    rpc_helper!(
        on_send_interior_change,
        u8,
        SEND_INTERIOR_CHANGE,
        "onSendInteriorChange"
    );
    rpc_helper!(
        on_send_request_class,
        i32,
        SEND_REQUEST_CLASS,
        "onSendRequestClass"
    );
    rpc_helper!(
        on_send_request_spawn,
        (),
        SEND_REQUEST_SPAWN,
        "onSendRequestSpawn"
    );
    rpc_helper!(
        on_send_menu_select,
        u8,
        SEND_MENU_SELECT,
        "onSendMenuSelect"
    );
    rpc_helper!(
        on_send_vehicle_destroyed,
        u16,
        SEND_VEHICLE_DESTROYED,
        "onSendVehicleDestroyed"
    );
    rpc_helper!(
        on_send_click_text_draw,
        u16,
        SEND_CLICK_TEXT_DRAW,
        "onSendClickTextDraw"
    );
    rpc_helper!(
        on_send_update_scores_and_pings,
        (),
        SEND_UPDATE_SCORES_AND_PINGS,
        "onSendUpdateScoresAndPings"
    );

    fn decode_string8(event: &mut Event<'_>) -> Result<Vec<u8>, EventError> {
        event.read_string8()
    }

    fn encode_string8(value: Vec<u8>) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.string8(&value)?;
        Ok(writer.finish())
    }

    fn decode_string32(event: &mut Event<'_>) -> Result<Vec<u8>, EventError> {
        event.read_string32(MAX_STRING32_BYTES)
    }

    fn encode_string32(value: Vec<u8>) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.string32(&value)?;
        Ok(writer.finish())
    }

    fn decode_dialog_response(event: &mut Event<'_>) -> Result<DialogResponse, EventError> {
        Ok(DialogResponse {
            dialog_id: event.read_u16()?,
            button: event.read_u8()?,
            list_item: event.read_u16()?,
            input: event.read_string8()?,
        })
    }

    fn encode_dialog_response(value: DialogResponse) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.dialog_id);
        writer.u8(value.button);
        writer.u16(value.list_item);
        writer.string8(&value.input)?;
        Ok(writer.finish())
    }

    fn decode_enter_vehicle(event: &mut Event<'_>) -> Result<EnterVehicle, EventError> {
        Ok(EnterVehicle {
            vehicle_id: event.read_u16()?,
            passenger: event.read_u8()? != 0,
        })
    }

    fn encode_enter_vehicle(value: EnterVehicle) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.u8(u8::from(value.passenger));
        Ok(writer.finish())
    }

    fn decode_click_player(event: &mut Event<'_>) -> Result<ClickPlayer, EventError> {
        Ok(ClickPlayer {
            player_id: event.read_u16()?,
            source: event.read_u8()?,
        })
    }

    fn encode_click_player(value: ClickPlayer) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u8(value.source);
        Ok(writer.finish())
    }

    fn decode_u8(event: &mut Event<'_>) -> Result<u8, EventError> {
        event.read_u8()
    }

    fn encode_u8(value: u8) -> Result<Vec<u8>, EventError> {
        Ok(vec![value])
    }

    fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
        Ok(event.read_u32()? as i32)
    }

    fn encode_i32(value: i32) -> Result<Vec<u8>, EventError> {
        Ok(value.to_le_bytes().to_vec())
    }

    fn decode_u16(event: &mut Event<'_>) -> Result<u16, EventError> {
        event.read_u16()
    }

    fn encode_u16(value: u16) -> Result<Vec<u8>, EventError> {
        Ok(value.to_le_bytes().to_vec())
    }

    fn decode_empty(_event: &mut Event<'_>) -> Result<(), EventError> {
        Ok(())
    }

    fn encode_empty(_value: ()) -> Result<Vec<u8>, EventError> {
        Ok(Vec::new())
    }

    fn decode_death_notification(event: &mut Event<'_>) -> Result<DeathNotification, EventError> {
        Ok(DeathNotification {
            reason: event.read_u8()?,
            killer_id: event.read_u16()?,
        })
    }

    fn encode_death_notification(value: DeathNotification) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u8(value.reason);
        writer.u16(value.killer_id);
        Ok(writer.finish())
    }

    fn decode_vector3(event: &mut Event<'_>) -> Result<Vector3, EventError> {
        Ok(Vector3 {
            x: event.read_f32()?,
            y: event.read_f32()?,
            z: event.read_f32()?,
        })
    }

    fn encode_vector3(value: Vector3) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.vector3(value);
        Ok(writer.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_writer_preserves_partial_bit_lengths() {
        let mut writer = PayloadWriter::new();
        writer.u8(0xA5);
        writer.bits(&[0b1100_0000], 3);
        let payload = writer.finish_bits();

        assert_eq!(payload.len_bits(), 11);
        assert_eq!(payload.as_bytes(), &[0xA5, 0b1100_0000]);
    }

    #[test]
    fn encoded_payload_rejects_bits_outside_its_buffer() {
        assert!(matches!(
            EncodedPayload::from_bits(vec![0], 9),
            Err(EventError::InvalidBitLength {
                bit_len: 9,
                byte_len: 1
            })
        ));
    }
}
