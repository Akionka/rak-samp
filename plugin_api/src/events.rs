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

/// A two-dimensional SA-MP coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
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
    /// A fixed-layout helper received a payload with an unexpected bit length.
    UnexpectedBitLength { bit_len: usize, expected: usize },
    /// A tagged protocol field contained a value not defined by the R1 layout.
    InvalidDiscriminant { value: u8 },
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
            Self::UnexpectedBitLength { bit_len, expected } => write!(
                formatter,
                "event has {bit_len} bits, but the fixed layout requires {expected}"
            ),
            Self::InvalidDiscriminant { value } => {
                write!(formatter, "invalid SA-MP R1 discriminant {value}")
            }
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

    fn i16(&mut self, value: i16) {
        self.u16(value as u16);
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

    fn bool(&mut self, value: bool) {
        self.bit(value);
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
        self.encoded_string_with_limit(api, value, MAX_ENCODED_STRING_BYTES)
    }

    fn encoded_string_with_limit(
        &mut self,
        api: HostApi,
        value: &[u8],
        limit: usize,
    ) -> Result<(), EventError> {
        if value.len() > limit {
            return Err(EventError::ValueOutOfRange {
                value: value.len(),
                maximum: limit,
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

/// A typed packet descriptor with its RakNet packet ID and read/write layout.
///
/// Packets and RPCs share the callback ABI and replacement behavior. This alias makes packet
/// helpers explicit at their call sites without creating another hook or subscription mechanism.
pub type Packet<T> = Rpc<T>;

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
        if event.remaining_bits() != 0 {
            return Err(EventError::UnexpectedBitLength {
                bit_len: event.remaining_bits(),
                expected: 0,
            });
        }
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

    /// MoonLoader's `onSetPlayerSkin` payload (RPC 153).
    ///
    /// Both fields are signed 32-bit values on the wire. They are kept as-is so custom or
    /// otherwise unknown skin IDs can be observed or replaced without lossy validation.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerSkin {
        pub player_id: i32,
        pub skin_id: i32,
    }

    /// MoonLoader's `onSetRaceCheckpoint` payload (RPC 38).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct RaceCheckpoint {
        pub checkpoint_type: u8,
        pub position: Vector3,
        pub next_position: Vector3,
        pub size: f32,
    }

    /// MoonLoader's `onPlayAudioStream` payload (RPC 41).
    #[derive(Clone, Debug, PartialEq)]
    pub struct AudioStream {
        pub url: Vec<u8>,
        pub position: Vector3,
        pub radius: f32,
        pub use_position: bool,
    }

    /// MoonLoader's `onSetObjectPosition` payload (RPC 45).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ObjectPosition {
        pub object_id: u16,
        pub position: Vector3,
    }

    /// MoonLoader's `onSetObjectRotation` payload (RPC 46).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ObjectRotation {
        pub object_id: u16,
        pub rotation: Vector3,
    }

    /// MoonLoader's `onPlayerDeathNotification` payload (RPC 55).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerDeathNotification {
        pub killer_id: u16,
        pub killed_id: u16,
        pub reason: u8,
    }

    /// MoonLoader's `onSetMapIcon` payload (RPC 56).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MapIcon {
        pub icon_id: u8,
        pub position: Vector3,
        pub icon_type: u8,
        pub color: i32,
        pub style: u8,
    }

    /// MoonLoader's `onRemoveVehicleComponent` payload (RPC 57).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleComponent {
        pub vehicle_id: u16,
        pub component_id: u16,
    }

    /// MoonLoader's `onLinkVehicleToInterior` payload (RPC 65).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleInterior {
        pub vehicle_id: u16,
        pub interior_id: u8,
    }

    /// MoonLoader's `onSetPlayerColor` payload (RPC 72).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerColor {
        pub player_id: u16,
        pub color: i32,
    }

    /// MoonLoader's `onSetPlayerSkillLevel` payload (RPC 34).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerSkill {
        pub player_id: u16,
        pub skill: i32,
        pub level: u16,
    }

    /// MoonLoader's `onRemoveBuilding` payload (RPC 43).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct RemoveBuilding {
        pub model_id: i32,
        pub position: Vector3,
        pub radius: f32,
    }

    /// MoonLoader's `onAttachObjectToPlayer` payload (RPC 75).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct AttachObjectToPlayer {
        pub object_id: u16,
        pub player_id: u16,
        pub offsets: Vector3,
        pub rotation: Vector3,
    }

    /// MoonLoader's `onCreateExplosion` payload (RPC 79).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Explosion {
        pub position: Vector3,
        pub style: i32,
        pub radius: f32,
    }

    /// MoonLoader's `onShowPlayerNameTag` payload (RPC 80).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerNameTag {
        pub player_id: u16,
        pub show: bool,
    }

    /// MoonLoader's `onSetPlayerFightingStyle` payload (RPC 89).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerFightingStyle {
        pub player_id: u16,
        pub style_id: u8,
    }

    /// MoonLoader's `onSetVehicleVelocity` payload (RPC 91).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleVelocity {
        pub turn: bool,
        pub velocity: Vector3,
    }

    /// MoonLoader's `onCreatePickup` payload (RPC 95).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Pickup {
        pub id: i32,
        pub model: i32,
        pub pickup_type: i32,
        pub position: Vector3,
    }

    /// MoonLoader's `onMoveObject` payload (RPC 99).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MoveObject {
        pub object_id: u16,
        pub from_position: Vector3,
        pub destination: Vector3,
        pub speed: f32,
        pub rotation: Vector3,
    }

    /// MoonLoader's `onTextDrawSetString` payload (RPC 105).
    #[derive(Clone, Debug, PartialEq)]
    pub struct TextDrawString {
        pub textdraw_id: u16,
        pub text: Vec<u8>,
    }

    /// MoonLoader's `onCreateGangZone` payload (RPC 108).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct GangZone {
        pub zone_id: u16,
        pub square_start: Vector2,
        pub square_end: Vector2,
        pub color: i32,
    }

    /// MoonLoader's `onSetVehicleNumberPlate` payload (RPC 123).
    #[derive(Clone, Debug, PartialEq)]
    pub struct VehicleNumberPlate {
        pub vehicle_id: u16,
        pub text: Vec<u8>,
    }

    /// MoonLoader's `onSpectatePlayer` / `onSpectateVehicle` payload.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Spectate {
        pub target_id: u16,
        pub camera_type: u8,
    }

    /// MoonLoader's `onSetWeaponAmmo` payload (RPC 145).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct WeaponAmmo {
        pub weapon_id: u8,
        pub ammo: u16,
    }

    /// MoonLoader's `onAttachTrailerToVehicle` payload (RPC 148).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TrailerAttachment {
        pub trailer_id: u16,
        pub vehicle_id: u16,
    }

    /// MoonLoader's `onSetCameraLookAt` payload (RPC 158).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct CameraLookAt {
        pub position: Vector3,
        pub cut_type: u8,
    }

    /// MoonLoader's `onSetVehicleParams` payload (RPC 161).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleParams {
        pub vehicle_id: u16,
        pub objective: bool,
        pub doors_locked: bool,
    }

    /// MoonLoader's `onPlayerEnterVehicle` payload (RPC 26).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerEnterVehicle {
        pub player_id: u16,
        pub vehicle_id: u16,
        pub passenger: bool,
    }

    /// MoonLoader's `onPlayerExitVehicle` payload (RPC 154).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerExitVehicle {
        pub player_id: u16,
        pub vehicle_id: u16,
    }

    /// MoonLoader's `onClientCheck` payload (RPC 103).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ClientCheck {
        pub request_type: u8,
        pub subject: i32,
        pub offset: u16,
        pub length: u16,
    }

    /// MoonLoader's `onVehicleTuningNotification` payload (RPC 96).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleTuningNotification {
        pub player_id: u16,
        pub event: i32,
        pub vehicle_id: i32,
        pub param1: i32,
        pub param2: i32,
    }

    /// MoonLoader's `onVehicleDamageStatusUpdate` payload (RPC 106).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleDamageStatus {
        pub vehicle_id: u16,
        pub panel_damage: i32,
        pub door_damage: i32,
        pub lights: u8,
        pub tires: u8,
    }

    /// MoonLoader's `onSetVehicleParamsEx` payload (RPC 24).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct VehicleParamsEx {
        pub vehicle_id: u16,
        pub params: [u8; 8],
        pub doors: [u8; 4],
        pub windows: [u8; 4],
    }

    /// MoonLoader's `onCreateActor` payload (RPC 171).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Actor {
        pub actor_id: u16,
        pub skin_id: i32,
        pub position: Vector3,
        pub rotation: f32,
        pub health: f32,
    }

    /// MoonLoader's `onSetActorFacingAngle` payload (RPC 175).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ActorAngle {
        pub actor_id: u16,
        pub angle: f32,
    }

    /// MoonLoader's `onSetActorPos` payload (RPC 176).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ActorPosition {
        pub actor_id: u16,
        pub position: Vector3,
    }

    /// MoonLoader's `onSetActorHealth` payload (RPC 178).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ActorHealth {
        pub actor_id: u16,
        pub health: f32,
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

    /// The maximum number of rows that R1 menus can expose per column.
    pub const MAX_MENU_ROWS: usize = 12;
    /// The R1 client accepts at most two menu columns.
    pub const MAX_MENU_COLUMNS: usize = 2;
    /// SA-MP objects expose at most sixteen material slots.
    pub const MAX_OBJECT_MATERIALS: usize = 16;
    /// The server can send at most one score/ping entry for each R1 player slot.
    pub const MAX_SCORE_PING_ENTRIES: usize = 1_000;
    /// R1's material-text codec accepts at most 2,047 payload bytes.
    pub const MAX_OBJECT_MATERIAL_TEXT_BYTES: usize = 2_047;

    /// Settings supplied by `onInitGame` (RPC 139).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct GameSettings {
        pub zone_names: bool,
        pub use_cj_walk: bool,
        pub allow_weapons: bool,
        pub limit_global_chat_radius: bool,
        pub global_chat_radius: f32,
        pub stunt_bonus: bool,
        pub nametag_draw_distance: f32,
        pub disable_enter_exits: bool,
        pub nametag_los: bool,
        pub tire_popping: bool,
        pub classes_available: i32,
        pub show_player_tags: bool,
        pub player_markers_mode: i32,
        pub world_time: u8,
        pub world_weather: u8,
        pub gravity: f32,
        pub lan_mode: bool,
        pub death_money_drop: i32,
        pub instagib: bool,
        pub normal_onfoot_send_rate: i32,
        pub normal_incar_send_rate: i32,
        pub normal_firing_send_rate: i32,
        pub send_multiplier: i32,
        pub lag_compensation_mode: i32,
        pub vehicle_friendly_fire: bool,
    }

    /// MoonLoader's `onInitGame` payload (RPC 139).
    #[derive(Clone, Debug, PartialEq)]
    pub struct InitGame {
        pub player_id: u16,
        pub host_name: Vec<u8>,
        pub settings: GameSettings,
        /// R1's 212 vehicle-model capability flags, retained byte-for-byte.
        pub vehicle_models: [u8; 212],
    }

    /// A class preview or spawn definition shared by the class and spawn RPCs.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct SpawnInfo {
        pub team: u8,
        pub skin: i32,
        /// R1 serializes this byte between the skin and position. Its purpose is unknown.
        pub unused: u8,
        pub position: Vector3,
        pub rotation: f32,
        pub weapons: [i32; 3],
        pub ammo: [i32; 3],
    }

    /// MoonLoader's `onRequestClassResponse` payload (RPC 128).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct RequestClassResponse {
        pub can_spawn: bool,
        pub spawn: SpawnInfo,
    }

    /// MoonLoader's `onPlayerStreamIn` payload (RPC 32).
    ///
    /// R1 sends one skill level for each of the eleven weapon-skill categories after the fixed
    /// player data.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerStreamIn {
        pub player_id: u16,
        pub team: u8,
        pub model: i32,
        pub position: Vector3,
        pub rotation: f32,
        pub color: i32,
        pub fighting_style: u8,
        pub weapon_skill_levels: [u16; 11],
    }

    /// MoonLoader's `onCreate3DText` payload (RPC 36).
    #[derive(Clone, Debug, PartialEq)]
    pub struct TextLabel3D {
        pub id: u16,
        pub color: i32,
        pub position: Vector3,
        pub distance: f32,
        pub test_los: bool,
        pub attached_player_id: u16,
        pub attached_vehicle_id: u16,
        pub text: Vec<u8>,
    }

    /// Object attachment fields that are present only when an object has an attachment target.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ObjectAttachment {
        pub offsets: Vector3,
        pub rotation: Vector3,
        pub sync_rotation: bool,
    }

    /// A texture-based object material. The field order is the R1 wire order.
    #[derive(Clone, Debug, PartialEq)]
    pub struct TextureMaterial {
        pub material_id: u8,
        pub model_id: u16,
        pub library_name: Vec<u8>,
        pub texture_name: Vec<u8>,
        pub color: i32,
    }

    /// A text-based object material. The encoded text deliberately remains bytes.
    #[derive(Clone, Debug, PartialEq)]
    pub struct TextMaterial {
        pub material_id: u8,
        pub material_size: u8,
        pub font_name: Vec<u8>,
        pub font_size: u8,
        pub bold: u8,
        pub font_color: i32,
        pub background_color: i32,
        pub align: u8,
        pub text: Vec<u8>,
    }

    /// One object material, preserving texture/text ordering during a replacement.
    #[derive(Clone, Debug, PartialEq)]
    pub enum ObjectMaterial {
        Texture(TextureMaterial),
        Text(TextMaterial),
    }

    /// MoonLoader's `onCreateObject` payload (RPC 44).
    #[derive(Clone, Debug, PartialEq)]
    pub struct Object {
        pub object_id: u16,
        pub model_id: i32,
        pub position: Vector3,
        pub rotation: Vector3,
        pub draw_distance: f32,
        pub no_camera_collision: bool,
        pub attach_to_vehicle_id: u16,
        pub attach_to_object_id: u16,
        pub attachment: Option<ObjectAttachment>,
        /// R1's original material-count field, retained independently of the decoded sequence.
        pub textures_count: u8,
        pub materials: Vec<ObjectMaterial>,
    }

    /// One update from RPC 84, which can carry either material variant.
    #[derive(Clone, Debug, PartialEq)]
    pub struct ObjectMaterialUpdate {
        pub object_id: u16,
        pub material: ObjectMaterial,
    }

    /// One column in an R1 menu initialization payload.
    #[derive(Clone, Debug, PartialEq)]
    pub struct MenuColumn {
        pub width: f32,
        pub title: [u8; 32],
        pub rows: Vec<[u8; 32]>,
    }

    /// MoonLoader's `onInitMenu` payload (RPC 76).
    #[derive(Clone, Debug, PartialEq)]
    pub struct InitMenu {
        pub menu_id: u8,
        pub two_columns: bool,
        pub title: [u8; 32],
        pub position: Vector2,
        pub columns: Vec<MenuColumn>,
        pub rows: [i32; MAX_MENU_ROWS],
        pub menu: bool,
    }

    /// MoonLoader's `onInterpolateCamera` payload (RPC 82).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct InterpolateCamera {
        pub set_position: bool,
        pub from_position: Vector3,
        pub destination: Vector3,
        pub time_ms: i32,
        pub mode: u8,
    }

    /// MoonLoader's `onToggleSelectTextDraw` payload (RPC 83).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ToggleSelectTextDraw {
        pub enabled: bool,
        pub hover_color: i32,
    }

    /// MoonLoader's player or actor animation payload.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Animation {
        pub animation_library: Vec<u8>,
        pub animation_name: Vec<u8>,
        pub frame_delta: f32,
        pub looped: bool,
        pub lock_x: bool,
        pub lock_y: bool,
        pub freeze: bool,
        pub time: i32,
    }

    /// MoonLoader's `onApplyPlayerAnimation` payload (RPC 86).
    #[derive(Clone, Debug, PartialEq)]
    pub struct PlayerAnimation {
        pub player_id: u16,
        pub animation: Animation,
    }

    /// MoonLoader's `onApplyActorAnimation` payload (RPC 173).
    #[derive(Clone, Debug, PartialEq)]
    pub struct ActorAnimation {
        pub actor_id: u16,
        pub animation: Animation,
    }

    /// MoonLoader's `onPlayCrimeReport` payload (RPC 112).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct CrimeReport {
        pub suspect_id: u16,
        pub in_vehicle: bool,
        pub vehicle_model: i32,
        pub vehicle_color: i32,
        pub crime: i32,
        pub coordinates: Vector3,
    }

    /// An attached player object, present only when `create` is true.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct AttachedObject {
        pub model_id: i32,
        pub bone: i32,
        pub offset: Vector3,
        pub rotation: Vector3,
        pub scale: Vector3,
        pub color1: i32,
        pub color2: i32,
    }

    /// MoonLoader's `onSetPlayerAttachedObject` payload (RPC 113).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerAttachedObject {
        pub player_id: u16,
        pub index: i32,
        pub object: Option<AttachedObject>,
    }

    /// MoonLoader's `onEnterEditObject` payload (RPC 117).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EnterEditObject {
        pub player_object: bool,
        pub object_id: u16,
    }

    /// The R1 textdraw shape and content sent by `onShowTextDraw`.
    #[derive(Clone, Debug, PartialEq)]
    pub struct TextDraw {
        pub flags: u8,
        pub letter_width: f32,
        pub letter_height: f32,
        pub letter_color: i32,
        pub line_width: f32,
        pub line_height: f32,
        pub box_color: i32,
        pub shadow: u8,
        pub outline: u8,
        pub background_color: i32,
        pub style: u8,
        pub selectable: u8,
        pub position: Vector2,
        pub model_id: u16,
        pub rotation: Vector3,
        pub zoom: f32,
        pub color1: i16,
        pub color2: i16,
        pub text: Vec<u8>,
    }

    /// MoonLoader's `onShowTextDraw` payload (RPC 134).
    #[derive(Clone, Debug, PartialEq)]
    pub struct ShowTextDraw {
        pub textdraw_id: u16,
        pub textdraw: TextDraw,
    }

    /// One score and ping record sent by RPC 155.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ScorePing {
        pub player_id: u16,
        pub score: i32,
        pub ping: i32,
    }

    /// MoonLoader's `onUpdateScoresAndPings` payload (RPC 155), retained in wire order.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct ScoresAndPings {
        pub entries: Vec<ScorePing>,
    }

    /// MoonLoader's `onVehicleStreamIn` vehicle data (RPC 164).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct StreamedVehicle {
        pub model: i32,
        pub position: Vector3,
        pub rotation: f32,
        pub body_color1: u8,
        pub body_color2: u8,
        pub health: f32,
        pub interior_id: u8,
        pub door_damage_status: i32,
        pub panel_damage_status: i32,
        pub light_damage_status: u8,
        pub tire_damage_status: u8,
        pub add_siren: u8,
        pub mod_slots: [u8; 14],
        pub paint_job: u8,
        pub interior_color1: i32,
        pub interior_color2: i32,
    }

    /// MoonLoader's `onVehicleStreamIn` payload (RPC 164).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleStreamIn {
        pub vehicle_id: u16,
        pub vehicle: StreamedVehicle,
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
    /// The `onSetPlayerSkin` descriptor.
    pub const SET_PLAYER_SKIN: Rpc<PlayerSkin> =
        Rpc::new(153, decode_player_skin, encode_player_skin);
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
    /// The `onCancelEdit` descriptor.
    pub const CANCEL_EDIT: Rpc<()> = Rpc::new(28, decode_empty, encode_empty);
    /// The `onSetToggleClock` descriptor.
    pub const SET_TOGGLE_CLOCK: Rpc<bool> = Rpc::new(30, decode_bool8, encode_bool8);
    /// The `onSetPlayerDrunk` descriptor.
    pub const SET_PLAYER_DRUNK: Rpc<i32> = Rpc::new(35, decode_i32, encode_i32);
    /// The `onSetRaceCheckpoint` descriptor.
    pub const SET_RACE_CHECKPOINT: Rpc<RaceCheckpoint> =
        Rpc::new(38, decode_race_checkpoint, encode_race_checkpoint);
    /// The `onPlayAudioStream` descriptor.
    pub const PLAY_AUDIO_STREAM: Rpc<AudioStream> =
        Rpc::new(41, decode_audio_stream, encode_audio_stream);
    /// The `onSetObjectPosition` descriptor.
    pub const SET_OBJECT_POSITION: Rpc<ObjectPosition> =
        Rpc::new(45, decode_object_position, encode_object_position);
    /// The `onSetObjectRotation` descriptor.
    pub const SET_OBJECT_ROTATION: Rpc<ObjectRotation> =
        Rpc::new(46, decode_object_rotation, encode_object_rotation);
    /// The `onDestroyObject` descriptor.
    pub const DESTROY_OBJECT: Rpc<u16> = Rpc::new(47, decode_u16, encode_u16);
    /// The `onPlayerDeathNotification` descriptor.
    pub const PLAYER_DEATH_NOTIFICATION: Rpc<PlayerDeathNotification> = Rpc::new(
        55,
        decode_player_death_notification,
        encode_player_death_notification,
    );
    /// The `onSetMapIcon` descriptor.
    pub const SET_MAP_ICON: Rpc<MapIcon> = Rpc::new(56, decode_map_icon, encode_map_icon);
    /// The `onRemoveVehicleComponent` descriptor.
    pub const REMOVE_VEHICLE_COMPONENT: Rpc<VehicleComponent> =
        Rpc::new(57, decode_vehicle_component, encode_vehicle_component);
    /// The `onRemove3DTextLabel` descriptor.
    pub const REMOVE_3D_TEXT_LABEL: Rpc<u16> = Rpc::new(58, decode_u16, encode_u16);
    /// The `onUpdateGlobalTimer` descriptor.
    pub const UPDATE_GLOBAL_TIMER: Rpc<i32> = Rpc::new(60, decode_i32, encode_i32);
    /// The `onDestroyPickup` descriptor.
    pub const DESTROY_PICKUP: Rpc<i32> = Rpc::new(63, decode_i32, encode_i32);
    /// The `onLinkVehicleToInterior` descriptor.
    pub const LINK_VEHICLE_TO_INTERIOR: Rpc<VehicleInterior> =
        Rpc::new(65, decode_vehicle_interior, encode_vehicle_interior);
    /// The `onSetPlayerColor` descriptor.
    pub const SET_PLAYER_COLOR: Rpc<PlayerColor> =
        Rpc::new(72, decode_player_color, encode_player_color);
    /// The `onRequestSpawnResponse` descriptor.
    pub const REQUEST_SPAWN_RESPONSE: Rpc<bool> = Rpc::new(129, decode_bool8, encode_bool8);
    /// The `onSetShopName` descriptor. The protocol field is exactly 32 bytes.
    pub const SET_SHOP_NAME: Rpc<[u8; 32]> =
        Rpc::new(33, decode_fixed_string32, encode_fixed_string32);
    /// The `onSetPlayerSkillLevel` descriptor.
    pub const SET_PLAYER_SKILL_LEVEL: Rpc<PlayerSkill> =
        Rpc::new(34, decode_player_skill, encode_player_skill);
    /// The `onRemoveBuilding` descriptor.
    pub const REMOVE_BUILDING: Rpc<RemoveBuilding> =
        Rpc::new(43, decode_remove_building, encode_remove_building);
    /// The `onAttachObjectToPlayer` descriptor.
    pub const ATTACH_OBJECT_TO_PLAYER: Rpc<AttachObjectToPlayer> = Rpc::new(
        75,
        decode_attach_object_to_player,
        encode_attach_object_to_player,
    );
    /// The `onShowMenu` descriptor.
    pub const SHOW_MENU: Rpc<u8> = Rpc::new(77, decode_u8, encode_u8);
    /// The `onHideMenu` descriptor.
    pub const HIDE_MENU: Rpc<u8> = Rpc::new(78, decode_u8, encode_u8);
    /// The `onCreateExplosion` descriptor.
    pub const CREATE_EXPLOSION: Rpc<Explosion> = Rpc::new(79, decode_explosion, encode_explosion);
    /// The `onShowPlayerNameTag` descriptor.
    pub const SHOW_PLAYER_NAME_TAG: Rpc<PlayerNameTag> =
        Rpc::new(80, decode_player_name_tag, encode_player_name_tag);
    /// The `onAttachCameraToObject` descriptor.
    pub const ATTACH_CAMERA_TO_OBJECT: Rpc<u16> = Rpc::new(81, decode_u16, encode_u16);
    /// The `onGangZoneStopFlash` descriptor.
    pub const GANG_ZONE_STOP_FLASH: Rpc<u16> = Rpc::new(85, decode_u16, encode_u16);
    /// The `onClearPlayerAnimation` descriptor.
    pub const CLEAR_PLAYER_ANIMATION: Rpc<u16> = Rpc::new(87, decode_u16, encode_u16);
    /// The `onSetPlayerSpecialAction` descriptor.
    pub const SET_PLAYER_SPECIAL_ACTION: Rpc<u8> = Rpc::new(88, decode_u8, encode_u8);
    /// The `onSetPlayerFightingStyle` descriptor.
    pub const SET_PLAYER_FIGHTING_STYLE: Rpc<PlayerFightingStyle> = Rpc::new(
        89,
        decode_player_fighting_style,
        encode_player_fighting_style,
    );
    /// The `onSetPlayerVelocity` descriptor.
    pub const SET_PLAYER_VELOCITY: Rpc<Vector3> = Rpc::new(90, decode_vector3, encode_vector3);
    /// The `onSetVehicleVelocity` descriptor.
    pub const SET_VEHICLE_VELOCITY: Rpc<VehicleVelocity> =
        Rpc::new(91, decode_vehicle_velocity, encode_vehicle_velocity);
    /// The `onCreatePickup` descriptor.
    pub const CREATE_PICKUP: Rpc<Pickup> = Rpc::new(95, decode_pickup, encode_pickup);
    /// The `onMoveObject` descriptor.
    pub const MOVE_OBJECT: Rpc<MoveObject> = Rpc::new(99, decode_move_object, encode_move_object);
    /// The `onTextDrawSetString` descriptor.
    pub const TEXT_DRAW_SET_STRING: Rpc<TextDrawString> =
        Rpc::new(105, decode_text_draw_string, encode_text_draw_string);
    /// The `onCreateGangZone` descriptor.
    pub const CREATE_GANG_ZONE: Rpc<GangZone> = Rpc::new(108, decode_gang_zone, encode_gang_zone);
    /// The `onGangZoneDestroy` descriptor.
    pub const GANG_ZONE_DESTROY: Rpc<u16> = Rpc::new(120, decode_u16, encode_u16);
    /// The `onGangZoneFlash` descriptor.
    pub const GANG_ZONE_FLASH: Rpc<(u16, i32)> = Rpc::new(121, decode_u16_i32, encode_u16_i32);
    /// The `onStopObject` descriptor.
    pub const STOP_OBJECT: Rpc<u16> = Rpc::new(122, decode_u16, encode_u16);
    /// The `onSetVehicleNumberPlate` descriptor.
    pub const SET_VEHICLE_NUMBER_PLATE: Rpc<VehicleNumberPlate> = Rpc::new(
        123,
        decode_vehicle_number_plate,
        encode_vehicle_number_plate,
    );
    /// The `onSpectatePlayer` descriptor.
    pub const SPECTATE_PLAYER: Rpc<Spectate> = Rpc::new(126, decode_spectate, encode_spectate);
    /// The `onSpectateVehicle` descriptor.
    pub const SPECTATE_VEHICLE: Rpc<Spectate> = Rpc::new(127, decode_spectate, encode_spectate);
    /// The `onConnectionRejected` descriptor.
    pub const CONNECTION_REJECTED: Rpc<u8> = Rpc::new(130, decode_u8, encode_u8);
    /// The `onRemoveMapIcon` descriptor.
    pub const REMOVE_MAP_ICON: Rpc<u8> = Rpc::new(144, decode_u8, encode_u8);
    /// The `onSetWeaponAmmo` descriptor.
    pub const SET_WEAPON_AMMO: Rpc<WeaponAmmo> =
        Rpc::new(145, decode_weapon_ammo, encode_weapon_ammo);
    /// The `onSetGravity` descriptor.
    pub const SET_GRAVITY: Rpc<f32> = Rpc::new(146, decode_f32, encode_f32);
    /// The `onAttachTrailerToVehicle` descriptor.
    pub const ATTACH_TRAILER_TO_VEHICLE: Rpc<TrailerAttachment> =
        Rpc::new(148, decode_trailer_attachment, encode_trailer_attachment);
    /// The `onDetachTrailerFromVehicle` descriptor.
    pub const DETACH_TRAILER_FROM_VEHICLE: Rpc<u16> = Rpc::new(149, decode_u16, encode_u16);
    /// The `onSetCameraPosition` descriptor.
    pub const SET_CAMERA_POSITION: Rpc<Vector3> = Rpc::new(157, decode_vector3, encode_vector3);
    /// The `onSetCameraLookAt` descriptor.
    pub const SET_CAMERA_LOOK_AT: Rpc<CameraLookAt> =
        Rpc::new(158, decode_camera_look_at, encode_camera_look_at);
    /// The `onSetVehicleParams` descriptor.
    pub const SET_VEHICLE_PARAMS: Rpc<VehicleParams> =
        Rpc::new(161, decode_vehicle_params, encode_vehicle_params);
    /// The `onPlayerDeath` descriptor.
    pub const PLAYER_DEATH: Rpc<u16> = Rpc::new(166, decode_u16, encode_u16);
    /// The `onPlayerEnterVehicle` descriptor.
    pub const PLAYER_ENTER_VEHICLE: Rpc<PlayerEnterVehicle> =
        Rpc::new(26, decode_player_enter_vehicle, encode_player_enter_vehicle);
    /// The `onPlayerExitVehicle` descriptor.
    pub const PLAYER_EXIT_VEHICLE: Rpc<PlayerExitVehicle> =
        Rpc::new(154, decode_player_exit_vehicle, encode_player_exit_vehicle);
    /// The `onClientCheck` descriptor.
    pub const CLIENT_CHECK: Rpc<ClientCheck> =
        Rpc::new(103, decode_client_check, encode_client_check);
    /// The `onSetVehicleParamsEx` descriptor.
    pub const SET_VEHICLE_PARAMS_EX: Rpc<VehicleParamsEx> =
        Rpc::new(24, decode_vehicle_params_ex, encode_vehicle_params_ex);
    /// The `onVehicleTuningNotification` descriptor.
    pub const VEHICLE_TUNING_NOTIFICATION: Rpc<VehicleTuningNotification> = Rpc::new(
        96,
        decode_vehicle_tuning_notification,
        encode_vehicle_tuning_notification,
    );
    /// The `onSetVehicleTires` descriptor.
    pub const SET_VEHICLE_TIRES: Rpc<(u16, u8)> = Rpc::new(98, decode_u16_u8, encode_u16_u8);
    /// The `onVehicleDamageStatusUpdate` descriptor.
    pub const VEHICLE_DAMAGE_STATUS_UPDATE: Rpc<VehicleDamageStatus> = Rpc::new(
        106,
        decode_vehicle_damage_status,
        encode_vehicle_damage_status,
    );
    /// The `onToggleWidescreen` descriptor.
    pub const TOGGLE_WIDESCREEN: Rpc<bool> = Rpc::new(111, decode_bool8, encode_bool8);
    /// The `onDestroyActor` descriptor.
    pub const DESTROY_ACTOR: Rpc<u16> = Rpc::new(172, decode_u16, encode_u16);
    /// The `onDestroyWeaponPickup` descriptor.
    pub const DESTROY_WEAPON_PICKUP: Rpc<u8> = Rpc::new(151, decode_u8, encode_u8);
    /// The `onEditAttachedObject` descriptor.
    pub const EDIT_ATTACHED_OBJECT: Rpc<i32> = Rpc::new(116, decode_i32, encode_i32);
    /// The `onEnterSelectObject` descriptor.
    pub const ENTER_SELECT_OBJECT: Rpc<()> = Rpc::new(27, decode_empty, encode_empty);
    /// The `onServerStatisticsResponse` descriptor.
    pub const SERVER_STATISTICS_RESPONSE: Rpc<()> = Rpc::new(102, decode_empty, encode_empty);
    /// The `onSetPlayerDrunkVisuals` descriptor.
    pub const SET_PLAYER_DRUNK_VISUALS: Rpc<i32> = Rpc::new(92, decode_i32, encode_i32);
    /// The `onSetPlayerDrunkHandling` descriptor.
    pub const SET_PLAYER_DRUNK_HANDLING: Rpc<i32> = Rpc::new(150, decode_i32, encode_i32);
    /// The `onCreateActor` descriptor.
    pub const CREATE_ACTOR: Rpc<Actor> = Rpc::new(171, decode_actor, encode_actor);
    /// The `onClearActorAnimation` descriptor.
    pub const CLEAR_ACTOR_ANIMATION: Rpc<u16> = Rpc::new(174, decode_u16, encode_u16);
    /// The `onSetActorFacingAngle` descriptor.
    pub const SET_ACTOR_FACING_ANGLE: Rpc<ActorAngle> =
        Rpc::new(175, decode_actor_angle, encode_actor_angle);
    /// The `onSetActorPos` descriptor.
    pub const SET_ACTOR_POSITION: Rpc<ActorPosition> =
        Rpc::new(176, decode_actor_position, encode_actor_position);
    /// The `onSetActorHealth` descriptor.
    pub const SET_ACTOR_HEALTH: Rpc<ActorHealth> =
        Rpc::new(178, decode_actor_health, encode_actor_health);
    /// The `onSetPlayerObjectNoCameraCol` descriptor.
    pub const SET_PLAYER_OBJECT_NO_CAMERA_COL: Rpc<u16> = Rpc::new(169, decode_u16, encode_u16);
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
    /// The R1 `onInitGame` descriptor.
    pub const INIT_GAME: Rpc<InitGame> = Rpc::new_bits(139, decode_init_game, encode_init_game);
    /// The R1 `onRequestClassResponse` descriptor.
    pub const REQUEST_CLASS_RESPONSE: Rpc<RequestClassResponse> = Rpc::new_bits(
        128,
        decode_request_class_response,
        encode_request_class_response,
    );
    /// The R1 `onPlayerStreamIn` descriptor.
    pub const PLAYER_STREAM_IN: Rpc<PlayerStreamIn> =
        Rpc::new_bits(32, decode_player_stream_in, encode_player_stream_in);
    /// The R1 `onCreate3DText` descriptor.
    pub const CREATE_3D_TEXT: Rpc<TextLabel3D> =
        Rpc::new_bits(36, decode_text_label_3d, encode_text_label_3d);
    /// The R1 `onCreateObject` descriptor.
    pub const CREATE_OBJECT: Rpc<Object> = Rpc::new_bits(44, decode_object, encode_object);
    /// The R1 `onSetSpawnInfo` descriptor.
    pub const SET_SPAWN_INFO: Rpc<SpawnInfo> =
        Rpc::new_bits(68, decode_spawn_info, encode_spawn_info);
    /// The R1 `onInitMenu` descriptor.
    pub const INIT_MENU: Rpc<InitMenu> = Rpc::new_bits(76, decode_init_menu, encode_init_menu);
    /// The R1 `onInterpolateCamera` descriptor.
    pub const INTERPOLATE_CAMERA: Rpc<InterpolateCamera> =
        Rpc::new_bits(82, decode_interpolate_camera, encode_interpolate_camera);
    /// The R1 `onToggleSelectTextDraw` descriptor.
    pub const TOGGLE_SELECT_TEXT_DRAW: Rpc<ToggleSelectTextDraw> = Rpc::new_bits(
        83,
        decode_toggle_select_text_draw,
        encode_toggle_select_text_draw,
    );
    /// The R1 object material descriptor. [`ObjectMaterial`] preserves either material variant.
    pub const SET_OBJECT_MATERIAL: Rpc<ObjectMaterialUpdate> = Rpc::new_bits(
        84,
        decode_object_material_update,
        encode_object_material_update,
    );
    /// The R1 `onApplyPlayerAnimation` descriptor.
    pub const APPLY_PLAYER_ANIMATION: Rpc<PlayerAnimation> =
        Rpc::new_bits(86, decode_player_animation, encode_player_animation);
    /// The R1 `onEnableStuntBonus` descriptor.
    pub const ENABLE_STUNT_BONUS: Rpc<bool> = Rpc::new_bits(104, decode_bit_bool, encode_bit_bool);
    /// The R1 `onPlayCrimeReport` descriptor.
    pub const PLAY_CRIME_REPORT: Rpc<CrimeReport> =
        Rpc::new_bits(112, decode_crime_report, encode_crime_report);
    /// The R1 `onSetPlayerAttachedObject` descriptor.
    pub const SET_PLAYER_ATTACHED_OBJECT: Rpc<PlayerAttachedObject> = Rpc::new_bits(
        113,
        decode_player_attached_object,
        encode_player_attached_object,
    );
    /// The R1 `onEnterEditObject` descriptor.
    pub const ENTER_EDIT_OBJECT: Rpc<EnterEditObject> =
        Rpc::new_bits(117, decode_enter_edit_object, encode_enter_edit_object);
    /// The R1 `onTogglePlayerSpectating` descriptor.
    pub const TOGGLE_PLAYER_SPECTATING: Rpc<bool> =
        Rpc::new_bits(124, decode_bool32, encode_bool32);
    /// The R1 `onShowTextDraw` descriptor.
    pub const SHOW_TEXT_DRAW: Rpc<ShowTextDraw> =
        Rpc::new_bits(134, decode_show_text_draw, encode_show_text_draw);
    /// The R1 `onTextDrawHide` descriptor.
    pub const TEXT_DRAW_HIDE: Rpc<u16> = Rpc::new(135, decode_u16, encode_u16);
    /// The R1 `onInitGame` score/ping update descriptor.
    pub const UPDATE_SCORES_AND_PINGS: Rpc<ScoresAndPings> =
        Rpc::new_bits(155, decode_scores_and_pings, encode_scores_and_pings);
    /// The R1 `onVehicleStreamIn` descriptor.
    pub const VEHICLE_STREAM_IN: Rpc<VehicleStreamIn> =
        Rpc::new_bits(164, decode_vehicle_stream_in, encode_vehicle_stream_in);
    /// The R1 `onDisableVehicleCollisions` descriptor.
    pub const DISABLE_VEHICLE_COLLISIONS: Rpc<bool> =
        Rpc::new_bits(167, decode_bit_bool, encode_bit_bool);
    /// The R1 `onToggleCameraTargetNotifying` descriptor.
    pub const TOGGLE_CAMERA_TARGET_NOTIFYING: Rpc<bool> =
        Rpc::new_bits(170, decode_bit_bool, encode_bit_bool);
    /// The R1 `onApplyActorAnimation` descriptor.
    pub const APPLY_ACTOR_ANIMATION: Rpc<ActorAnimation> =
        Rpc::new_bits(173, decode_actor_animation, encode_actor_animation);

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
    rpc_helper!(on_cancel_edit, (), CANCEL_EDIT, "onCancelEdit");
    rpc_helper!(
        on_set_toggle_clock,
        bool,
        SET_TOGGLE_CLOCK,
        "onSetToggleClock"
    );
    rpc_helper!(
        on_set_player_drunk,
        i32,
        SET_PLAYER_DRUNK,
        "onSetPlayerDrunk"
    );
    rpc_helper!(
        on_set_race_checkpoint,
        RaceCheckpoint,
        SET_RACE_CHECKPOINT,
        "onSetRaceCheckpoint"
    );
    rpc_helper!(
        on_play_audio_stream,
        AudioStream,
        PLAY_AUDIO_STREAM,
        "onPlayAudioStream"
    );
    rpc_helper!(
        on_set_object_position,
        ObjectPosition,
        SET_OBJECT_POSITION,
        "onSetObjectPosition"
    );
    rpc_helper!(
        on_set_object_rotation,
        ObjectRotation,
        SET_OBJECT_ROTATION,
        "onSetObjectRotation"
    );
    rpc_helper!(on_destroy_object, u16, DESTROY_OBJECT, "onDestroyObject");
    rpc_helper!(
        on_player_death_notification,
        PlayerDeathNotification,
        PLAYER_DEATH_NOTIFICATION,
        "onPlayerDeathNotification"
    );
    rpc_helper!(on_set_map_icon, MapIcon, SET_MAP_ICON, "onSetMapIcon");
    rpc_helper!(
        on_remove_vehicle_component,
        VehicleComponent,
        REMOVE_VEHICLE_COMPONENT,
        "onRemoveVehicleComponent"
    );
    rpc_helper!(
        on_remove_3d_text_label,
        u16,
        REMOVE_3D_TEXT_LABEL,
        "onRemove3DTextLabel"
    );
    rpc_helper!(
        on_update_global_timer,
        i32,
        UPDATE_GLOBAL_TIMER,
        "onUpdateGlobalTimer"
    );
    rpc_helper!(on_destroy_pickup, i32, DESTROY_PICKUP, "onDestroyPickup");
    rpc_helper!(
        on_link_vehicle_to_interior,
        VehicleInterior,
        LINK_VEHICLE_TO_INTERIOR,
        "onLinkVehicleToInterior"
    );
    rpc_helper!(
        on_set_player_color,
        PlayerColor,
        SET_PLAYER_COLOR,
        "onSetPlayerColor"
    );
    rpc_helper!(
        on_request_spawn_response,
        bool,
        REQUEST_SPAWN_RESPONSE,
        "onRequestSpawnResponse"
    );
    rpc_helper!(on_set_shop_name, [u8; 32], SET_SHOP_NAME, "onSetShopName");
    rpc_helper!(
        on_set_player_skill_level,
        PlayerSkill,
        SET_PLAYER_SKILL_LEVEL,
        "onSetPlayerSkillLevel"
    );
    rpc_helper!(
        on_remove_building,
        RemoveBuilding,
        REMOVE_BUILDING,
        "onRemoveBuilding"
    );
    rpc_helper!(
        on_attach_object_to_player,
        AttachObjectToPlayer,
        ATTACH_OBJECT_TO_PLAYER,
        "onAttachObjectToPlayer"
    );
    rpc_helper!(on_show_menu, u8, SHOW_MENU, "onShowMenu");
    rpc_helper!(on_hide_menu, u8, HIDE_MENU, "onHideMenu");
    rpc_helper!(
        on_create_explosion,
        Explosion,
        CREATE_EXPLOSION,
        "onCreateExplosion"
    );
    rpc_helper!(
        on_show_player_name_tag,
        PlayerNameTag,
        SHOW_PLAYER_NAME_TAG,
        "onShowPlayerNameTag"
    );
    rpc_helper!(
        on_attach_camera_to_object,
        u16,
        ATTACH_CAMERA_TO_OBJECT,
        "onAttachCameraToObject"
    );
    rpc_helper!(
        on_gang_zone_stop_flash,
        u16,
        GANG_ZONE_STOP_FLASH,
        "onGangZoneStopFlash"
    );
    rpc_helper!(
        on_clear_player_animation,
        u16,
        CLEAR_PLAYER_ANIMATION,
        "onClearPlayerAnimation"
    );
    rpc_helper!(
        on_set_player_special_action,
        u8,
        SET_PLAYER_SPECIAL_ACTION,
        "onSetPlayerSpecialAction"
    );
    rpc_helper!(
        on_set_player_fighting_style,
        PlayerFightingStyle,
        SET_PLAYER_FIGHTING_STYLE,
        "onSetPlayerFightingStyle"
    );
    rpc_helper!(
        on_set_player_velocity,
        Vector3,
        SET_PLAYER_VELOCITY,
        "onSetPlayerVelocity"
    );
    rpc_helper!(
        on_set_vehicle_velocity,
        VehicleVelocity,
        SET_VEHICLE_VELOCITY,
        "onSetVehicleVelocity"
    );
    rpc_helper!(on_create_pickup, Pickup, CREATE_PICKUP, "onCreatePickup");
    rpc_helper!(on_move_object, MoveObject, MOVE_OBJECT, "onMoveObject");
    rpc_helper!(
        on_text_draw_set_string,
        TextDrawString,
        TEXT_DRAW_SET_STRING,
        "onTextDrawSetString"
    );
    rpc_helper!(
        on_create_gang_zone,
        GangZone,
        CREATE_GANG_ZONE,
        "onCreateGangZone"
    );
    rpc_helper!(
        on_gang_zone_destroy,
        u16,
        GANG_ZONE_DESTROY,
        "onGangZoneDestroy"
    );
    rpc_helper!(
        on_gang_zone_flash,
        (u16, i32),
        GANG_ZONE_FLASH,
        "onGangZoneFlash"
    );
    rpc_helper!(on_stop_object, u16, STOP_OBJECT, "onStopObject");
    rpc_helper!(
        on_set_vehicle_number_plate,
        VehicleNumberPlate,
        SET_VEHICLE_NUMBER_PLATE,
        "onSetVehicleNumberPlate"
    );
    rpc_helper!(
        on_spectate_player,
        Spectate,
        SPECTATE_PLAYER,
        "onSpectatePlayer"
    );
    rpc_helper!(
        on_spectate_vehicle,
        Spectate,
        SPECTATE_VEHICLE,
        "onSpectateVehicle"
    );
    rpc_helper!(
        on_connection_rejected,
        u8,
        CONNECTION_REJECTED,
        "onConnectionRejected"
    );
    rpc_helper!(on_remove_map_icon, u8, REMOVE_MAP_ICON, "onRemoveMapIcon");
    rpc_helper!(
        on_set_weapon_ammo,
        WeaponAmmo,
        SET_WEAPON_AMMO,
        "onSetWeaponAmmo"
    );
    rpc_helper!(on_set_gravity, f32, SET_GRAVITY, "onSetGravity");
    rpc_helper!(
        on_attach_trailer_to_vehicle,
        TrailerAttachment,
        ATTACH_TRAILER_TO_VEHICLE,
        "onAttachTrailerToVehicle"
    );
    rpc_helper!(
        on_detach_trailer_from_vehicle,
        u16,
        DETACH_TRAILER_FROM_VEHICLE,
        "onDetachTrailerFromVehicle"
    );
    rpc_helper!(
        on_set_camera_position,
        Vector3,
        SET_CAMERA_POSITION,
        "onSetCameraPosition"
    );
    rpc_helper!(
        on_set_camera_look_at,
        CameraLookAt,
        SET_CAMERA_LOOK_AT,
        "onSetCameraLookAt"
    );
    rpc_helper!(
        on_set_vehicle_params,
        VehicleParams,
        SET_VEHICLE_PARAMS,
        "onSetVehicleParams"
    );
    rpc_helper!(on_player_death, u16, PLAYER_DEATH, "onPlayerDeath");
    rpc_helper!(
        on_player_enter_vehicle,
        PlayerEnterVehicle,
        PLAYER_ENTER_VEHICLE,
        "onPlayerEnterVehicle"
    );
    rpc_helper!(
        on_player_exit_vehicle,
        PlayerExitVehicle,
        PLAYER_EXIT_VEHICLE,
        "onPlayerExitVehicle"
    );
    rpc_helper!(on_client_check, ClientCheck, CLIENT_CHECK, "onClientCheck");
    rpc_helper!(
        on_set_vehicle_params_ex,
        VehicleParamsEx,
        SET_VEHICLE_PARAMS_EX,
        "onSetVehicleParamsEx"
    );
    rpc_helper!(
        on_vehicle_tuning_notification,
        VehicleTuningNotification,
        VEHICLE_TUNING_NOTIFICATION,
        "onVehicleTuningNotification"
    );
    rpc_helper!(
        on_set_vehicle_tires,
        (u16, u8),
        SET_VEHICLE_TIRES,
        "onSetVehicleTires"
    );
    rpc_helper!(
        on_vehicle_damage_status_update,
        VehicleDamageStatus,
        VEHICLE_DAMAGE_STATUS_UPDATE,
        "onVehicleDamageStatusUpdate"
    );
    rpc_helper!(
        on_toggle_widescreen,
        bool,
        TOGGLE_WIDESCREEN,
        "onToggleWidescreen"
    );
    rpc_helper!(on_destroy_actor, u16, DESTROY_ACTOR, "onDestroyActor");
    rpc_helper!(
        on_destroy_weapon_pickup,
        u8,
        DESTROY_WEAPON_PICKUP,
        "onDestroyWeaponPickup"
    );
    rpc_helper!(
        on_edit_attached_object,
        i32,
        EDIT_ATTACHED_OBJECT,
        "onEditAttachedObject"
    );
    rpc_helper!(
        on_enter_select_object,
        (),
        ENTER_SELECT_OBJECT,
        "onEnterSelectObject"
    );
    rpc_helper!(
        on_server_statistics_response,
        (),
        SERVER_STATISTICS_RESPONSE,
        "onServerStatisticsResponse"
    );
    rpc_helper!(
        on_set_player_drunk_visuals,
        i32,
        SET_PLAYER_DRUNK_VISUALS,
        "onSetPlayerDrunkVisuals"
    );
    rpc_helper!(
        on_set_player_drunk_handling,
        i32,
        SET_PLAYER_DRUNK_HANDLING,
        "onSetPlayerDrunkHandling"
    );
    rpc_helper!(on_create_actor, Actor, CREATE_ACTOR, "onCreateActor");
    rpc_helper!(
        on_clear_actor_animation,
        u16,
        CLEAR_ACTOR_ANIMATION,
        "onClearActorAnimation"
    );
    rpc_helper!(
        on_set_actor_facing_angle,
        ActorAngle,
        SET_ACTOR_FACING_ANGLE,
        "onSetActorFacingAngle"
    );
    rpc_helper!(
        on_set_actor_position,
        ActorPosition,
        SET_ACTOR_POSITION,
        "onSetActorPos"
    );
    rpc_helper!(
        on_set_actor_health,
        ActorHealth,
        SET_ACTOR_HEALTH,
        "onSetActorHealth"
    );
    rpc_helper!(
        on_set_player_object_no_camera_col,
        u16,
        SET_PLAYER_OBJECT_NO_CAMERA_COL,
        "onSetPlayerObjectNoCameraCol"
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
    rpc_helper!(
        on_set_player_skin,
        PlayerSkin,
        SET_PLAYER_SKIN,
        "onSetPlayerSkin"
    );
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
    rpc_helper!(on_init_game, InitGame, INIT_GAME, "onInitGame");
    rpc_helper!(
        on_request_class_response,
        RequestClassResponse,
        REQUEST_CLASS_RESPONSE,
        "onRequestClassResponse"
    );
    rpc_helper!(
        on_player_stream_in,
        PlayerStreamIn,
        PLAYER_STREAM_IN,
        "onPlayerStreamIn"
    );
    rpc_helper!(
        on_create_3d_text,
        TextLabel3D,
        CREATE_3D_TEXT,
        "onCreate3DText"
    );
    rpc_helper!(on_create_object, Object, CREATE_OBJECT, "onCreateObject");
    rpc_helper!(
        on_set_spawn_info,
        SpawnInfo,
        SET_SPAWN_INFO,
        "onSetSpawnInfo"
    );
    rpc_helper!(on_init_menu, InitMenu, INIT_MENU, "onInitMenu");
    rpc_helper!(
        on_interpolate_camera,
        InterpolateCamera,
        INTERPOLATE_CAMERA,
        "onInterpolateCamera"
    );
    rpc_helper!(
        on_toggle_select_text_draw,
        ToggleSelectTextDraw,
        TOGGLE_SELECT_TEXT_DRAW,
        "onToggleSelectTextDraw"
    );
    rpc_helper!(
        on_set_object_material,
        ObjectMaterialUpdate,
        SET_OBJECT_MATERIAL,
        "onSetObjectMaterial/onSetObjectMaterialText"
    );
    rpc_helper!(
        on_apply_player_animation,
        PlayerAnimation,
        APPLY_PLAYER_ANIMATION,
        "onApplyPlayerAnimation"
    );
    rpc_helper!(
        on_enable_stunt_bonus,
        bool,
        ENABLE_STUNT_BONUS,
        "onEnableStuntBonus"
    );
    rpc_helper!(
        on_play_crime_report,
        CrimeReport,
        PLAY_CRIME_REPORT,
        "onPlayCrimeReport"
    );
    rpc_helper!(
        on_set_player_attached_object,
        PlayerAttachedObject,
        SET_PLAYER_ATTACHED_OBJECT,
        "onSetPlayerAttachedObject"
    );
    rpc_helper!(
        on_enter_edit_object,
        EnterEditObject,
        ENTER_EDIT_OBJECT,
        "onEnterEditObject"
    );
    rpc_helper!(
        on_toggle_player_spectating,
        bool,
        TOGGLE_PLAYER_SPECTATING,
        "onTogglePlayerSpectating"
    );
    rpc_helper!(
        on_show_text_draw,
        ShowTextDraw,
        SHOW_TEXT_DRAW,
        "onShowTextDraw"
    );
    rpc_helper!(on_text_draw_hide, u16, TEXT_DRAW_HIDE, "onTextDrawHide");
    rpc_helper!(
        on_update_scores_and_pings,
        ScoresAndPings,
        UPDATE_SCORES_AND_PINGS,
        "onUpdateScoresAndPings"
    );
    rpc_helper!(
        on_vehicle_stream_in,
        VehicleStreamIn,
        VEHICLE_STREAM_IN,
        "onVehicleStreamIn"
    );
    rpc_helper!(
        on_disable_vehicle_collisions,
        bool,
        DISABLE_VEHICLE_COLLISIONS,
        "onDisableVehicleCollisions"
    );
    rpc_helper!(
        on_toggle_camera_target_notifying,
        bool,
        TOGGLE_CAMERA_TARGET_NOTIFYING,
        "onToggleCameraTargetNotifying"
    );
    rpc_helper!(
        on_apply_actor_animation,
        ActorAnimation,
        APPLY_ACTOR_ANIMATION,
        "onApplyActorAnimation"
    );

    fn read_bit_bool(event: &mut Event<'_>) -> Result<bool, EventError> {
        Ok(event.read_bits(1)?[0] & 0x80 != 0)
    }

    fn decode_bit_bool(event: &mut Event<'_>) -> Result<bool, EventError> {
        read_bit_bool(event)
    }

    fn encode_bit_bool(_api: HostApi, value: bool) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.bool(value);
        Ok(writer.finish_bits())
    }

    fn decode_bool32(event: &mut Event<'_>) -> Result<bool, EventError> {
        Ok(event.read_u32()? != 0)
    }

    fn encode_bool32(_api: HostApi, value: bool) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(u32::from(value));
        Ok(writer.finish_bits())
    }

    fn read_i16(event: &mut Event<'_>) -> Result<i16, EventError> {
        Ok(event.read_u16()? as i16)
    }

    fn write_i32(writer: &mut PayloadWriter, value: i32) {
        writer.u32(value as u32);
    }

    fn read_fixed_string32(event: &mut Event<'_>) -> Result<[u8; 32], EventError> {
        read_array(event)
    }

    fn write_fixed_string32(writer: &mut PayloadWriter, value: [u8; 32]) {
        writer.bytes(&value);
    }

    fn decode_spawn_info_fields(event: &mut Event<'_>) -> Result<SpawnInfo, EventError> {
        Ok(SpawnInfo {
            team: event.read_u8()?,
            skin: decode_i32(event)?,
            unused: event.read_u8()?,
            position: decode_vector3(event)?,
            rotation: event.read_f32()?,
            weapons: [decode_i32(event)?, decode_i32(event)?, decode_i32(event)?],
            ammo: [decode_i32(event)?, decode_i32(event)?, decode_i32(event)?],
        })
    }

    fn encode_spawn_info_fields(writer: &mut PayloadWriter, value: SpawnInfo) {
        writer.u8(value.team);
        write_i32(writer, value.skin);
        writer.u8(value.unused);
        writer.vector3(value.position);
        writer.f32(value.rotation);
        for weapon in value.weapons {
            write_i32(writer, weapon);
        }
        for ammo in value.ammo {
            write_i32(writer, ammo);
        }
    }

    fn decode_init_game(event: &mut Event<'_>) -> Result<InitGame, EventError> {
        let mut settings = GameSettings {
            zone_names: read_bit_bool(event)?,
            use_cj_walk: read_bit_bool(event)?,
            allow_weapons: read_bit_bool(event)?,
            limit_global_chat_radius: read_bit_bool(event)?,
            global_chat_radius: event.read_f32()?,
            stunt_bonus: read_bit_bool(event)?,
            nametag_draw_distance: event.read_f32()?,
            disable_enter_exits: read_bit_bool(event)?,
            nametag_los: read_bit_bool(event)?,
            tire_popping: read_bit_bool(event)?,
            classes_available: decode_i32(event)?,
            show_player_tags: false,
            player_markers_mode: 0,
            world_time: 0,
            world_weather: 0,
            gravity: 0.0,
            lan_mode: false,
            death_money_drop: 0,
            instagib: false,
            normal_onfoot_send_rate: 0,
            normal_incar_send_rate: 0,
            normal_firing_send_rate: 0,
            send_multiplier: 0,
            lag_compensation_mode: 0,
            vehicle_friendly_fire: false,
        };
        let player_id = event.read_u16()?;
        settings.show_player_tags = read_bit_bool(event)?;
        settings.player_markers_mode = decode_i32(event)?;
        settings.world_time = event.read_u8()?;
        settings.world_weather = event.read_u8()?;
        settings.gravity = event.read_f32()?;
        settings.lan_mode = read_bit_bool(event)?;
        settings.death_money_drop = decode_i32(event)?;
        settings.instagib = read_bit_bool(event)?;
        settings.normal_onfoot_send_rate = decode_i32(event)?;
        settings.normal_incar_send_rate = decode_i32(event)?;
        settings.normal_firing_send_rate = decode_i32(event)?;
        settings.send_multiplier = decode_i32(event)?;
        settings.lag_compensation_mode = decode_i32(event)?;
        let host_name = event.read_string8()?;
        let vehicle_models = read_array::<212>(event)?;
        settings.vehicle_friendly_fire = decode_bool32(event)?;
        Ok(InitGame {
            player_id,
            host_name,
            settings,
            vehicle_models,
        })
    }

    fn encode_init_game(api: HostApi, value: InitGame) -> Result<EncodedPayload, EventError> {
        let _ = api;
        let settings = value.settings;
        let mut writer = PayloadWriter::new();
        writer.bool(settings.zone_names);
        writer.bool(settings.use_cj_walk);
        writer.bool(settings.allow_weapons);
        writer.bool(settings.limit_global_chat_radius);
        writer.f32(settings.global_chat_radius);
        writer.bool(settings.stunt_bonus);
        writer.f32(settings.nametag_draw_distance);
        writer.bool(settings.disable_enter_exits);
        writer.bool(settings.nametag_los);
        writer.bool(settings.tire_popping);
        write_i32(&mut writer, settings.classes_available);
        writer.u16(value.player_id);
        writer.bool(settings.show_player_tags);
        write_i32(&mut writer, settings.player_markers_mode);
        writer.u8(settings.world_time);
        writer.u8(settings.world_weather);
        writer.f32(settings.gravity);
        writer.bool(settings.lan_mode);
        write_i32(&mut writer, settings.death_money_drop);
        writer.bool(settings.instagib);
        write_i32(&mut writer, settings.normal_onfoot_send_rate);
        write_i32(&mut writer, settings.normal_incar_send_rate);
        write_i32(&mut writer, settings.normal_firing_send_rate);
        write_i32(&mut writer, settings.send_multiplier);
        write_i32(&mut writer, settings.lag_compensation_mode);
        writer.string8(&value.host_name)?;
        writer.bytes(&value.vehicle_models);
        writer.u32(u32::from(settings.vehicle_friendly_fire));
        Ok(writer.finish_bits())
    }

    fn decode_request_class_response(
        event: &mut Event<'_>,
    ) -> Result<RequestClassResponse, EventError> {
        Ok(RequestClassResponse {
            can_spawn: decode_bool8(event)?,
            spawn: decode_spawn_info_fields(event)?,
        })
    }

    fn encode_request_class_response(
        _api: HostApi,
        value: RequestClassResponse,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u8(u8::from(value.can_spawn));
        encode_spawn_info_fields(&mut writer, value.spawn);
        Ok(writer.finish_bits())
    }

    fn decode_player_stream_in(event: &mut Event<'_>) -> Result<PlayerStreamIn, EventError> {
        let player_id = event.read_u16()?;
        let team = event.read_u8()?;
        let model = decode_i32(event)?;
        let position = decode_vector3(event)?;
        let rotation = event.read_f32()?;
        let color = decode_i32(event)?;
        let fighting_style = event.read_u8()?;
        let mut weapon_skill_levels = [0; 11];
        for skill_level in &mut weapon_skill_levels {
            *skill_level = event.read_u16()?;
        }
        Ok(PlayerStreamIn {
            player_id,
            team,
            model,
            position,
            rotation,
            color,
            fighting_style,
            weapon_skill_levels,
        })
    }

    fn encode_player_stream_in(
        _api: HostApi,
        value: PlayerStreamIn,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u8(value.team);
        write_i32(&mut writer, value.model);
        writer.vector3(value.position);
        writer.f32(value.rotation);
        write_i32(&mut writer, value.color);
        writer.u8(value.fighting_style);
        for skill_level in value.weapon_skill_levels {
            writer.u16(skill_level);
        }
        Ok(writer.finish_bits())
    }

    fn decode_text_label_3d(event: &mut Event<'_>) -> Result<TextLabel3D, EventError> {
        Ok(TextLabel3D {
            id: event.read_u16()?,
            color: decode_i32(event)?,
            position: decode_vector3(event)?,
            distance: event.read_f32()?,
            test_los: decode_bool8(event)?,
            attached_player_id: event.read_u16()?,
            attached_vehicle_id: event.read_u16()?,
            text: event.read_encoded_string(MAX_ENCODED_STRING_BYTES + 1)?,
        })
    }

    fn encode_text_label_3d(
        api: HostApi,
        value: TextLabel3D,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.id);
        write_i32(&mut writer, value.color);
        writer.vector3(value.position);
        writer.f32(value.distance);
        writer.u8(u8::from(value.test_los));
        writer.u16(value.attached_player_id);
        writer.u16(value.attached_vehicle_id);
        writer.encoded_string(api, &value.text)?;
        Ok(writer.finish_bits())
    }

    fn decode_texture_material(event: &mut Event<'_>) -> Result<TextureMaterial, EventError> {
        Ok(TextureMaterial {
            material_id: event.read_u8()?,
            model_id: event.read_u16()?,
            library_name: event.read_string8()?,
            texture_name: event.read_string8()?,
            color: decode_i32(event)?,
        })
    }

    fn encode_texture_material(
        writer: &mut PayloadWriter,
        value: TextureMaterial,
    ) -> Result<(), EventError> {
        writer.u8(1);
        writer.u8(value.material_id);
        writer.u16(value.model_id);
        writer.string8(&value.library_name)?;
        writer.string8(&value.texture_name)?;
        write_i32(writer, value.color);
        Ok(())
    }

    fn decode_text_material(event: &mut Event<'_>) -> Result<TextMaterial, EventError> {
        Ok(TextMaterial {
            material_id: event.read_u8()?,
            material_size: event.read_u8()?,
            font_name: event.read_string8()?,
            font_size: event.read_u8()?,
            bold: event.read_u8()?,
            font_color: decode_i32(event)?,
            background_color: decode_i32(event)?,
            align: event.read_u8()?,
            text: event.read_encoded_string(MAX_OBJECT_MATERIAL_TEXT_BYTES + 1)?,
        })
    }

    fn encode_text_material(
        api: HostApi,
        writer: &mut PayloadWriter,
        value: TextMaterial,
    ) -> Result<(), EventError> {
        writer.u8(2);
        writer.u8(value.material_id);
        writer.u8(value.material_size);
        writer.string8(&value.font_name)?;
        writer.u8(value.font_size);
        writer.u8(value.bold);
        write_i32(writer, value.font_color);
        write_i32(writer, value.background_color);
        writer.u8(value.align);
        writer.encoded_string_with_limit(api, &value.text, MAX_OBJECT_MATERIAL_TEXT_BYTES)
    }

    fn decode_object_material(event: &mut Event<'_>) -> Result<ObjectMaterial, EventError> {
        match event.read_u8()? {
            1 => Ok(ObjectMaterial::Texture(decode_texture_material(event)?)),
            2 => Ok(ObjectMaterial::Text(decode_text_material(event)?)),
            value => Err(EventError::InvalidDiscriminant { value }),
        }
    }

    fn encode_object_material(
        api: HostApi,
        writer: &mut PayloadWriter,
        value: ObjectMaterial,
    ) -> Result<(), EventError> {
        match value {
            ObjectMaterial::Texture(value) => encode_texture_material(writer, value),
            ObjectMaterial::Text(value) => encode_text_material(api, writer, value),
        }
    }

    fn decode_object(event: &mut Event<'_>) -> Result<Object, EventError> {
        let object_id = event.read_u16()?;
        let model_id = decode_i32(event)?;
        let position = decode_vector3(event)?;
        let rotation = decode_vector3(event)?;
        let draw_distance = event.read_f32()?;
        let no_camera_collision = decode_bool8(event)?;
        let attach_to_vehicle_id = event.read_u16()?;
        let attach_to_object_id = event.read_u16()?;
        let attachment = (attach_to_vehicle_id != u16::MAX || attach_to_object_id != u16::MAX)
            .then(|| {
                Ok(ObjectAttachment {
                    offsets: decode_vector3(event)?,
                    rotation: decode_vector3(event)?,
                    sync_rotation: decode_bool8(event)?,
                })
            })
            .transpose()?;
        let textures_count = event.read_u8()?;
        let mut materials = Vec::new();
        while event.remaining_bits() != 0 {
            if materials.len() == MAX_OBJECT_MATERIALS {
                return Err(EventError::LengthExceedsLimit {
                    length: materials.len() + 1,
                    limit: MAX_OBJECT_MATERIALS,
                });
            }
            materials.push(decode_object_material(event)?);
        }
        Ok(Object {
            object_id,
            model_id,
            position,
            rotation,
            draw_distance,
            no_camera_collision,
            attach_to_vehicle_id,
            attach_to_object_id,
            attachment,
            textures_count,
            materials,
        })
    }

    fn encode_object(api: HostApi, value: Object) -> Result<EncodedPayload, EventError> {
        if value.materials.len() > MAX_OBJECT_MATERIALS {
            return Err(EventError::LengthExceedsLimit {
                length: value.materials.len(),
                limit: MAX_OBJECT_MATERIALS,
            });
        }
        let attachment_required =
            value.attach_to_vehicle_id != u16::MAX || value.attach_to_object_id != u16::MAX;
        if attachment_required != value.attachment.is_some() {
            return Err(EventError::ValueOutOfRange {
                value: usize::from(value.attachment.is_some()),
                maximum: usize::from(attachment_required),
            });
        }
        let mut writer = PayloadWriter::new();
        writer.u16(value.object_id);
        write_i32(&mut writer, value.model_id);
        writer.vector3(value.position);
        writer.vector3(value.rotation);
        writer.f32(value.draw_distance);
        writer.u8(u8::from(value.no_camera_collision));
        writer.u16(value.attach_to_vehicle_id);
        writer.u16(value.attach_to_object_id);
        if let Some(attachment) = value.attachment {
            writer.vector3(attachment.offsets);
            writer.vector3(attachment.rotation);
            writer.u8(u8::from(attachment.sync_rotation));
        }
        writer.u8(value.textures_count);
        for material in value.materials {
            encode_object_material(api, &mut writer, material)?;
        }
        Ok(writer.finish_bits())
    }

    fn decode_spawn_info(event: &mut Event<'_>) -> Result<SpawnInfo, EventError> {
        decode_spawn_info_fields(event)
    }

    fn encode_spawn_info(api: HostApi, value: SpawnInfo) -> Result<EncodedPayload, EventError> {
        let _ = api;
        let mut writer = PayloadWriter::new();
        encode_spawn_info_fields(&mut writer, value);
        Ok(writer.finish_bits())
    }

    fn decode_menu_column(event: &mut Event<'_>, width: f32) -> Result<MenuColumn, EventError> {
        let title = read_fixed_string32(event)?;
        let row_count = usize::from(event.read_u8()?);
        if row_count > MAX_MENU_ROWS {
            return Err(EventError::LengthExceedsLimit {
                length: row_count,
                limit: MAX_MENU_ROWS,
            });
        }
        let mut rows = Vec::with_capacity(row_count);
        for _ in 0..row_count {
            rows.push(read_fixed_string32(event)?);
        }
        Ok(MenuColumn { width, title, rows })
    }

    fn encode_menu_column(writer: &mut PayloadWriter, value: MenuColumn) -> Result<(), EventError> {
        if value.rows.len() > MAX_MENU_ROWS {
            return Err(EventError::LengthExceedsLimit {
                length: value.rows.len(),
                limit: MAX_MENU_ROWS,
            });
        }
        write_fixed_string32(writer, value.title);
        writer.u8(value.rows.len() as u8);
        for row in value.rows {
            write_fixed_string32(writer, row);
        }
        Ok(())
    }

    fn decode_init_menu(event: &mut Event<'_>) -> Result<InitMenu, EventError> {
        let menu_id = event.read_u8()?;
        let two_columns = decode_bool32(event)?;
        let title = read_fixed_string32(event)?;
        let position = decode_vector2(event)?;
        let width1 = event.read_f32()?;
        let width2 = two_columns.then(|| event.read_f32()).transpose()?;
        let menu = decode_bool32(event)?;
        let mut rows = [0_i32; MAX_MENU_ROWS];
        for row in &mut rows {
            *row = decode_i32(event)?;
        }
        let mut columns = Vec::with_capacity(if two_columns { 2 } else { 1 });
        columns.push(decode_menu_column(event, width1)?);
        if let Some(width2) = width2 {
            columns.push(decode_menu_column(event, width2)?);
        }
        Ok(InitMenu {
            menu_id,
            two_columns,
            title,
            position,
            columns,
            rows,
            menu,
        })
    }

    fn encode_init_menu(api: HostApi, value: InitMenu) -> Result<EncodedPayload, EventError> {
        let _ = api;
        let expected_columns = if value.two_columns { 2 } else { 1 };
        if value.columns.len() != expected_columns || value.columns.len() > MAX_MENU_COLUMNS {
            return Err(EventError::ValueOutOfRange {
                value: value.columns.len(),
                maximum: expected_columns,
            });
        }
        let mut columns = value.columns.into_iter();
        let first = columns.next().ok_or(EventError::ValueOutOfRange {
            value: 0,
            maximum: 1,
        })?;
        let second = columns.next();
        let mut writer = PayloadWriter::new();
        writer.u8(value.menu_id);
        writer.u32(u32::from(value.two_columns));
        write_fixed_string32(&mut writer, value.title);
        encode_vector2(&mut writer, value.position);
        writer.f32(first.width);
        if let Some(column) = second.as_ref() {
            writer.f32(column.width);
        }
        writer.u32(u32::from(value.menu));
        for row in value.rows {
            write_i32(&mut writer, row);
        }
        encode_menu_column(&mut writer, first)?;
        if let Some(column) = second {
            encode_menu_column(&mut writer, column)?;
        }
        Ok(writer.finish_bits())
    }

    fn decode_interpolate_camera(event: &mut Event<'_>) -> Result<InterpolateCamera, EventError> {
        Ok(InterpolateCamera {
            set_position: read_bit_bool(event)?,
            from_position: decode_vector3(event)?,
            destination: decode_vector3(event)?,
            time_ms: decode_i32(event)?,
            mode: event.read_u8()?,
        })
    }

    fn encode_interpolate_camera(
        _api: HostApi,
        value: InterpolateCamera,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.bool(value.set_position);
        writer.vector3(value.from_position);
        writer.vector3(value.destination);
        write_i32(&mut writer, value.time_ms);
        writer.u8(value.mode);
        Ok(writer.finish_bits())
    }

    fn decode_toggle_select_text_draw(
        event: &mut Event<'_>,
    ) -> Result<ToggleSelectTextDraw, EventError> {
        Ok(ToggleSelectTextDraw {
            enabled: read_bit_bool(event)?,
            hover_color: decode_i32(event)?,
        })
    }

    fn encode_toggle_select_text_draw(
        _api: HostApi,
        value: ToggleSelectTextDraw,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.bool(value.enabled);
        write_i32(&mut writer, value.hover_color);
        Ok(writer.finish_bits())
    }

    fn decode_object_material_update(
        event: &mut Event<'_>,
    ) -> Result<ObjectMaterialUpdate, EventError> {
        Ok(ObjectMaterialUpdate {
            object_id: event.read_u16()?,
            material: decode_object_material(event)?,
        })
    }

    fn encode_object_material_update(
        api: HostApi,
        value: ObjectMaterialUpdate,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.object_id);
        encode_object_material(api, &mut writer, value.material)?;
        Ok(writer.finish_bits())
    }

    fn decode_animation(event: &mut Event<'_>) -> Result<Animation, EventError> {
        Ok(Animation {
            animation_library: event.read_string8()?,
            animation_name: event.read_string8()?,
            frame_delta: event.read_f32()?,
            looped: read_bit_bool(event)?,
            lock_x: read_bit_bool(event)?,
            lock_y: read_bit_bool(event)?,
            freeze: read_bit_bool(event)?,
            time: decode_i32(event)?,
        })
    }

    fn encode_animation(writer: &mut PayloadWriter, value: Animation) -> Result<(), EventError> {
        writer.string8(&value.animation_library)?;
        writer.string8(&value.animation_name)?;
        writer.f32(value.frame_delta);
        writer.bool(value.looped);
        writer.bool(value.lock_x);
        writer.bool(value.lock_y);
        writer.bool(value.freeze);
        write_i32(writer, value.time);
        Ok(())
    }

    fn decode_player_animation(event: &mut Event<'_>) -> Result<PlayerAnimation, EventError> {
        Ok(PlayerAnimation {
            player_id: event.read_u16()?,
            animation: decode_animation(event)?,
        })
    }

    fn encode_player_animation(
        _api: HostApi,
        value: PlayerAnimation,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        encode_animation(&mut writer, value.animation)?;
        Ok(writer.finish_bits())
    }

    fn decode_actor_animation(event: &mut Event<'_>) -> Result<ActorAnimation, EventError> {
        Ok(ActorAnimation {
            actor_id: event.read_u16()?,
            animation: decode_animation(event)?,
        })
    }

    fn encode_actor_animation(
        _api: HostApi,
        value: ActorAnimation,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.actor_id);
        encode_animation(&mut writer, value.animation)?;
        Ok(writer.finish_bits())
    }

    fn decode_crime_report(event: &mut Event<'_>) -> Result<CrimeReport, EventError> {
        Ok(CrimeReport {
            suspect_id: event.read_u16()?,
            in_vehicle: decode_bool32(event)?,
            vehicle_model: decode_i32(event)?,
            vehicle_color: decode_i32(event)?,
            crime: decode_i32(event)?,
            coordinates: decode_vector3(event)?,
        })
    }

    fn encode_crime_report(
        _api: HostApi,
        value: CrimeReport,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.suspect_id);
        writer.u32(u32::from(value.in_vehicle));
        write_i32(&mut writer, value.vehicle_model);
        write_i32(&mut writer, value.vehicle_color);
        write_i32(&mut writer, value.crime);
        writer.vector3(value.coordinates);
        Ok(writer.finish_bits())
    }

    fn decode_attached_object(event: &mut Event<'_>) -> Result<AttachedObject, EventError> {
        Ok(AttachedObject {
            model_id: decode_i32(event)?,
            bone: decode_i32(event)?,
            offset: decode_vector3(event)?,
            rotation: decode_vector3(event)?,
            scale: decode_vector3(event)?,
            color1: decode_i32(event)?,
            color2: decode_i32(event)?,
        })
    }

    fn encode_attached_object(writer: &mut PayloadWriter, value: AttachedObject) {
        write_i32(writer, value.model_id);
        write_i32(writer, value.bone);
        writer.vector3(value.offset);
        writer.vector3(value.rotation);
        writer.vector3(value.scale);
        write_i32(writer, value.color1);
        write_i32(writer, value.color2);
    }

    fn decode_player_attached_object(
        event: &mut Event<'_>,
    ) -> Result<PlayerAttachedObject, EventError> {
        let player_id = event.read_u16()?;
        let index = decode_i32(event)?;
        let create = read_bit_bool(event)?;
        let object = create.then(|| decode_attached_object(event)).transpose()?;
        Ok(PlayerAttachedObject {
            player_id,
            index,
            object,
        })
    }

    fn encode_player_attached_object(
        _api: HostApi,
        value: PlayerAttachedObject,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        write_i32(&mut writer, value.index);
        writer.bool(value.object.is_some());
        if let Some(object) = value.object {
            encode_attached_object(&mut writer, object);
        }
        Ok(writer.finish_bits())
    }

    fn decode_enter_edit_object(event: &mut Event<'_>) -> Result<EnterEditObject, EventError> {
        Ok(EnterEditObject {
            player_object: read_bit_bool(event)?,
            object_id: event.read_u16()?,
        })
    }

    fn encode_enter_edit_object(
        _api: HostApi,
        value: EnterEditObject,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.bool(value.player_object);
        writer.u16(value.object_id);
        Ok(writer.finish_bits())
    }

    fn decode_show_text_draw(event: &mut Event<'_>) -> Result<ShowTextDraw, EventError> {
        let textdraw_id = event.read_u16()?;
        let textdraw = TextDraw {
            flags: event.read_u8()?,
            letter_width: event.read_f32()?,
            letter_height: event.read_f32()?,
            letter_color: decode_i32(event)?,
            line_width: event.read_f32()?,
            line_height: event.read_f32()?,
            box_color: decode_i32(event)?,
            shadow: event.read_u8()?,
            outline: event.read_u8()?,
            background_color: decode_i32(event)?,
            style: event.read_u8()?,
            selectable: event.read_u8()?,
            position: decode_vector2(event)?,
            model_id: event.read_u16()?,
            rotation: decode_vector3(event)?,
            zoom: event.read_f32()?,
            color1: read_i16(event)?,
            color2: read_i16(event)?,
            text: {
                let length = usize::from(event.read_u16()?);
                if length > MAX_STRING32_BYTES {
                    return Err(EventError::LengthExceedsLimit {
                        length,
                        limit: MAX_STRING32_BYTES,
                    });
                }
                event.read_bytes(length)?
            },
        };
        Ok(ShowTextDraw {
            textdraw_id,
            textdraw,
        })
    }

    fn encode_show_text_draw(
        _api: HostApi,
        value: ShowTextDraw,
    ) -> Result<EncodedPayload, EventError> {
        if value.textdraw.text.len() > MAX_STRING32_BYTES {
            return Err(EventError::LengthExceedsLimit {
                length: value.textdraw.text.len(),
                limit: MAX_STRING32_BYTES,
            });
        }
        let textdraw = value.textdraw;
        let mut writer = PayloadWriter::new();
        writer.u16(value.textdraw_id);
        writer.u8(textdraw.flags);
        writer.f32(textdraw.letter_width);
        writer.f32(textdraw.letter_height);
        write_i32(&mut writer, textdraw.letter_color);
        writer.f32(textdraw.line_width);
        writer.f32(textdraw.line_height);
        write_i32(&mut writer, textdraw.box_color);
        writer.u8(textdraw.shadow);
        writer.u8(textdraw.outline);
        write_i32(&mut writer, textdraw.background_color);
        writer.u8(textdraw.style);
        writer.u8(textdraw.selectable);
        encode_vector2(&mut writer, textdraw.position);
        writer.u16(textdraw.model_id);
        writer.vector3(textdraw.rotation);
        writer.f32(textdraw.zoom);
        writer.i16(textdraw.color1);
        writer.i16(textdraw.color2);
        writer.u16(textdraw.text.len() as u16);
        writer.bytes(&textdraw.text);
        Ok(writer.finish_bits())
    }

    fn decode_scores_and_pings(event: &mut Event<'_>) -> Result<ScoresAndPings, EventError> {
        let bit_len = event.remaining_bits();
        if !bit_len.is_multiple_of(80) {
            return Err(EventError::UnexpectedBitLength {
                bit_len,
                expected: 80,
            });
        }
        let count = bit_len / 80;
        if count > MAX_SCORE_PING_ENTRIES {
            return Err(EventError::LengthExceedsLimit {
                length: count,
                limit: MAX_SCORE_PING_ENTRIES,
            });
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            entries.push(ScorePing {
                player_id: event.read_u16()?,
                score: decode_i32(event)?,
                ping: decode_i32(event)?,
            });
        }
        Ok(ScoresAndPings { entries })
    }

    fn encode_scores_and_pings(
        _api: HostApi,
        value: ScoresAndPings,
    ) -> Result<EncodedPayload, EventError> {
        if value.entries.len() > MAX_SCORE_PING_ENTRIES {
            return Err(EventError::LengthExceedsLimit {
                length: value.entries.len(),
                limit: MAX_SCORE_PING_ENTRIES,
            });
        }
        let mut writer = PayloadWriter::new();
        for entry in value.entries {
            writer.u16(entry.player_id);
            write_i32(&mut writer, entry.score);
            write_i32(&mut writer, entry.ping);
        }
        Ok(writer.finish_bits())
    }

    fn decode_vehicle_stream_in(event: &mut Event<'_>) -> Result<VehicleStreamIn, EventError> {
        Ok(VehicleStreamIn {
            vehicle_id: event.read_u16()?,
            vehicle: StreamedVehicle {
                model: decode_i32(event)?,
                position: decode_vector3(event)?,
                rotation: event.read_f32()?,
                body_color1: event.read_u8()?,
                body_color2: event.read_u8()?,
                health: event.read_f32()?,
                interior_id: event.read_u8()?,
                door_damage_status: decode_i32(event)?,
                panel_damage_status: decode_i32(event)?,
                light_damage_status: event.read_u8()?,
                tire_damage_status: event.read_u8()?,
                add_siren: event.read_u8()?,
                mod_slots: read_array(event)?,
                paint_job: event.read_u8()?,
                interior_color1: decode_i32(event)?,
                interior_color2: decode_i32(event)?,
            },
        })
    }

    fn encode_vehicle_stream_in(
        _api: HostApi,
        value: VehicleStreamIn,
    ) -> Result<EncodedPayload, EventError> {
        let vehicle = value.vehicle;
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        write_i32(&mut writer, vehicle.model);
        writer.vector3(vehicle.position);
        writer.f32(vehicle.rotation);
        writer.u8(vehicle.body_color1);
        writer.u8(vehicle.body_color2);
        writer.f32(vehicle.health);
        writer.u8(vehicle.interior_id);
        write_i32(&mut writer, vehicle.door_damage_status);
        write_i32(&mut writer, vehicle.panel_damage_status);
        writer.u8(vehicle.light_damage_status);
        writer.u8(vehicle.tire_damage_status);
        writer.u8(vehicle.add_siren);
        writer.bytes(&vehicle.mod_slots);
        writer.u8(vehicle.paint_job);
        write_i32(&mut writer, vehicle.interior_color1);
        write_i32(&mut writer, vehicle.interior_color2);
        Ok(writer.finish_bits())
    }

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

    fn decode_race_checkpoint(event: &mut Event<'_>) -> Result<RaceCheckpoint, EventError> {
        Ok(RaceCheckpoint {
            checkpoint_type: event.read_u8()?,
            position: decode_vector3(event)?,
            next_position: decode_vector3(event)?,
            size: event.read_f32()?,
        })
    }

    fn encode_race_checkpoint(value: RaceCheckpoint) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u8(value.checkpoint_type);
        writer.vector3(value.position);
        writer.vector3(value.next_position);
        writer.f32(value.size);
        Ok(writer.finish())
    }

    fn decode_audio_stream(event: &mut Event<'_>) -> Result<AudioStream, EventError> {
        Ok(AudioStream {
            url: event.read_string8()?,
            position: decode_vector3(event)?,
            radius: event.read_f32()?,
            use_position: decode_bool8(event)?,
        })
    }

    fn encode_audio_stream(value: AudioStream) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.string8(&value.url)?;
        writer.vector3(value.position);
        writer.f32(value.radius);
        writer.u8(u8::from(value.use_position));
        Ok(writer.finish())
    }

    fn decode_object_position(event: &mut Event<'_>) -> Result<ObjectPosition, EventError> {
        Ok(ObjectPosition {
            object_id: event.read_u16()?,
            position: decode_vector3(event)?,
        })
    }

    fn encode_object_position(value: ObjectPosition) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.object_id);
        writer.vector3(value.position);
        Ok(writer.finish())
    }

    fn decode_object_rotation(event: &mut Event<'_>) -> Result<ObjectRotation, EventError> {
        Ok(ObjectRotation {
            object_id: event.read_u16()?,
            rotation: decode_vector3(event)?,
        })
    }

    fn encode_object_rotation(value: ObjectRotation) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.object_id);
        writer.vector3(value.rotation);
        Ok(writer.finish())
    }

    fn decode_player_death_notification(
        event: &mut Event<'_>,
    ) -> Result<PlayerDeathNotification, EventError> {
        Ok(PlayerDeathNotification {
            killer_id: event.read_u16()?,
            killed_id: event.read_u16()?,
            reason: event.read_u8()?,
        })
    }

    fn encode_player_death_notification(
        value: PlayerDeathNotification,
    ) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.killer_id);
        writer.u16(value.killed_id);
        writer.u8(value.reason);
        Ok(writer.finish())
    }

    fn decode_map_icon(event: &mut Event<'_>) -> Result<MapIcon, EventError> {
        Ok(MapIcon {
            icon_id: event.read_u8()?,
            position: decode_vector3(event)?,
            icon_type: event.read_u8()?,
            color: decode_i32(event)?,
            style: event.read_u8()?,
        })
    }

    fn encode_map_icon(value: MapIcon) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u8(value.icon_id);
        writer.vector3(value.position);
        writer.u8(value.icon_type);
        writer.u32(value.color as u32);
        writer.u8(value.style);
        Ok(writer.finish())
    }

    fn decode_vehicle_component(event: &mut Event<'_>) -> Result<VehicleComponent, EventError> {
        Ok(VehicleComponent {
            vehicle_id: event.read_u16()?,
            component_id: event.read_u16()?,
        })
    }

    fn encode_vehicle_component(value: VehicleComponent) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.u16(value.component_id);
        Ok(writer.finish())
    }

    fn decode_vehicle_interior(event: &mut Event<'_>) -> Result<VehicleInterior, EventError> {
        Ok(VehicleInterior {
            vehicle_id: event.read_u16()?,
            interior_id: event.read_u8()?,
        })
    }

    fn encode_vehicle_interior(value: VehicleInterior) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.u8(value.interior_id);
        Ok(writer.finish())
    }

    fn decode_player_color(event: &mut Event<'_>) -> Result<PlayerColor, EventError> {
        Ok(PlayerColor {
            player_id: event.read_u16()?,
            color: decode_i32(event)?,
        })
    }

    fn encode_player_color(value: PlayerColor) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u32(value.color as u32);
        Ok(writer.finish())
    }

    fn decode_fixed_string32(event: &mut Event<'_>) -> Result<[u8; 32], EventError> {
        event
            .read_bytes(32)?
            .try_into()
            .map_err(|_| EventError::Host(RakRsResult::NativeCallFailed))
    }

    fn encode_fixed_string32(value: [u8; 32]) -> Result<Vec<u8>, EventError> {
        Ok(value.to_vec())
    }

    fn decode_player_skill(event: &mut Event<'_>) -> Result<PlayerSkill, EventError> {
        Ok(PlayerSkill {
            player_id: event.read_u16()?,
            skill: decode_i32(event)?,
            level: event.read_u16()?,
        })
    }

    fn encode_player_skill(value: PlayerSkill) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u32(value.skill as u32);
        writer.u16(value.level);
        Ok(writer.finish())
    }

    fn decode_remove_building(event: &mut Event<'_>) -> Result<RemoveBuilding, EventError> {
        Ok(RemoveBuilding {
            model_id: decode_i32(event)?,
            position: decode_vector3(event)?,
            radius: event.read_f32()?,
        })
    }

    fn encode_remove_building(value: RemoveBuilding) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.model_id as u32);
        writer.vector3(value.position);
        writer.f32(value.radius);
        Ok(writer.finish())
    }

    fn decode_attach_object_to_player(
        event: &mut Event<'_>,
    ) -> Result<AttachObjectToPlayer, EventError> {
        Ok(AttachObjectToPlayer {
            object_id: event.read_u16()?,
            player_id: event.read_u16()?,
            offsets: decode_vector3(event)?,
            rotation: decode_vector3(event)?,
        })
    }

    fn encode_attach_object_to_player(value: AttachObjectToPlayer) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.object_id);
        writer.u16(value.player_id);
        writer.vector3(value.offsets);
        writer.vector3(value.rotation);
        Ok(writer.finish())
    }

    fn decode_explosion(event: &mut Event<'_>) -> Result<Explosion, EventError> {
        Ok(Explosion {
            position: decode_vector3(event)?,
            style: decode_i32(event)?,
            radius: event.read_f32()?,
        })
    }

    fn encode_explosion(value: Explosion) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.vector3(value.position);
        writer.u32(value.style as u32);
        writer.f32(value.radius);
        Ok(writer.finish())
    }

    fn decode_player_name_tag(event: &mut Event<'_>) -> Result<PlayerNameTag, EventError> {
        Ok(PlayerNameTag {
            player_id: event.read_u16()?,
            show: decode_bool8(event)?,
        })
    }

    fn encode_player_name_tag(value: PlayerNameTag) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u8(u8::from(value.show));
        Ok(writer.finish())
    }

    fn decode_player_fighting_style(
        event: &mut Event<'_>,
    ) -> Result<PlayerFightingStyle, EventError> {
        Ok(PlayerFightingStyle {
            player_id: event.read_u16()?,
            style_id: event.read_u8()?,
        })
    }

    fn encode_player_fighting_style(value: PlayerFightingStyle) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u8(value.style_id);
        Ok(writer.finish())
    }

    fn decode_vehicle_velocity(event: &mut Event<'_>) -> Result<VehicleVelocity, EventError> {
        Ok(VehicleVelocity {
            turn: decode_bool8(event)?,
            velocity: decode_vector3(event)?,
        })
    }

    fn encode_vehicle_velocity(value: VehicleVelocity) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u8(u8::from(value.turn));
        writer.vector3(value.velocity);
        Ok(writer.finish())
    }

    fn decode_pickup(event: &mut Event<'_>) -> Result<Pickup, EventError> {
        Ok(Pickup {
            id: decode_i32(event)?,
            model: decode_i32(event)?,
            pickup_type: decode_i32(event)?,
            position: decode_vector3(event)?,
        })
    }

    fn encode_pickup(value: Pickup) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.id as u32);
        writer.u32(value.model as u32);
        writer.u32(value.pickup_type as u32);
        writer.vector3(value.position);
        Ok(writer.finish())
    }

    fn decode_move_object(event: &mut Event<'_>) -> Result<MoveObject, EventError> {
        Ok(MoveObject {
            object_id: event.read_u16()?,
            from_position: decode_vector3(event)?,
            destination: decode_vector3(event)?,
            speed: event.read_f32()?,
            rotation: decode_vector3(event)?,
        })
    }

    fn encode_move_object(value: MoveObject) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.object_id);
        writer.vector3(value.from_position);
        writer.vector3(value.destination);
        writer.f32(value.speed);
        writer.vector3(value.rotation);
        Ok(writer.finish())
    }

    fn decode_text_draw_string(event: &mut Event<'_>) -> Result<TextDrawString, EventError> {
        let textdraw_id = event.read_u16()?;
        let length = usize::from(event.read_u16()?);
        if length > MAX_STRING32_BYTES {
            return Err(EventError::LengthExceedsLimit {
                length,
                limit: MAX_STRING32_BYTES,
            });
        }
        Ok(TextDrawString {
            textdraw_id,
            text: event.read_bytes(length)?,
        })
    }

    fn encode_text_draw_string(value: TextDrawString) -> Result<Vec<u8>, EventError> {
        if value.text.len() > MAX_STRING32_BYTES {
            return Err(EventError::LengthExceedsLimit {
                length: value.text.len(),
                limit: MAX_STRING32_BYTES,
            });
        }
        let mut writer = PayloadWriter::new();
        writer.u16(value.textdraw_id);
        writer.u16(value.text.len() as u16);
        writer.bytes(&value.text);
        Ok(writer.finish())
    }

    fn decode_vector2(event: &mut Event<'_>) -> Result<Vector2, EventError> {
        Ok(Vector2 {
            x: event.read_f32()?,
            y: event.read_f32()?,
        })
    }

    fn encode_vector2(writer: &mut PayloadWriter, value: Vector2) {
        writer.f32(value.x);
        writer.f32(value.y);
    }

    fn decode_gang_zone(event: &mut Event<'_>) -> Result<GangZone, EventError> {
        Ok(GangZone {
            zone_id: event.read_u16()?,
            square_start: decode_vector2(event)?,
            square_end: decode_vector2(event)?,
            color: decode_i32(event)?,
        })
    }

    fn encode_gang_zone(value: GangZone) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.zone_id);
        encode_vector2(&mut writer, value.square_start);
        encode_vector2(&mut writer, value.square_end);
        writer.u32(value.color as u32);
        Ok(writer.finish())
    }

    fn decode_u16_i32(event: &mut Event<'_>) -> Result<(u16, i32), EventError> {
        Ok((event.read_u16()?, decode_i32(event)?))
    }

    fn encode_u16_i32(value: (u16, i32)) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.0);
        writer.u32(value.1 as u32);
        Ok(writer.finish())
    }

    fn decode_vehicle_number_plate(
        event: &mut Event<'_>,
    ) -> Result<VehicleNumberPlate, EventError> {
        Ok(VehicleNumberPlate {
            vehicle_id: event.read_u16()?,
            text: event.read_string8()?,
        })
    }

    fn encode_vehicle_number_plate(value: VehicleNumberPlate) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.string8(&value.text)?;
        Ok(writer.finish())
    }

    fn decode_spectate(event: &mut Event<'_>) -> Result<Spectate, EventError> {
        Ok(Spectate {
            target_id: event.read_u16()?,
            camera_type: event.read_u8()?,
        })
    }

    fn encode_spectate(value: Spectate) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.target_id);
        writer.u8(value.camera_type);
        Ok(writer.finish())
    }

    fn decode_weapon_ammo(event: &mut Event<'_>) -> Result<WeaponAmmo, EventError> {
        Ok(WeaponAmmo {
            weapon_id: event.read_u8()?,
            ammo: event.read_u16()?,
        })
    }

    fn encode_weapon_ammo(value: WeaponAmmo) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u8(value.weapon_id);
        writer.u16(value.ammo);
        Ok(writer.finish())
    }

    fn decode_trailer_attachment(event: &mut Event<'_>) -> Result<TrailerAttachment, EventError> {
        Ok(TrailerAttachment {
            trailer_id: event.read_u16()?,
            vehicle_id: event.read_u16()?,
        })
    }

    fn encode_trailer_attachment(value: TrailerAttachment) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.trailer_id);
        writer.u16(value.vehicle_id);
        Ok(writer.finish())
    }

    fn decode_camera_look_at(event: &mut Event<'_>) -> Result<CameraLookAt, EventError> {
        Ok(CameraLookAt {
            position: decode_vector3(event)?,
            cut_type: event.read_u8()?,
        })
    }

    fn encode_camera_look_at(value: CameraLookAt) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.vector3(value.position);
        writer.u8(value.cut_type);
        Ok(writer.finish())
    }

    fn decode_vehicle_params(event: &mut Event<'_>) -> Result<VehicleParams, EventError> {
        Ok(VehicleParams {
            vehicle_id: event.read_u16()?,
            objective: decode_bool8(event)?,
            doors_locked: decode_bool8(event)?,
        })
    }

    fn encode_vehicle_params(value: VehicleParams) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.u8(u8::from(value.objective));
        writer.u8(u8::from(value.doors_locked));
        Ok(writer.finish())
    }

    fn decode_player_enter_vehicle(
        event: &mut Event<'_>,
    ) -> Result<PlayerEnterVehicle, EventError> {
        Ok(PlayerEnterVehicle {
            player_id: event.read_u16()?,
            vehicle_id: event.read_u16()?,
            passenger: decode_bool8(event)?,
        })
    }

    fn encode_player_enter_vehicle(value: PlayerEnterVehicle) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u16(value.vehicle_id);
        writer.u8(u8::from(value.passenger));
        Ok(writer.finish())
    }

    fn decode_player_exit_vehicle(event: &mut Event<'_>) -> Result<PlayerExitVehicle, EventError> {
        Ok(PlayerExitVehicle {
            player_id: event.read_u16()?,
            vehicle_id: event.read_u16()?,
        })
    }

    fn encode_player_exit_vehicle(value: PlayerExitVehicle) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u16(value.vehicle_id);
        Ok(writer.finish())
    }

    fn decode_client_check(event: &mut Event<'_>) -> Result<ClientCheck, EventError> {
        Ok(ClientCheck {
            request_type: event.read_u8()?,
            subject: decode_i32(event)?,
            offset: event.read_u16()?,
            length: event.read_u16()?,
        })
    }

    fn encode_client_check(value: ClientCheck) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u8(value.request_type);
        writer.u32(value.subject as u32);
        writer.u16(value.offset);
        writer.u16(value.length);
        Ok(writer.finish())
    }

    fn read_array<const N: usize>(event: &mut Event<'_>) -> Result<[u8; N], EventError> {
        event
            .read_bytes(N)?
            .try_into()
            .map_err(|_| EventError::Host(RakRsResult::NativeCallFailed))
    }

    fn decode_vehicle_params_ex(event: &mut Event<'_>) -> Result<VehicleParamsEx, EventError> {
        Ok(VehicleParamsEx {
            vehicle_id: event.read_u16()?,
            params: read_array(event)?,
            doors: read_array(event)?,
            windows: read_array(event)?,
        })
    }

    fn encode_vehicle_params_ex(value: VehicleParamsEx) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.bytes(&value.params);
        writer.bytes(&value.doors);
        writer.bytes(&value.windows);
        Ok(writer.finish())
    }

    fn decode_vehicle_tuning_notification(
        event: &mut Event<'_>,
    ) -> Result<VehicleTuningNotification, EventError> {
        Ok(VehicleTuningNotification {
            player_id: event.read_u16()?,
            event: decode_i32(event)?,
            vehicle_id: decode_i32(event)?,
            param1: decode_i32(event)?,
            param2: decode_i32(event)?,
        })
    }

    fn encode_vehicle_tuning_notification(
        value: VehicleTuningNotification,
    ) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u32(value.event as u32);
        writer.u32(value.vehicle_id as u32);
        writer.u32(value.param1 as u32);
        writer.u32(value.param2 as u32);
        Ok(writer.finish())
    }

    fn decode_u16_u8(event: &mut Event<'_>) -> Result<(u16, u8), EventError> {
        Ok((event.read_u16()?, event.read_u8()?))
    }

    fn encode_u16_u8(value: (u16, u8)) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.0);
        writer.u8(value.1);
        Ok(writer.finish())
    }

    fn decode_vehicle_damage_status(
        event: &mut Event<'_>,
    ) -> Result<VehicleDamageStatus, EventError> {
        Ok(VehicleDamageStatus {
            vehicle_id: event.read_u16()?,
            panel_damage: decode_i32(event)?,
            door_damage: decode_i32(event)?,
            lights: event.read_u8()?,
            tires: event.read_u8()?,
        })
    }

    fn encode_vehicle_damage_status(value: VehicleDamageStatus) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.u32(value.panel_damage as u32);
        writer.u32(value.door_damage as u32);
        writer.u8(value.lights);
        writer.u8(value.tires);
        Ok(writer.finish())
    }

    fn decode_actor(event: &mut Event<'_>) -> Result<Actor, EventError> {
        Ok(Actor {
            actor_id: event.read_u16()?,
            skin_id: decode_i32(event)?,
            position: decode_vector3(event)?,
            rotation: event.read_f32()?,
            health: event.read_f32()?,
        })
    }

    fn encode_actor(value: Actor) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.actor_id);
        writer.u32(value.skin_id as u32);
        writer.vector3(value.position);
        writer.f32(value.rotation);
        writer.f32(value.health);
        Ok(writer.finish())
    }

    fn decode_actor_angle(event: &mut Event<'_>) -> Result<ActorAngle, EventError> {
        Ok(ActorAngle {
            actor_id: event.read_u16()?,
            angle: event.read_f32()?,
        })
    }

    fn encode_actor_angle(value: ActorAngle) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.actor_id);
        writer.f32(value.angle);
        Ok(writer.finish())
    }

    fn decode_actor_position(event: &mut Event<'_>) -> Result<ActorPosition, EventError> {
        Ok(ActorPosition {
            actor_id: event.read_u16()?,
            position: decode_vector3(event)?,
        })
    }

    fn encode_actor_position(value: ActorPosition) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.actor_id);
        writer.vector3(value.position);
        Ok(writer.finish())
    }

    fn decode_actor_health(event: &mut Event<'_>) -> Result<ActorHealth, EventError> {
        Ok(ActorHealth {
            actor_id: event.read_u16()?,
            health: event.read_f32()?,
        })
    }

    fn encode_actor_health(value: ActorHealth) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.actor_id);
        writer.f32(value.health);
        Ok(writer.finish())
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

    fn decode_player_skin(event: &mut Event<'_>) -> Result<PlayerSkin, EventError> {
        Ok(PlayerSkin {
            player_id: decode_i32(event)?,
            skin_id: decode_i32(event)?,
        })
    }

    fn encode_player_skin(value: PlayerSkin) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.player_id as u32);
        writer.u32(value.skin_id as u32);
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

    /// MoonLoader's `onSendClientJoin` payload (RPC 25).
    #[derive(Clone, Debug, PartialEq)]
    pub struct ClientJoin {
        pub version: i32,
        pub mod_id: u8,
        pub nickname: Vec<u8>,
        pub challenge_response: i32,
        pub join_auth_key: Vec<u8>,
        pub client_version: Vec<u8>,
        pub challenge_response2: i32,
    }

    /// MoonLoader's `onSendEnterEditObject` payload (RPC 27).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct EnterEditObject {
        pub object_type: i32,
        pub object_id: u16,
        pub model_id: i32,
        pub position: Vector3,
    }

    /// MoonLoader's `onSendVehicleTuningNotification` payload (RPC 96).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleTuning {
        pub vehicle_id: i32,
        pub param1: i32,
        pub param2: i32,
        pub event: i32,
    }

    /// MoonLoader's `onSendClientCheckResponse` payload (RPC 103).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ClientCheckResponse {
        pub request_type: u8,
        pub result1: i32,
        pub result2: u8,
    }

    /// MoonLoader's `onSendVehicleDamaged` payload (RPC 106).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleDamage {
        pub vehicle_id: u16,
        pub panel_damage: i32,
        pub door_damage: i32,
        pub lights: u8,
        pub tires: u8,
    }

    /// MoonLoader's `onSendEditAttachedObject` payload (RPC 116).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct EditAttachedObject {
        pub response: i32,
        pub index: i32,
        pub model_id: i32,
        pub bone: i32,
        pub position: Vector3,
        pub rotation: Vector3,
        pub scale: Vector3,
        pub color1: i32,
        pub color2: i32,
    }

    /// MoonLoader's `onSendEditObject` payload (RPC 117).
    ///
    /// `player_object` is a one-bit RakNet boolean; replacements preserve that exact-bit layout.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct EditObject {
        pub player_object: bool,
        pub object_id: u16,
        pub response: i32,
        pub position: Vector3,
        pub rotation: Vector3,
    }

    /// MoonLoader's shared `onSendGiveDamage` / `onSendTakeDamage` payload (RPC 115).
    ///
    /// `take` is a one-bit RakNet boolean. `false` identifies give-damage traffic and `true`
    /// identifies take-damage traffic.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Damage {
        pub player_id: u16,
        pub damage: f32,
        pub weapon: i32,
        pub body_part: i32,
        pub take: bool,
    }

    /// MoonLoader's `onSendMoneyIncreaseNotification` payload (RPC 31).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct MoneyIncrease {
        pub amount: i32,
        pub increase_type: i32,
    }

    /// MoonLoader's `onSendNPCJoin` payload (RPC 54).
    #[derive(Clone, Debug, PartialEq)]
    pub struct NpcJoin {
        pub version: i32,
        pub mod_id: u8,
        pub nickname: Vec<u8>,
        pub challenge_response: i32,
    }

    /// MoonLoader's `onSendCameraTargetUpdate` payload (RPC 168).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct CameraTargetUpdate {
        pub object_id: u16,
        pub vehicle_id: u16,
        pub player_id: u16,
        pub actor_id: u16,
    }

    /// MoonLoader's `onSendGiveActorDamage` payload (RPC 177).
    ///
    /// `unused` is a one-bit RakNet boolean retained for wire compatibility.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ActorDamage {
        pub unused: bool,
        pub actor_id: u16,
        pub damage: f32,
        pub weapon: i32,
        pub body_part: i32,
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
    /// The `onSendClientJoin` descriptor.
    pub const SEND_CLIENT_JOIN: Rpc<ClientJoin> =
        Rpc::new(25, decode_client_join, encode_client_join);
    /// The `onSendEnterEditObject` descriptor.
    pub const SEND_ENTER_EDIT_OBJECT: Rpc<EnterEditObject> =
        Rpc::new(27, decode_enter_edit_object, encode_enter_edit_object);
    /// The `onSendMoneyIncreaseNotification` descriptor.
    pub const SEND_MONEY_INCREASE: Rpc<MoneyIncrease> =
        Rpc::new(31, decode_money_increase, encode_money_increase);
    /// The `onSendNPCJoin` descriptor.
    pub const SEND_NPC_JOIN: Rpc<NpcJoin> = Rpc::new(54, decode_npc_join, encode_npc_join);
    /// The `onSendVehicleTuningNotification` descriptor.
    pub const SEND_VEHICLE_TUNING: Rpc<VehicleTuning> =
        Rpc::new(96, decode_vehicle_tuning, encode_vehicle_tuning);
    /// The `onSendPickedUpWeapon` descriptor.
    pub const SEND_PICKED_UP_WEAPON: Rpc<u16> = Rpc::new(97, decode_u16, encode_u16);
    /// The `onSendServerStatisticsRequest` descriptor.
    pub const SEND_SERVER_STATISTICS_REQUEST: Rpc<()> = Rpc::new(102, decode_empty, encode_empty);
    /// The `onSendClientCheckResponse` descriptor.
    pub const SEND_CLIENT_CHECK_RESPONSE: Rpc<ClientCheckResponse> = Rpc::new(
        103,
        decode_client_check_response,
        encode_client_check_response,
    );
    /// The `onSendVehicleDamaged` descriptor.
    pub const SEND_VEHICLE_DAMAGED: Rpc<VehicleDamage> =
        Rpc::new(106, decode_vehicle_damage, encode_vehicle_damage);
    /// The shared `onSendGiveDamage` / `onSendTakeDamage` descriptor.
    pub const SEND_DAMAGE: Rpc<Damage> = Rpc::new_bits(115, decode_damage, encode_damage);
    /// The `onSendEditAttachedObject` descriptor.
    pub const SEND_EDIT_ATTACHED_OBJECT: Rpc<EditAttachedObject> = Rpc::new(
        116,
        decode_edit_attached_object,
        encode_edit_attached_object,
    );
    /// The `onSendEditObject` descriptor.
    pub const SEND_EDIT_OBJECT: Rpc<EditObject> =
        Rpc::new_bits(117, decode_edit_object, encode_edit_object);
    /// The `onSendPickedUpPickup` descriptor.
    pub const SEND_PICKED_UP_PICKUP: Rpc<i32> = Rpc::new(131, decode_i32, encode_i32);
    /// The `onSendQuitMenu` descriptor.
    pub const SEND_QUIT_MENU: Rpc<()> = Rpc::new(140, decode_empty, encode_empty);
    /// The `onSendCameraTargetUpdate` descriptor.
    pub const SEND_CAMERA_TARGET_UPDATE: Rpc<CameraTargetUpdate> = Rpc::new(
        168,
        decode_camera_target_update,
        encode_camera_target_update,
    );
    /// The `onSendGiveActorDamage` descriptor.
    pub const SEND_GIVE_ACTOR_DAMAGE: Rpc<ActorDamage> =
        Rpc::new_bits(177, decode_actor_damage, encode_actor_damage);

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
    rpc_helper!(
        on_send_client_join,
        ClientJoin,
        SEND_CLIENT_JOIN,
        "onSendClientJoin"
    );
    rpc_helper!(
        on_send_enter_edit_object,
        EnterEditObject,
        SEND_ENTER_EDIT_OBJECT,
        "onSendEnterEditObject"
    );
    rpc_helper!(
        on_send_money_increase,
        MoneyIncrease,
        SEND_MONEY_INCREASE,
        "onSendMoneyIncreaseNotification"
    );
    rpc_helper!(on_send_npc_join, NpcJoin, SEND_NPC_JOIN, "onSendNPCJoin");
    rpc_helper!(
        on_send_vehicle_tuning,
        VehicleTuning,
        SEND_VEHICLE_TUNING,
        "onSendVehicleTuningNotification"
    );
    rpc_helper!(
        on_send_picked_up_weapon,
        u16,
        SEND_PICKED_UP_WEAPON,
        "onSendPickedUpWeapon"
    );
    rpc_helper!(
        on_send_server_statistics_request,
        (),
        SEND_SERVER_STATISTICS_REQUEST,
        "onSendServerStatisticsRequest"
    );
    rpc_helper!(
        on_send_client_check_response,
        ClientCheckResponse,
        SEND_CLIENT_CHECK_RESPONSE,
        "onSendClientCheckResponse"
    );
    rpc_helper!(
        on_send_vehicle_damaged,
        VehicleDamage,
        SEND_VEHICLE_DAMAGED,
        "onSendVehicleDamaged"
    );
    rpc_helper!(
        on_send_edit_attached_object,
        EditAttachedObject,
        SEND_EDIT_ATTACHED_OBJECT,
        "onSendEditAttachedObject"
    );
    rpc_helper!(
        on_send_edit_object,
        EditObject,
        SEND_EDIT_OBJECT,
        "onSendEditObject"
    );
    rpc_helper!(
        on_send_picked_up_pickup,
        i32,
        SEND_PICKED_UP_PICKUP,
        "onSendPickedUpPickup"
    );
    rpc_helper!(on_send_quit_menu, (), SEND_QUIT_MENU, "onSendQuitMenu");
    rpc_helper!(
        on_send_camera_target_update,
        CameraTargetUpdate,
        SEND_CAMERA_TARGET_UPDATE,
        "onSendCameraTargetUpdate"
    );
    rpc_helper!(
        on_send_give_actor_damage,
        ActorDamage,
        SEND_GIVE_ACTOR_DAMAGE,
        "onSendGiveActorDamage"
    );

    /// Handles `onSendGiveDamage` from an outgoing raw RPC callback.
    ///
    /// # Safety
    ///
    /// See [`super::handle`].
    pub unsafe fn on_send_give_damage(
        api: HostApi,
        raw: *mut RakRsEventV1,
        handler: impl FnOnce(Damage) -> RpcAction<Damage>,
    ) -> Result<RakRsHookAction, EventError> {
        unsafe {
            handle(api, raw, SEND_DAMAGE, |value| {
                if value.take {
                    RpcAction::Continue
                } else {
                    handler(value)
                }
            })
        }
    }

    /// Handles `onSendTakeDamage` from an outgoing raw RPC callback.
    ///
    /// # Safety
    ///
    /// See [`super::handle`].
    pub unsafe fn on_send_take_damage(
        api: HostApi,
        raw: *mut RakRsEventV1,
        handler: impl FnOnce(Damage) -> RpcAction<Damage>,
    ) -> Result<RakRsHookAction, EventError> {
        unsafe {
            handle(api, raw, SEND_DAMAGE, |value| {
                if value.take {
                    handler(value)
                } else {
                    RpcAction::Continue
                }
            })
        }
    }

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

    fn decode_client_join(event: &mut Event<'_>) -> Result<ClientJoin, EventError> {
        Ok(ClientJoin {
            version: decode_i32(event)?,
            mod_id: event.read_u8()?,
            nickname: event.read_string8()?,
            challenge_response: decode_i32(event)?,
            join_auth_key: event.read_string8()?,
            client_version: event.read_string8()?,
            challenge_response2: decode_i32(event)?,
        })
    }

    fn encode_client_join(value: ClientJoin) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.version as u32);
        writer.u8(value.mod_id);
        writer.string8(&value.nickname)?;
        writer.u32(value.challenge_response as u32);
        writer.string8(&value.join_auth_key)?;
        writer.string8(&value.client_version)?;
        writer.u32(value.challenge_response2 as u32);
        Ok(writer.finish())
    }

    fn decode_enter_edit_object(event: &mut Event<'_>) -> Result<EnterEditObject, EventError> {
        Ok(EnterEditObject {
            object_type: decode_i32(event)?,
            object_id: event.read_u16()?,
            model_id: decode_i32(event)?,
            position: decode_vector3(event)?,
        })
    }

    fn encode_enter_edit_object(value: EnterEditObject) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.object_type as u32);
        writer.u16(value.object_id);
        writer.u32(value.model_id as u32);
        writer.vector3(value.position);
        Ok(writer.finish())
    }

    fn decode_money_increase(event: &mut Event<'_>) -> Result<MoneyIncrease, EventError> {
        Ok(MoneyIncrease {
            amount: decode_i32(event)?,
            increase_type: decode_i32(event)?,
        })
    }

    fn encode_money_increase(value: MoneyIncrease) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.amount as u32);
        writer.u32(value.increase_type as u32);
        Ok(writer.finish())
    }

    fn decode_npc_join(event: &mut Event<'_>) -> Result<NpcJoin, EventError> {
        Ok(NpcJoin {
            version: decode_i32(event)?,
            mod_id: event.read_u8()?,
            nickname: event.read_string8()?,
            challenge_response: decode_i32(event)?,
        })
    }

    fn encode_npc_join(value: NpcJoin) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.version as u32);
        writer.u8(value.mod_id);
        writer.string8(&value.nickname)?;
        writer.u32(value.challenge_response as u32);
        Ok(writer.finish())
    }

    fn decode_vehicle_tuning(event: &mut Event<'_>) -> Result<VehicleTuning, EventError> {
        Ok(VehicleTuning {
            vehicle_id: decode_i32(event)?,
            param1: decode_i32(event)?,
            param2: decode_i32(event)?,
            event: decode_i32(event)?,
        })
    }

    fn encode_vehicle_tuning(value: VehicleTuning) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.vehicle_id as u32);
        writer.u32(value.param1 as u32);
        writer.u32(value.param2 as u32);
        writer.u32(value.event as u32);
        Ok(writer.finish())
    }

    fn decode_client_check_response(
        event: &mut Event<'_>,
    ) -> Result<ClientCheckResponse, EventError> {
        Ok(ClientCheckResponse {
            request_type: event.read_u8()?,
            result1: decode_i32(event)?,
            result2: event.read_u8()?,
        })
    }

    fn encode_client_check_response(value: ClientCheckResponse) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u8(value.request_type);
        writer.u32(value.result1 as u32);
        writer.u8(value.result2);
        Ok(writer.finish())
    }

    fn decode_vehicle_damage(event: &mut Event<'_>) -> Result<VehicleDamage, EventError> {
        Ok(VehicleDamage {
            vehicle_id: event.read_u16()?,
            panel_damage: decode_i32(event)?,
            door_damage: decode_i32(event)?,
            lights: event.read_u8()?,
            tires: event.read_u8()?,
        })
    }

    fn encode_vehicle_damage(value: VehicleDamage) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.u32(value.panel_damage as u32);
        writer.u32(value.door_damage as u32);
        writer.u8(value.lights);
        writer.u8(value.tires);
        Ok(writer.finish())
    }

    fn decode_edit_attached_object(
        event: &mut Event<'_>,
    ) -> Result<EditAttachedObject, EventError> {
        Ok(EditAttachedObject {
            response: decode_i32(event)?,
            index: decode_i32(event)?,
            model_id: decode_i32(event)?,
            bone: decode_i32(event)?,
            position: decode_vector3(event)?,
            rotation: decode_vector3(event)?,
            scale: decode_vector3(event)?,
            color1: decode_i32(event)?,
            color2: decode_i32(event)?,
        })
    }

    fn encode_edit_attached_object(value: EditAttachedObject) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.response as u32);
        writer.u32(value.index as u32);
        writer.u32(value.model_id as u32);
        writer.u32(value.bone as u32);
        writer.vector3(value.position);
        writer.vector3(value.rotation);
        writer.vector3(value.scale);
        writer.u32(value.color1 as u32);
        writer.u32(value.color2 as u32);
        Ok(writer.finish())
    }

    fn decode_bool(event: &mut Event<'_>) -> Result<bool, EventError> {
        Ok(event.read_bits(1)?[0] & 0x80 != 0)
    }

    fn decode_edit_object(event: &mut Event<'_>) -> Result<EditObject, EventError> {
        Ok(EditObject {
            player_object: decode_bool(event)?,
            object_id: event.read_u16()?,
            response: decode_i32(event)?,
            position: decode_vector3(event)?,
            rotation: decode_vector3(event)?,
        })
    }

    fn encode_edit_object(_api: HostApi, value: EditObject) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.bit(value.player_object);
        writer.u16(value.object_id);
        writer.u32(value.response as u32);
        writer.vector3(value.position);
        writer.vector3(value.rotation);
        Ok(writer.finish_bits())
    }

    fn decode_damage(event: &mut Event<'_>) -> Result<Damage, EventError> {
        Ok(Damage {
            take: decode_bool(event)?,
            player_id: event.read_u16()?,
            damage: event.read_f32()?,
            weapon: decode_i32(event)?,
            body_part: decode_i32(event)?,
        })
    }

    fn encode_damage(_api: HostApi, value: Damage) -> Result<EncodedPayload, EventError> {
        encode_damage_payload(value)
    }

    pub(super) fn encode_damage_payload(value: Damage) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.bit(value.take);
        writer.u16(value.player_id);
        writer.f32(value.damage);
        writer.u32(value.weapon as u32);
        writer.u32(value.body_part as u32);
        Ok(writer.finish_bits())
    }

    fn decode_camera_target_update(
        event: &mut Event<'_>,
    ) -> Result<CameraTargetUpdate, EventError> {
        Ok(CameraTargetUpdate {
            object_id: event.read_u16()?,
            vehicle_id: event.read_u16()?,
            player_id: event.read_u16()?,
            actor_id: event.read_u16()?,
        })
    }

    fn encode_camera_target_update(value: CameraTargetUpdate) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.object_id);
        writer.u16(value.vehicle_id);
        writer.u16(value.player_id);
        writer.u16(value.actor_id);
        Ok(writer.finish())
    }

    fn decode_actor_damage(event: &mut Event<'_>) -> Result<ActorDamage, EventError> {
        Ok(ActorDamage {
            unused: decode_bool(event)?,
            actor_id: event.read_u16()?,
            damage: event.read_f32()?,
            weapon: decode_i32(event)?,
            body_part: decode_i32(event)?,
        })
    }

    fn encode_actor_damage(
        _api: HostApi,
        value: ActorDamage,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.bit(value.unused);
        writer.u16(value.actor_id);
        writer.f32(value.damage);
        writer.u32(value.weapon as u32);
        writer.u32(value.body_part as u32);
        Ok(writer.finish_bits())
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

/// Typed fixed-layout RakNet packet helpers.
///
/// The helpers in this module operate on raw packet subscriptions, not RPC subscriptions. They
/// cover packet layouts that are fixed and byte-aligned in the SA-MP protocol. The packed flag
/// bytes are intentionally exposed without splitting bit fields: their bit order is protocol
/// data, not a Rust memory layout.
pub mod packet {
    use super::*;

    /// SA-MP sends at most 13 weapon slots in one weapons-update packet.
    pub const MAX_WEAPON_SLOTS: usize = 13;
    /// R1 marker packets cannot contain more players than the protocol player-slot limit.
    pub const MAX_MARKERS: usize = 1_000;

    pub const AUTHENTICATION_ID: u8 = 12;
    pub const CONNECTION_ATTEMPT_FAILED_ID: u8 = 29;
    pub const NO_FREE_INCOMING_CONNECTIONS_ID: u8 = 31;
    pub const DISCONNECTION_NOTIFICATION_ID: u8 = 32;
    pub const CONNECTION_LOST_ID: u8 = 33;
    pub const CONNECTION_REQUEST_ACCEPTED_ID: u8 = 34;
    pub const CONNECTION_BANNED_ID: u8 = 36;
    pub const INVALID_PASSWORD_ID: u8 = 37;
    pub const VEHICLE_SYNC_ID: u8 = 200;
    pub const RCON_COMMAND_ID: u8 = 201;
    pub const AIM_SYNC_ID: u8 = 203;
    pub const STATS_UPDATE_ID: u8 = 205;
    pub const BULLET_SYNC_ID: u8 = 206;
    pub const PLAYER_SYNC_ID: u8 = 207;
    pub const MARKERS_SYNC_ID: u8 = 208;
    pub const UNOCCUPIED_SYNC_ID: u8 = 209;
    pub const TRAILER_SYNC_ID: u8 = 210;
    pub const PASSENGER_SYNC_ID: u8 = 211;
    pub const SPECTATOR_SYNC_ID: u8 = 212;

    /// The byte-aligned `ID_STATS_UPDATE` payload (packet 205).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct StatsUpdate {
        pub money: i32,
        pub drunk_level: i32,
    }

    /// One four-byte entry in an `ID_WEAPONS_UPDATE` packet.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct WeaponSlot {
        pub slot: u8,
        pub weapon: u8,
        pub ammo: u16,
    }

    /// MoonLoader's `onSendWeaponsUpdate` payload (packet 204).
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct WeaponsUpdate {
        pub player_target: u16,
        pub actor_target: u16,
        pub weapons: Vec<WeaponSlot>,
    }

    /// MoonLoader's `onConnectionRequestAccepted` payload (packet 34).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ConnectionAccepted {
        pub ip: i32,
        pub port: u16,
        pub player_id: u16,
        pub challenge: i32,
    }

    /// The fixed 68-byte local `ID_PLAYER_SYNC` payload (packet 207).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PlayerSync {
        pub left_right_keys: u16,
        pub up_down_keys: u16,
        /// Protocol-defined key bits, preserved as received.
        pub key_data: u16,
        pub position: Vector3,
        pub quaternion: [f32; 4],
        pub health: u8,
        pub armour: u8,
        /// Protocol-defined weapon and special-key bits, preserved as received.
        pub weapon_and_special_key: u8,
        pub special_action: u8,
        pub move_speed: Vector3,
        pub surfing_offsets: Vector3,
        pub surfing_vehicle_id: u16,
        pub animation_id: u16,
        pub animation_flags: u16,
    }

    /// The fixed 63-byte local `ID_VEHICLE_SYNC` payload (packet 200).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct VehicleSync {
        pub vehicle_id: u16,
        pub left_right_keys: u16,
        pub up_down_keys: u16,
        /// Protocol-defined key bits, preserved as received.
        pub key_data: u16,
        pub quaternion: [f32; 4],
        pub position: Vector3,
        pub move_speed: Vector3,
        pub vehicle_health: f32,
        pub player_health: u8,
        pub armour: u8,
        /// Protocol-defined weapon and special-key bits, preserved as received.
        pub weapon_and_special_key: u8,
        pub siren: u8,
        pub landing_gear_state: u8,
        pub trailer_id: u16,
        /// Four protocol bytes used as bike lean, train speed, or Hydra thrust data.
        pub vehicle_specific: [u8; 4],
    }

    /// The optional surfing data carried by an incoming remote player-sync packet.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct RemotePlayerSurfing {
        pub vehicle_id: u16,
        pub offsets: Vector3,
    }

    /// The optional animation data carried by an incoming remote player-sync packet.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct RemotePlayerAnimation {
        pub id: u16,
        pub flags: u16,
    }

    /// The compressed R1 `ID_PLAYER_SYNC` layout received from a remote player (packet 207).
    ///
    /// Unlike local outgoing [`PlayerSync`], its optional key, surf, and animation fields are
    /// serialized as individual bits and the velocity and quaternion are compressed.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct RemotePlayerSync {
        pub player_id: u16,
        pub left_right_keys: Option<u16>,
        pub up_down_keys: Option<u16>,
        pub key_data: u16,
        pub position: Vector3,
        pub quaternion: [f32; 4],
        pub health: u8,
        pub armour: u8,
        pub weapon: u8,
        pub special_action: u8,
        pub move_speed: Vector3,
        pub surfing: Option<RemotePlayerSurfing>,
        pub animation: Option<RemotePlayerAnimation>,
    }

    /// The compressed R1 `ID_VEHICLE_SYNC` layout received from a remote player (packet 200).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct RemoteVehicleSync {
        pub player_id: u16,
        pub vehicle_id: u16,
        pub left_right_keys: u16,
        pub up_down_keys: u16,
        pub key_data: u16,
        pub quaternion: [f32; 4],
        pub position: Vector3,
        pub move_speed: Vector3,
        pub vehicle_health: u16,
        pub player_health: u8,
        pub armour: u8,
        pub current_weapon: u8,
        pub siren: bool,
        pub landing_gear: bool,
        pub train_speed: Option<i32>,
        pub trailer_id: Option<u16>,
    }

    /// Signed R1 marker coordinates. They are not floating-point world coordinates.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct MarkerCoordinates {
        pub x: i16,
        pub y: i16,
        pub z: i16,
    }

    /// One R1 player marker, active or inactive.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Marker {
        pub player_id: u16,
        pub coordinates: Option<MarkerCoordinates>,
    }

    /// The R1 `ID_MARKERS_SYNC` payload (packet 208).
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct MarkersSync {
        pub markers: Vec<Marker>,
    }

    /// The fixed 24-byte `ID_PASSENGER_SYNC` payload (packet 211), excluding a remote player ID.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct PassengerSync {
        pub vehicle_id: u16,
        /// Protocol-defined seat, drive-by, and cuffed bits, preserved as received.
        pub seat_driveby_cuffed: u8,
        /// Protocol-defined weapon and special-key bits, preserved as received.
        pub weapon_and_special_key: u8,
        pub health: u8,
        pub armour: u8,
        pub left_right_keys: u16,
        pub up_down_keys: u16,
        pub key_data: u16,
        pub position: Vector3,
    }

    /// The fixed 31-byte `ID_AIM_SYNC` payload (packet 203), excluding a remote player ID.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct AimSync {
        pub camera_mode: u8,
        pub camera_front: Vector3,
        pub camera_position: Vector3,
        pub aim_z: f32,
        /// Protocol-defined camera-zoom and weapon-state bits, preserved as received.
        pub zoom_and_weapon_state: u8,
        pub aspect_ratio: u8,
    }

    /// The fixed 67-byte `ID_UNOCCUPIED_SYNC` payload (packet 209), excluding a remote player ID.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct UnoccupiedSync {
        pub vehicle_id: u16,
        pub seat_id: u8,
        pub roll: Vector3,
        pub direction: Vector3,
        pub position: Vector3,
        pub move_speed: Vector3,
        pub turn_speed: Vector3,
        pub vehicle_health: f32,
    }

    /// The fixed 54-byte `ID_TRAILER_SYNC` payload (packet 210), excluding a remote player ID.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TrailerSync {
        pub trailer_id: u16,
        pub position: Vector3,
        pub quaternion: [f32; 4],
        pub move_speed: Vector3,
        pub turn_speed: Vector3,
    }

    /// The fixed 40-byte `ID_BULLET_SYNC` payload (packet 206), excluding a remote player ID.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct BulletSync {
        pub target_type: u8,
        pub target_id: u16,
        pub origin: Vector3,
        pub target: Vector3,
        pub center: Vector3,
        pub weapon_id: u8,
    }

    /// The fixed 18-byte local `ID_SPECTATOR_SYNC` payload (packet 212).
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct SpectatorSync {
        pub left_right_keys: u16,
        pub up_down_keys: u16,
        pub key_data: u16,
        pub position: Vector3,
    }

    /// Synchronization data received from a specific remote player.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct RemoteSync<T> {
        pub player_id: u16,
        pub data: T,
    }

    fn require_exact_bytes(event: &Event<'_>, bytes: usize) -> Result<(), EventError> {
        let expected = bytes * u8::BITS as usize;
        let bit_len = event.remaining_bits();
        if bit_len == expected {
            Ok(())
        } else {
            Err(EventError::UnexpectedBitLength { bit_len, expected })
        }
    }

    fn decode_vector3(event: &mut Event<'_>) -> Result<Vector3, EventError> {
        Ok(Vector3 {
            x: event.read_f32()?,
            y: event.read_f32()?,
            z: event.read_f32()?,
        })
    }

    fn write_vector3(writer: &mut PayloadWriter, value: Vector3) {
        writer.vector3(value);
    }

    fn decode_quaternion(event: &mut Event<'_>) -> Result<[f32; 4], EventError> {
        Ok([
            event.read_f32()?,
            event.read_f32()?,
            event.read_f32()?,
            event.read_f32()?,
        ])
    }

    fn write_quaternion(writer: &mut PayloadWriter, value: [f32; 4]) {
        for component in value {
            writer.f32(component);
        }
    }

    fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
        Ok(event.read_u32()? as i32)
    }

    fn decode_stats_update(event: &mut Event<'_>) -> Result<StatsUpdate, EventError> {
        require_exact_bytes(event, 8)?;
        Ok(StatsUpdate {
            money: decode_i32(event)?,
            drunk_level: decode_i32(event)?,
        })
    }

    fn encode_stats_update(value: StatsUpdate) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.money as u32);
        writer.u32(value.drunk_level as u32);
        Ok(writer.finish())
    }

    fn decode_weapons_update(event: &mut Event<'_>) -> Result<WeaponsUpdate, EventError> {
        let bit_len = event.remaining_bits();
        if bit_len < 32 || !(bit_len - 32).is_multiple_of(32) {
            return Err(EventError::UnexpectedBitLength {
                bit_len,
                expected: 32,
            });
        }
        let weapon_count = (bit_len - 32) / 32;
        if weapon_count > MAX_WEAPON_SLOTS {
            return Err(EventError::LengthExceedsLimit {
                length: weapon_count,
                limit: MAX_WEAPON_SLOTS,
            });
        }
        let player_target = event.read_u16()?;
        let actor_target = event.read_u16()?;
        let mut weapons = Vec::with_capacity(weapon_count);
        for _ in 0..weapon_count {
            weapons.push(WeaponSlot {
                slot: event.read_u8()?,
                weapon: event.read_u8()?,
                ammo: event.read_u16()?,
            });
        }
        Ok(WeaponsUpdate {
            player_target,
            actor_target,
            weapons,
        })
    }

    fn encode_weapons_update(value: WeaponsUpdate) -> Result<Vec<u8>, EventError> {
        if value.weapons.len() > MAX_WEAPON_SLOTS {
            return Err(EventError::LengthExceedsLimit {
                length: value.weapons.len(),
                limit: MAX_WEAPON_SLOTS,
            });
        }
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_target);
        writer.u16(value.actor_target);
        for weapon in value.weapons {
            writer.u8(weapon.slot);
            writer.u8(weapon.weapon);
            writer.u16(weapon.ammo);
        }
        Ok(writer.finish())
    }

    fn decode_string8(event: &mut Event<'_>) -> Result<Vec<u8>, EventError> {
        let value = event.read_string8()?;
        let bit_len = event.remaining_bits();
        if bit_len == 0 {
            Ok(value)
        } else {
            Err(EventError::UnexpectedBitLength {
                bit_len,
                expected: 0,
            })
        }
    }

    fn encode_string8(value: Vec<u8>) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.string8(&value)?;
        Ok(writer.finish())
    }

    fn decode_connection_accepted(event: &mut Event<'_>) -> Result<ConnectionAccepted, EventError> {
        require_exact_bytes(event, 12)?;
        Ok(ConnectionAccepted {
            ip: decode_i32(event)?,
            port: event.read_u16()?,
            player_id: event.read_u16()?,
            challenge: decode_i32(event)?,
        })
    }

    fn encode_connection_accepted(value: ConnectionAccepted) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u32(value.ip as u32);
        writer.u16(value.port);
        writer.u16(value.player_id);
        writer.u32(value.challenge as u32);
        Ok(writer.finish())
    }

    fn decode_empty(event: &mut Event<'_>) -> Result<(), EventError> {
        require_exact_bytes(event, 0)
    }

    fn encode_empty(_value: ()) -> Result<Vec<u8>, EventError> {
        Ok(Vec::new())
    }

    fn decode_player_sync(event: &mut Event<'_>) -> Result<PlayerSync, EventError> {
        require_exact_bytes(event, 68)?;
        Ok(PlayerSync {
            left_right_keys: event.read_u16()?,
            up_down_keys: event.read_u16()?,
            key_data: event.read_u16()?,
            position: decode_vector3(event)?,
            quaternion: decode_quaternion(event)?,
            health: event.read_u8()?,
            armour: event.read_u8()?,
            weapon_and_special_key: event.read_u8()?,
            special_action: event.read_u8()?,
            move_speed: decode_vector3(event)?,
            surfing_offsets: decode_vector3(event)?,
            surfing_vehicle_id: event.read_u16()?,
            animation_id: event.read_u16()?,
            animation_flags: event.read_u16()?,
        })
    }

    fn encode_player_sync(value: PlayerSync) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.left_right_keys);
        writer.u16(value.up_down_keys);
        writer.u16(value.key_data);
        write_vector3(&mut writer, value.position);
        write_quaternion(&mut writer, value.quaternion);
        writer.u8(value.health);
        writer.u8(value.armour);
        writer.u8(value.weapon_and_special_key);
        writer.u8(value.special_action);
        write_vector3(&mut writer, value.move_speed);
        write_vector3(&mut writer, value.surfing_offsets);
        writer.u16(value.surfing_vehicle_id);
        writer.u16(value.animation_id);
        writer.u16(value.animation_flags);
        Ok(writer.finish())
    }

    fn decode_vehicle_sync(event: &mut Event<'_>) -> Result<VehicleSync, EventError> {
        require_exact_bytes(event, 63)?;
        Ok(VehicleSync {
            vehicle_id: event.read_u16()?,
            left_right_keys: event.read_u16()?,
            up_down_keys: event.read_u16()?,
            key_data: event.read_u16()?,
            quaternion: decode_quaternion(event)?,
            position: decode_vector3(event)?,
            move_speed: decode_vector3(event)?,
            vehicle_health: event.read_f32()?,
            player_health: event.read_u8()?,
            armour: event.read_u8()?,
            weapon_and_special_key: event.read_u8()?,
            siren: event.read_u8()?,
            landing_gear_state: event.read_u8()?,
            trailer_id: event.read_u16()?,
            vehicle_specific: event
                .read_bytes(4)?
                .try_into()
                .map_err(|_| EventError::Host(RakRsResult::NativeCallFailed))?,
        })
    }

    fn encode_vehicle_sync(value: VehicleSync) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.u16(value.left_right_keys);
        writer.u16(value.up_down_keys);
        writer.u16(value.key_data);
        write_quaternion(&mut writer, value.quaternion);
        write_vector3(&mut writer, value.position);
        write_vector3(&mut writer, value.move_speed);
        writer.f32(value.vehicle_health);
        writer.u8(value.player_health);
        writer.u8(value.armour);
        writer.u8(value.weapon_and_special_key);
        writer.u8(value.siren);
        writer.u8(value.landing_gear_state);
        writer.u16(value.trailer_id);
        writer.bytes(&value.vehicle_specific);
        Ok(writer.finish())
    }

    fn decode_passenger_sync(event: &mut Event<'_>) -> Result<PassengerSync, EventError> {
        require_exact_bytes(event, 24)?;
        Ok(PassengerSync {
            vehicle_id: event.read_u16()?,
            seat_driveby_cuffed: event.read_u8()?,
            weapon_and_special_key: event.read_u8()?,
            health: event.read_u8()?,
            armour: event.read_u8()?,
            left_right_keys: event.read_u16()?,
            up_down_keys: event.read_u16()?,
            key_data: event.read_u16()?,
            position: decode_vector3(event)?,
        })
    }

    fn encode_passenger_sync(value: PassengerSync) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.u8(value.seat_driveby_cuffed);
        writer.u8(value.weapon_and_special_key);
        writer.u8(value.health);
        writer.u8(value.armour);
        writer.u16(value.left_right_keys);
        writer.u16(value.up_down_keys);
        writer.u16(value.key_data);
        write_vector3(&mut writer, value.position);
        Ok(writer.finish())
    }

    fn decode_aim_sync(event: &mut Event<'_>) -> Result<AimSync, EventError> {
        require_exact_bytes(event, 31)?;
        Ok(AimSync {
            camera_mode: event.read_u8()?,
            camera_front: decode_vector3(event)?,
            camera_position: decode_vector3(event)?,
            aim_z: event.read_f32()?,
            zoom_and_weapon_state: event.read_u8()?,
            aspect_ratio: event.read_u8()?,
        })
    }

    fn encode_aim_sync(value: AimSync) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u8(value.camera_mode);
        write_vector3(&mut writer, value.camera_front);
        write_vector3(&mut writer, value.camera_position);
        writer.f32(value.aim_z);
        writer.u8(value.zoom_and_weapon_state);
        writer.u8(value.aspect_ratio);
        Ok(writer.finish())
    }

    fn decode_unoccupied_sync(event: &mut Event<'_>) -> Result<UnoccupiedSync, EventError> {
        require_exact_bytes(event, 67)?;
        Ok(UnoccupiedSync {
            vehicle_id: event.read_u16()?,
            seat_id: event.read_u8()?,
            roll: decode_vector3(event)?,
            direction: decode_vector3(event)?,
            position: decode_vector3(event)?,
            move_speed: decode_vector3(event)?,
            turn_speed: decode_vector3(event)?,
            vehicle_health: event.read_f32()?,
        })
    }

    fn encode_unoccupied_sync(value: UnoccupiedSync) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.vehicle_id);
        writer.u8(value.seat_id);
        write_vector3(&mut writer, value.roll);
        write_vector3(&mut writer, value.direction);
        write_vector3(&mut writer, value.position);
        write_vector3(&mut writer, value.move_speed);
        write_vector3(&mut writer, value.turn_speed);
        writer.f32(value.vehicle_health);
        Ok(writer.finish())
    }

    fn decode_trailer_sync(event: &mut Event<'_>) -> Result<TrailerSync, EventError> {
        require_exact_bytes(event, 54)?;
        Ok(TrailerSync {
            trailer_id: event.read_u16()?,
            position: decode_vector3(event)?,
            quaternion: decode_quaternion(event)?,
            move_speed: decode_vector3(event)?,
            turn_speed: decode_vector3(event)?,
        })
    }

    fn encode_trailer_sync(value: TrailerSync) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.trailer_id);
        write_vector3(&mut writer, value.position);
        write_quaternion(&mut writer, value.quaternion);
        write_vector3(&mut writer, value.move_speed);
        write_vector3(&mut writer, value.turn_speed);
        Ok(writer.finish())
    }

    fn decode_bullet_sync(event: &mut Event<'_>) -> Result<BulletSync, EventError> {
        require_exact_bytes(event, 40)?;
        Ok(BulletSync {
            target_type: event.read_u8()?,
            target_id: event.read_u16()?,
            origin: decode_vector3(event)?,
            target: decode_vector3(event)?,
            center: decode_vector3(event)?,
            weapon_id: event.read_u8()?,
        })
    }

    fn encode_bullet_sync(value: BulletSync) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u8(value.target_type);
        writer.u16(value.target_id);
        write_vector3(&mut writer, value.origin);
        write_vector3(&mut writer, value.target);
        write_vector3(&mut writer, value.center);
        writer.u8(value.weapon_id);
        Ok(writer.finish())
    }

    fn decode_spectator_sync(event: &mut Event<'_>) -> Result<SpectatorSync, EventError> {
        require_exact_bytes(event, 18)?;
        Ok(SpectatorSync {
            left_right_keys: event.read_u16()?,
            up_down_keys: event.read_u16()?,
            key_data: event.read_u16()?,
            position: decode_vector3(event)?,
        })
    }

    fn encode_spectator_sync(value: SpectatorSync) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.left_right_keys);
        writer.u16(value.up_down_keys);
        writer.u16(value.key_data);
        write_vector3(&mut writer, value.position);
        Ok(writer.finish())
    }

    fn decode_rcon_command(event: &mut Event<'_>) -> Result<Vec<u8>, EventError> {
        let command = event.read_string32(MAX_STRING32_BYTES)?;
        if event.remaining_bits() == 0 {
            Ok(command)
        } else {
            Err(EventError::UnexpectedBitLength {
                bit_len: event.remaining_bits(),
                expected: 0,
            })
        }
    }

    fn encode_rcon_command(value: Vec<u8>) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.string32(&value)?;
        Ok(writer.finish())
    }

    fn decode_remote<T>(
        event: &mut Event<'_>,
        decode: fn(&mut Event<'_>) -> Result<T, EventError>,
    ) -> Result<RemoteSync<T>, EventError> {
        let player_id = event.read_u16()?;
        let data = decode(event)?;
        Ok(RemoteSync { player_id, data })
    }

    fn encode_remote<T>(
        value: RemoteSync<T>,
        encode: fn(T) -> Result<Vec<u8>, EventError>,
    ) -> Result<Vec<u8>, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.bytes(&encode(value.data)?);
        Ok(writer.finish())
    }

    fn decode_remote_aim_sync(event: &mut Event<'_>) -> Result<RemoteSync<AimSync>, EventError> {
        require_exact_bytes(event, 33)?;
        decode_remote(event, decode_aim_sync)
    }

    fn encode_remote_aim_sync(value: RemoteSync<AimSync>) -> Result<Vec<u8>, EventError> {
        encode_remote(value, encode_aim_sync)
    }

    fn decode_remote_bullet_sync(
        event: &mut Event<'_>,
    ) -> Result<RemoteSync<BulletSync>, EventError> {
        require_exact_bytes(event, 42)?;
        decode_remote(event, decode_bullet_sync)
    }

    fn encode_remote_bullet_sync(value: RemoteSync<BulletSync>) -> Result<Vec<u8>, EventError> {
        encode_remote(value, encode_bullet_sync)
    }

    fn decode_remote_unoccupied_sync(
        event: &mut Event<'_>,
    ) -> Result<RemoteSync<UnoccupiedSync>, EventError> {
        require_exact_bytes(event, 69)?;
        decode_remote(event, decode_unoccupied_sync)
    }

    fn encode_remote_unoccupied_sync(
        value: RemoteSync<UnoccupiedSync>,
    ) -> Result<Vec<u8>, EventError> {
        encode_remote(value, encode_unoccupied_sync)
    }

    fn decode_remote_trailer_sync(
        event: &mut Event<'_>,
    ) -> Result<RemoteSync<TrailerSync>, EventError> {
        require_exact_bytes(event, 56)?;
        decode_remote(event, decode_trailer_sync)
    }

    fn encode_remote_trailer_sync(value: RemoteSync<TrailerSync>) -> Result<Vec<u8>, EventError> {
        encode_remote(value, encode_trailer_sync)
    }

    fn decode_remote_passenger_sync(
        event: &mut Event<'_>,
    ) -> Result<RemoteSync<PassengerSync>, EventError> {
        require_exact_bytes(event, 26)?;
        decode_remote(event, decode_passenger_sync)
    }

    fn encode_remote_passenger_sync(
        value: RemoteSync<PassengerSync>,
    ) -> Result<Vec<u8>, EventError> {
        encode_remote(value, encode_passenger_sync)
    }

    fn read_bit_bool(event: &mut Event<'_>) -> Result<bool, EventError> {
        Ok(event.read_bits(1)?[0] & 0x80 != 0)
    }

    fn read_compressed_float(event: &mut Event<'_>) -> Result<f32, EventError> {
        Ok(f32::from(event.read_u16()?) / 32_767.5 - 1.0)
    }

    fn write_compressed_float(writer: &mut PayloadWriter, value: f32) {
        let value = value.clamp(-1.0, 1.0);
        writer.u16(((value + 1.0) * 32_767.5).floor() as u16);
    }

    fn decode_compressed_vector(event: &mut Event<'_>) -> Result<Vector3, EventError> {
        let magnitude = event.read_f32()?;
        if magnitude == 0.0 {
            return Ok(Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            });
        }
        Ok(Vector3 {
            x: read_compressed_float(event)? * magnitude,
            y: read_compressed_float(event)? * magnitude,
            z: read_compressed_float(event)? * magnitude,
        })
    }

    fn encode_compressed_vector(writer: &mut PayloadWriter, value: Vector3) {
        let magnitude = (value.x.mul_add(value.x, value.y * value.y) + value.z * value.z).sqrt();
        writer.f32(magnitude);
        if magnitude != 0.0 {
            write_compressed_float(writer, value.x / magnitude);
            write_compressed_float(writer, value.y / magnitude);
            write_compressed_float(writer, value.z / magnitude);
        }
    }

    fn decode_normalized_quaternion(event: &mut Event<'_>) -> Result<[f32; 4], EventError> {
        let w_negative = read_bit_bool(event)?;
        let x_negative = read_bit_bool(event)?;
        let y_negative = read_bit_bool(event)?;
        let z_negative = read_bit_bool(event)?;
        let mut x = f32::from(event.read_u16()?) / 65_535.0;
        let mut y = f32::from(event.read_u16()?) / 65_535.0;
        let mut z = f32::from(event.read_u16()?) / 65_535.0;
        if x_negative {
            x = -x;
        }
        if y_negative {
            y = -y;
        }
        if z_negative {
            z = -z;
        }
        let mut w = (1.0 - x * x - y * y - z * z).max(0.0).sqrt();
        if w_negative {
            w = -w;
        }
        Ok([w, x, y, z])
    }

    fn encode_normalized_quaternion(writer: &mut PayloadWriter, value: [f32; 4]) {
        let [w, x, y, z] = value;
        writer.bool(w < 0.0);
        writer.bool(x < 0.0);
        writer.bool(y < 0.0);
        writer.bool(z < 0.0);
        for component in [x, y, z] {
            writer.u16((component.abs().clamp(0.0, 1.0) * 65_535.0).floor() as u16);
        }
    }

    fn decode_health_armour(value: u8) -> (u8, u8) {
        (((value >> 4) * 7).min(100), ((value & 0x0F) * 7).min(100))
    }

    fn encode_health_armour(health: u8, armour: u8) -> u8 {
        let health = if health >= 100 {
            0xF0
        } else {
            (health / 7) << 4
        };
        let armour = if armour >= 100 { 0x0F } else { armour / 7 };
        health | armour
    }

    fn decode_remote_player_sync(event: &mut Event<'_>) -> Result<RemotePlayerSync, EventError> {
        let player_id = event.read_u16()?;
        let left_right_keys = read_bit_bool(event)?
            .then(|| event.read_u16())
            .transpose()?;
        let up_down_keys = read_bit_bool(event)?
            .then(|| event.read_u16())
            .transpose()?;
        let key_data = event.read_u16()?;
        let position = decode_vector3(event)?;
        let quaternion = decode_normalized_quaternion(event)?;
        let (health, armour) = decode_health_armour(event.read_u8()?);
        let weapon = event.read_u8()?;
        let special_action = event.read_u8()?;
        let move_speed = decode_compressed_vector(event)?;
        let surfing = read_bit_bool(event)?
            .then(|| {
                Ok(RemotePlayerSurfing {
                    vehicle_id: event.read_u16()?,
                    offsets: decode_vector3(event)?,
                })
            })
            .transpose()?;
        let animation = read_bit_bool(event)?
            .then(|| {
                Ok(RemotePlayerAnimation {
                    id: event.read_u16()?,
                    flags: event.read_u16()?,
                })
            })
            .transpose()?;
        Ok(RemotePlayerSync {
            player_id,
            left_right_keys,
            up_down_keys,
            key_data,
            position,
            quaternion,
            health,
            armour,
            weapon,
            special_action,
            move_speed,
            surfing,
            animation,
        })
    }

    fn encode_remote_player_sync(
        _api: HostApi,
        value: RemotePlayerSync,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.bool(value.left_right_keys.is_some());
        if let Some(keys) = value.left_right_keys {
            writer.u16(keys);
        }
        writer.bool(value.up_down_keys.is_some());
        if let Some(keys) = value.up_down_keys {
            writer.u16(keys);
        }
        writer.u16(value.key_data);
        write_vector3(&mut writer, value.position);
        encode_normalized_quaternion(&mut writer, value.quaternion);
        writer.u8(encode_health_armour(value.health, value.armour));
        writer.u8(value.weapon);
        writer.u8(value.special_action);
        encode_compressed_vector(&mut writer, value.move_speed);
        writer.bool(value.surfing.is_some());
        if let Some(surfing) = value.surfing {
            writer.u16(surfing.vehicle_id);
            write_vector3(&mut writer, surfing.offsets);
        }
        writer.bool(value.animation.is_some());
        if let Some(animation) = value.animation {
            writer.u16(animation.id);
            writer.u16(animation.flags);
        }
        Ok(writer.finish_bits())
    }

    fn decode_remote_vehicle_sync(event: &mut Event<'_>) -> Result<RemoteVehicleSync, EventError> {
        let player_id = event.read_u16()?;
        let vehicle_id = event.read_u16()?;
        let left_right_keys = event.read_u16()?;
        let up_down_keys = event.read_u16()?;
        let key_data = event.read_u16()?;
        let quaternion = decode_normalized_quaternion(event)?;
        let position = decode_vector3(event)?;
        let move_speed = decode_compressed_vector(event)?;
        let vehicle_health = event.read_u16()?;
        let (player_health, armour) = decode_health_armour(event.read_u8()?);
        let current_weapon = event.read_u8()?;
        let siren = read_bit_bool(event)?;
        let landing_gear = read_bit_bool(event)?;
        let train_speed = read_bit_bool(event)?
            .then(|| decode_i32(event))
            .transpose()?;
        let trailer_id = read_bit_bool(event)?
            .then(|| event.read_u16())
            .transpose()?;
        Ok(RemoteVehicleSync {
            player_id,
            vehicle_id,
            left_right_keys,
            up_down_keys,
            key_data,
            quaternion,
            position,
            move_speed,
            vehicle_health,
            player_health,
            armour,
            current_weapon,
            siren,
            landing_gear,
            train_speed,
            trailer_id,
        })
    }

    fn encode_remote_vehicle_sync(
        _api: HostApi,
        value: RemoteVehicleSync,
    ) -> Result<EncodedPayload, EventError> {
        let mut writer = PayloadWriter::new();
        writer.u16(value.player_id);
        writer.u16(value.vehicle_id);
        writer.u16(value.left_right_keys);
        writer.u16(value.up_down_keys);
        writer.u16(value.key_data);
        encode_normalized_quaternion(&mut writer, value.quaternion);
        write_vector3(&mut writer, value.position);
        encode_compressed_vector(&mut writer, value.move_speed);
        writer.u16(value.vehicle_health);
        writer.u8(encode_health_armour(value.player_health, value.armour));
        writer.u8(value.current_weapon);
        writer.bool(value.siren);
        writer.bool(value.landing_gear);
        writer.bool(value.train_speed.is_some());
        if let Some(train_speed) = value.train_speed {
            writer.u32(train_speed as u32);
        }
        writer.bool(value.trailer_id.is_some());
        if let Some(trailer_id) = value.trailer_id {
            writer.u16(trailer_id);
        }
        Ok(writer.finish_bits())
    }

    fn decode_markers_sync(event: &mut Event<'_>) -> Result<MarkersSync, EventError> {
        let count = decode_i32(event)?;
        let count = usize::try_from(count).map_err(|_| EventError::ValueOutOfRange {
            value: 0,
            maximum: MAX_MARKERS,
        })?;
        if count > MAX_MARKERS {
            return Err(EventError::LengthExceedsLimit {
                length: count,
                limit: MAX_MARKERS,
            });
        }
        let mut markers = Vec::with_capacity(count);
        for _ in 0..count {
            let player_id = event.read_u16()?;
            let coordinates = read_bit_bool(event)?
                .then(|| {
                    Ok(MarkerCoordinates {
                        x: event.read_u16()? as i16,
                        y: event.read_u16()? as i16,
                        z: event.read_u16()? as i16,
                    })
                })
                .transpose()?;
            markers.push(Marker {
                player_id,
                coordinates,
            });
        }
        consume_terminal_alignment_padding(event)?;
        Ok(MarkersSync { markers })
    }

    // R1 supplies `ID_MARKERS_SYNC` through a byte-backed `Packet`, even though each
    // marker has a one-bit active flag. Its advertised packet bit size consequently
    // includes up to seven terminal transport-padding bits after the final marker.
    // The packet buffer does not promise their value, so consume that sub-byte suffix;
    // a complete extra byte is semantic trailing data and remains malformed.
    fn consume_terminal_alignment_padding(event: &mut Event<'_>) -> Result<(), EventError> {
        let padding_bits = event.remaining_bits();
        if padding_bits == 0 {
            return Ok(());
        }
        if padding_bits >= u8::BITS as usize {
            return Err(EventError::UnexpectedBitLength {
                bit_len: padding_bits,
                expected: 0,
            });
        }
        let _ = event.read_bits(padding_bits)?;
        Ok(())
    }

    fn encode_markers_sync(
        _api: HostApi,
        value: MarkersSync,
    ) -> Result<EncodedPayload, EventError> {
        if value.markers.len() > MAX_MARKERS {
            return Err(EventError::LengthExceedsLimit {
                length: value.markers.len(),
                limit: MAX_MARKERS,
            });
        }
        let mut writer = PayloadWriter::new();
        writer.u32(value.markers.len() as u32);
        for marker in value.markers {
            writer.u16(marker.player_id);
            writer.bool(marker.coordinates.is_some());
            if let Some(coordinates) = marker.coordinates {
                writer.u16(coordinates.x as u16);
                writer.u16(coordinates.y as u16);
                writer.u16(coordinates.z as u16);
            }
        }
        Ok(writer.finish_bits())
    }

    /// Typed outgoing packet helpers.
    pub mod outgoing {
        use super::*;

        pub const SEND_RCON_COMMAND: Packet<Vec<u8>> =
            Packet::new(RCON_COMMAND_ID, decode_rcon_command, encode_rcon_command);
        /// The `onSendAuthenticationResponse` descriptor.
        pub const SEND_AUTHENTICATION_RESPONSE: Packet<Vec<u8>> =
            Packet::new(AUTHENTICATION_ID, decode_string8, encode_string8);
        pub const SEND_STATS_UPDATE: Packet<StatsUpdate> =
            Packet::new(STATS_UPDATE_ID, decode_stats_update, encode_stats_update);
        /// The `onSendWeaponsUpdate` descriptor.
        pub const SEND_WEAPONS_UPDATE: Packet<WeaponsUpdate> =
            Packet::new(204, decode_weapons_update, encode_weapons_update);
        pub const SEND_PLAYER_SYNC: Packet<PlayerSync> =
            Packet::new(PLAYER_SYNC_ID, decode_player_sync, encode_player_sync);
        pub const SEND_VEHICLE_SYNC: Packet<VehicleSync> =
            Packet::new(VEHICLE_SYNC_ID, decode_vehicle_sync, encode_vehicle_sync);
        pub const SEND_PASSENGER_SYNC: Packet<PassengerSync> = Packet::new(
            PASSENGER_SYNC_ID,
            decode_passenger_sync,
            encode_passenger_sync,
        );
        pub const SEND_AIM_SYNC: Packet<AimSync> =
            Packet::new(AIM_SYNC_ID, decode_aim_sync, encode_aim_sync);
        pub const SEND_UNOCCUPIED_SYNC: Packet<UnoccupiedSync> = Packet::new(
            UNOCCUPIED_SYNC_ID,
            decode_unoccupied_sync,
            encode_unoccupied_sync,
        );
        pub const SEND_TRAILER_SYNC: Packet<TrailerSync> =
            Packet::new(TRAILER_SYNC_ID, decode_trailer_sync, encode_trailer_sync);
        pub const SEND_BULLET_SYNC: Packet<BulletSync> =
            Packet::new(BULLET_SYNC_ID, decode_bullet_sync, encode_bullet_sync);
        pub const SEND_SPECTATOR_SYNC: Packet<SpectatorSync> = Packet::new(
            SPECTATOR_SYNC_ID,
            decode_spectator_sync,
            encode_spectator_sync,
        );

        macro_rules! packet_helper {
            ($name:ident, $value:ty, $packet:ident, $event_name:literal) => {
                #[doc = concat!("Handles MoonLoader's `", $event_name, "` from an outgoing raw packet callback.")]
                ///
                /// # Safety
                ///
                /// See [`super::super::handle`].
                pub unsafe fn $name(
                    api: HostApi,
                    raw: *mut RakRsEventV1,
                    handler: impl FnOnce($value) -> RpcAction<$value>,
                ) -> Result<RakRsHookAction, EventError> {
                    unsafe { handle(api, raw, $packet, handler) }
                }
            };
        }

        packet_helper!(
            on_send_rcon_command,
            Vec<u8>,
            SEND_RCON_COMMAND,
            "onSendRconCommand"
        );
        packet_helper!(
            on_send_authentication_response,
            Vec<u8>,
            SEND_AUTHENTICATION_RESPONSE,
            "onSendAuthenticationResponse"
        );
        packet_helper!(
            on_send_stats_update,
            StatsUpdate,
            SEND_STATS_UPDATE,
            "onSendStatsUpdate"
        );
        packet_helper!(
            on_send_weapons_update,
            WeaponsUpdate,
            SEND_WEAPONS_UPDATE,
            "onSendWeaponsUpdate"
        );
        packet_helper!(
            on_send_player_sync,
            PlayerSync,
            SEND_PLAYER_SYNC,
            "onSendPlayerSync"
        );
        packet_helper!(
            on_send_vehicle_sync,
            VehicleSync,
            SEND_VEHICLE_SYNC,
            "onSendVehicleSync"
        );
        packet_helper!(
            on_send_passenger_sync,
            PassengerSync,
            SEND_PASSENGER_SYNC,
            "onSendPassengerSync"
        );
        packet_helper!(on_send_aim_sync, AimSync, SEND_AIM_SYNC, "onSendAimSync");
        packet_helper!(
            on_send_unoccupied_sync,
            UnoccupiedSync,
            SEND_UNOCCUPIED_SYNC,
            "onSendUnoccupiedSync"
        );
        packet_helper!(
            on_send_trailer_sync,
            TrailerSync,
            SEND_TRAILER_SYNC,
            "onSendTrailerSync"
        );
        packet_helper!(
            on_send_bullet_sync,
            BulletSync,
            SEND_BULLET_SYNC,
            "onSendBulletSync"
        );
        packet_helper!(
            on_send_spectator_sync,
            SpectatorSync,
            SEND_SPECTATOR_SYNC,
            "onSendSpectatorSync"
        );
    }

    /// Typed incoming packet helpers.
    pub mod incoming {
        use super::*;

        /// The `onAuthenticationRequest` descriptor.
        pub const AUTHENTICATION_REQUEST: Packet<Vec<u8>> =
            Packet::new(AUTHENTICATION_ID, decode_string8, encode_string8);
        /// The `onConnectionRequestAccepted` descriptor.
        pub const CONNECTION_ACCEPTED: Packet<ConnectionAccepted> = Packet::new(
            CONNECTION_REQUEST_ACCEPTED_ID,
            decode_connection_accepted,
            encode_connection_accepted,
        );
        /// The `onConnectionLost` descriptor.
        pub const CONNECTION_LOST: Packet<()> =
            Packet::new(CONNECTION_LOST_ID, decode_empty, encode_empty);
        /// The `onConnectionBanned` descriptor.
        pub const CONNECTION_BANNED: Packet<()> =
            Packet::new(CONNECTION_BANNED_ID, decode_empty, encode_empty);
        /// The `onConnectionAttemptFailed` descriptor.
        pub const CONNECTION_ATTEMPT_FAILED: Packet<()> =
            Packet::new(CONNECTION_ATTEMPT_FAILED_ID, decode_empty, encode_empty);
        /// The `onConnectionNoFreeSlot` descriptor.
        pub const CONNECTION_NO_FREE_SLOT: Packet<()> =
            Packet::new(NO_FREE_INCOMING_CONNECTIONS_ID, decode_empty, encode_empty);
        /// The `onConnectionPasswordInvalid` descriptor.
        pub const CONNECTION_PASSWORD_INVALID: Packet<()> =
            Packet::new(INVALID_PASSWORD_ID, decode_empty, encode_empty);
        /// The `onConnectionClosed` descriptor.
        pub const CONNECTION_CLOSED: Packet<()> =
            Packet::new(DISCONNECTION_NOTIFICATION_ID, decode_empty, encode_empty);
        /// The compressed R1 remote-player sync descriptor.
        pub const PLAYER_SYNC: Packet<RemotePlayerSync> = Packet::new_bits(
            PLAYER_SYNC_ID,
            decode_remote_player_sync,
            encode_remote_player_sync,
        );
        /// The compressed R1 remote-vehicle sync descriptor.
        pub const VEHICLE_SYNC: Packet<RemoteVehicleSync> = Packet::new_bits(
            VEHICLE_SYNC_ID,
            decode_remote_vehicle_sync,
            encode_remote_vehicle_sync,
        );
        /// The variable-length R1 marker-sync descriptor.
        pub const MARKERS_SYNC: Packet<MarkersSync> =
            Packet::new_bits(MARKERS_SYNC_ID, decode_markers_sync, encode_markers_sync);
        pub const AIM_SYNC: Packet<RemoteSync<AimSync>> =
            Packet::new(AIM_SYNC_ID, decode_remote_aim_sync, encode_remote_aim_sync);
        pub const BULLET_SYNC: Packet<RemoteSync<BulletSync>> = Packet::new(
            BULLET_SYNC_ID,
            decode_remote_bullet_sync,
            encode_remote_bullet_sync,
        );
        pub const UNOCCUPIED_SYNC: Packet<RemoteSync<UnoccupiedSync>> = Packet::new(
            UNOCCUPIED_SYNC_ID,
            decode_remote_unoccupied_sync,
            encode_remote_unoccupied_sync,
        );
        pub const TRAILER_SYNC: Packet<RemoteSync<TrailerSync>> = Packet::new(
            TRAILER_SYNC_ID,
            decode_remote_trailer_sync,
            encode_remote_trailer_sync,
        );
        pub const PASSENGER_SYNC: Packet<RemoteSync<PassengerSync>> = Packet::new(
            PASSENGER_SYNC_ID,
            decode_remote_passenger_sync,
            encode_remote_passenger_sync,
        );

        macro_rules! packet_helper {
            ($name:ident, $value:ty, $packet:ident, $event_name:literal) => {
                #[doc = concat!("Handles MoonLoader's `", $event_name, "` from an incoming raw packet callback.")]
                ///
                /// # Safety
                ///
                /// See [`super::super::handle`].
                pub unsafe fn $name(
                    api: HostApi,
                    raw: *mut RakRsEventV1,
                    handler: impl FnOnce($value) -> RpcAction<$value>,
                ) -> Result<RakRsHookAction, EventError> {
                    unsafe { handle(api, raw, $packet, handler) }
                }
            };
        }

        packet_helper!(on_aim_sync, RemoteSync<AimSync>, AIM_SYNC, "onAimSync");
        packet_helper!(
            on_authentication_request,
            Vec<u8>,
            AUTHENTICATION_REQUEST,
            "onAuthenticationRequest"
        );
        packet_helper!(
            on_connection_accepted,
            ConnectionAccepted,
            CONNECTION_ACCEPTED,
            "onConnectionRequestAccepted"
        );
        packet_helper!(on_connection_lost, (), CONNECTION_LOST, "onConnectionLost");
        packet_helper!(
            on_connection_banned,
            (),
            CONNECTION_BANNED,
            "onConnectionBanned"
        );
        packet_helper!(
            on_connection_attempt_failed,
            (),
            CONNECTION_ATTEMPT_FAILED,
            "onConnectionAttemptFailed"
        );
        packet_helper!(
            on_connection_no_free_slot,
            (),
            CONNECTION_NO_FREE_SLOT,
            "onConnectionNoFreeSlot"
        );
        packet_helper!(
            on_connection_password_invalid,
            (),
            CONNECTION_PASSWORD_INVALID,
            "onConnectionPasswordInvalid"
        );
        packet_helper!(
            on_connection_closed,
            (),
            CONNECTION_CLOSED,
            "onConnectionClosed"
        );
        packet_helper!(
            on_player_sync,
            RemotePlayerSync,
            PLAYER_SYNC,
            "onPlayerSync"
        );
        packet_helper!(
            on_vehicle_sync,
            RemoteVehicleSync,
            VEHICLE_SYNC,
            "onVehicleSync"
        );
        packet_helper!(on_markers_sync, MarkersSync, MARKERS_SYNC, "onMarkersSync");
        packet_helper!(
            on_bullet_sync,
            RemoteSync<BulletSync>,
            BULLET_SYNC,
            "onBulletSync"
        );
        packet_helper!(
            on_unoccupied_sync,
            RemoteSync<UnoccupiedSync>,
            UNOCCUPIED_SYNC,
            "onUnoccupiedSync"
        );
        packet_helper!(
            on_trailer_sync,
            RemoteSync<TrailerSync>,
            TRAILER_SYNC,
            "onTrailerSync"
        );
        packet_helper!(
            on_passenger_sync,
            RemoteSync<PassengerSync>,
            PASSENGER_SYNC,
            "onPassengerSync"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::{mem, ptr};

    #[repr(C)]
    struct TestEvent {
        id: u8,
        bytes: Vec<u8>,
        bit_len: usize,
        read_offset: usize,
    }

    impl TestEvent {
        fn new(id: u8, payload: EncodedPayload) -> Self {
            Self {
                id,
                bytes: payload.bytes,
                bit_len: payload.bit_len,
                read_offset: 0,
            }
        }
    }

    unsafe fn test_event<'a>(event: *mut RakRsEventV1) -> &'a mut TestEvent {
        unsafe { &mut *event.cast::<TestEvent>() }
    }

    unsafe extern "system" fn test_event_id(event: *const RakRsEventV1) -> u8 {
        unsafe { (&*event.cast::<TestEvent>()).id }
    }

    unsafe extern "system" fn test_event_reset_read(event: *mut RakRsEventV1) -> RakRsResult {
        unsafe { test_event(event) }.read_offset = 0;
        RakRsResult::Ok
    }

    unsafe extern "system" fn test_event_clear(event: *mut RakRsEventV1) -> RakRsResult {
        let event = unsafe { test_event(event) };
        event.bytes.clear();
        event.bit_len = 0;
        event.read_offset = 0;
        RakRsResult::Ok
    }

    unsafe extern "system" fn test_event_read_bits(
        event: *mut RakRsEventV1,
        output: *mut u8,
        bit_len: usize,
    ) -> RakRsResult {
        let event = unsafe { test_event(event) };
        if event.read_offset.saturating_add(bit_len) > event.bit_len {
            return RakRsResult::ReadOutOfBounds;
        }
        let byte_len = bit_len.div_ceil(u8::BITS as usize);
        if byte_len != 0 {
            unsafe { ptr::write_bytes(output, 0, byte_len) };
        }
        for bit in 0..bit_len {
            let source = event.bytes[(event.read_offset + bit) / 8]
                & (0x80 >> ((event.read_offset + bit) % 8));
            if source != 0 {
                unsafe { *output.add(bit / 8) |= 0x80 >> (bit % 8) };
            }
        }
        event.read_offset += bit_len;
        RakRsResult::Ok
    }

    unsafe extern "system" fn test_event_read_u8(
        event: *mut RakRsEventV1,
        output: *mut u8,
    ) -> RakRsResult {
        unsafe { test_event_read_bits(event, output, 8) }
    }

    unsafe extern "system" fn test_event_read_u16(
        event: *mut RakRsEventV1,
        output: *mut u16,
    ) -> RakRsResult {
        let mut bytes = [0; 2];
        let result = unsafe { test_event_read_bits(event, bytes.as_mut_ptr(), 16) };
        if result == RakRsResult::Ok {
            unsafe { output.write(u16::from_le_bytes(bytes)) };
        }
        result
    }

    unsafe extern "system" fn test_event_read_u32(
        event: *mut RakRsEventV1,
        output: *mut u32,
    ) -> RakRsResult {
        let mut bytes = [0; 4];
        let result = unsafe { test_event_read_bits(event, bytes.as_mut_ptr(), 32) };
        if result == RakRsResult::Ok {
            unsafe { output.write(u32::from_le_bytes(bytes)) };
        }
        result
    }

    unsafe extern "system" fn test_event_read_f32(
        event: *mut RakRsEventV1,
        output: *mut f32,
    ) -> RakRsResult {
        let mut bits = 0;
        let result = unsafe { test_event_read_u32(event, &raw mut bits) };
        if result == RakRsResult::Ok {
            unsafe { output.write(f32::from_bits(bits)) };
        }
        result
    }

    unsafe extern "system" fn test_event_read_bytes(
        event: *mut RakRsEventV1,
        output: *mut u8,
        byte_len: usize,
    ) -> RakRsResult {
        unsafe { test_event_read_bits(event, output, byte_len * 8) }
    }

    unsafe extern "system" fn test_event_write_u8(
        _event: *mut RakRsEventV1,
        _value: u8,
    ) -> RakRsResult {
        RakRsResult::NativeCallFailed
    }

    unsafe extern "system" fn test_event_write_u16(
        _event: *mut RakRsEventV1,
        _value: u16,
    ) -> RakRsResult {
        RakRsResult::NativeCallFailed
    }

    unsafe extern "system" fn test_event_write_u32(
        _event: *mut RakRsEventV1,
        _value: u32,
    ) -> RakRsResult {
        RakRsResult::NativeCallFailed
    }

    unsafe extern "system" fn test_event_write_f32(
        _event: *mut RakRsEventV1,
        _value: f32,
    ) -> RakRsResult {
        RakRsResult::NativeCallFailed
    }

    unsafe extern "system" fn test_event_write_bytes(
        _event: *mut RakRsEventV1,
        _value: *const u8,
        _byte_len: usize,
    ) -> RakRsResult {
        RakRsResult::NativeCallFailed
    }

    unsafe extern "system" fn test_event_replace_bytes(
        event: *mut RakRsEventV1,
        bytes: *const u8,
        byte_len: usize,
    ) -> RakRsResult {
        unsafe { test_event_replace_bits(event, bytes, byte_len, byte_len * 8) }
    }

    unsafe extern "system" fn test_event_replace_bits(
        event: *mut RakRsEventV1,
        bytes: *const u8,
        byte_len: usize,
        bit_len: usize,
    ) -> RakRsResult {
        if bit_len > byte_len.saturating_mul(8) {
            return RakRsResult::InvalidArgument;
        }
        let event = unsafe { test_event(event) };
        event.bytes = if byte_len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(bytes, byte_len) }.to_vec()
        };
        event.bit_len = bit_len;
        event.read_offset = 0;
        RakRsResult::Ok
    }

    unsafe extern "system" fn test_event_remaining_bits(event: *mut RakRsEventV1) -> usize {
        let event = unsafe { test_event(event) };
        event.bit_len - event.read_offset
    }

    unsafe extern "system" fn test_encoded_string(
        value: *const u8,
        value_len: usize,
        output: *mut u8,
        output_capacity: usize,
        bit_len: *mut usize,
    ) -> RakRsResult {
        if (value.is_null() && value_len != 0) || output.is_null() || bit_len.is_null() {
            return RakRsResult::InvalidArgument;
        }
        if value_len > u16::MAX as usize {
            return RakRsResult::PayloadTooLarge;
        }
        let value = if value_len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(value, value_len) }
        };
        let mut writer = PayloadWriter::new();
        writer.u16(value_len as u16);
        writer.bytes(value);
        let encoded = writer.finish_bits();
        if encoded.bytes.len() > output_capacity {
            return RakRsResult::PayloadTooLarge;
        }
        unsafe {
            ptr::copy_nonoverlapping(encoded.bytes.as_ptr(), output, encoded.bytes.len());
            bit_len.write(encoded.bit_len);
        }
        RakRsResult::Ok
    }

    unsafe extern "system" fn test_read_encoded_string(
        event: *mut RakRsEventV1,
        output: *mut u8,
        output_capacity: usize,
        output_len: *mut usize,
    ) -> RakRsResult {
        if output.is_null() || output_len.is_null() {
            return RakRsResult::InvalidArgument;
        }
        let mut length = 0;
        let result = unsafe { test_event_read_u16(event, &raw mut length) };
        if result != RakRsResult::Ok {
            return result;
        }
        let length = usize::from(length);
        if length > output_capacity {
            return RakRsResult::PayloadTooLarge;
        }
        let result = unsafe { test_event_read_bytes(event, output, length) };
        if result == RakRsResult::Ok {
            unsafe { output_len.write(length) };
        }
        result
    }

    extern "system" fn test_status() -> crate::RakRsHostStatus {
        crate::RakRsHostStatus::Ready
    }

    unsafe extern "system" fn test_register(
        _direction: crate::RakRsDirection,
        _callback: Option<crate::RakRsEventCallbackV1>,
        _user_data: *mut core::ffi::c_void,
        _subscription: *mut crate::RakRsSubscription,
    ) -> RakRsResult {
        RakRsResult::NativeCallFailed
    }

    unsafe extern "system" fn test_unregister(
        _subscription: crate::RakRsSubscription,
    ) -> RakRsResult {
        RakRsResult::NativeCallFailed
    }

    unsafe extern "system" fn test_send(
        _id: u8,
        _bytes: *const u8,
        _byte_len: usize,
        _bit_len: usize,
        _options: crate::RakRsSendOptions,
    ) -> RakRsResult {
        RakRsResult::NativeCallFailed
    }

    unsafe extern "system" fn test_emulate(
        _id: u8,
        _bytes: *const u8,
        _byte_len: usize,
        _bit_len: usize,
    ) -> RakRsResult {
        RakRsResult::NativeCallFailed
    }

    fn test_api() -> HostApi {
        let api = Box::leak(Box::new(crate::RakRsApiV1 {
            abi_version: crate::ABI_VERSION_V1,
            size: mem::size_of::<crate::RakRsApiV1>() as u32,
            host_status: test_status,
            register_packet: test_register,
            register_rpc: test_register,
            unregister: test_unregister,
            event_id: test_event_id,
            event_reset_read: test_event_reset_read,
            event_clear: test_event_clear,
            event_read_u8: test_event_read_u8,
            event_read_u16: test_event_read_u16,
            event_read_u32: test_event_read_u32,
            event_read_f32: test_event_read_f32,
            event_read_bytes: test_event_read_bytes,
            event_write_u8: test_event_write_u8,
            event_write_u16: test_event_write_u16,
            event_write_u32: test_event_write_u32,
            event_write_f32: test_event_write_f32,
            event_write_bytes: test_event_write_bytes,
            send_packet: test_send,
            send_rpc: test_send,
            event_replace_bytes: test_event_replace_bytes,
            unregister_and_wait: test_unregister,
            emulate_incoming_packet: test_emulate,
            emulate_incoming_rpc: test_emulate,
            event_remaining_bits: test_event_remaining_bits,
            event_read_bits: test_event_read_bits,
            event_replace_bits: test_event_replace_bits,
            encode_string: test_encoded_string,
            event_read_encoded_string: test_read_encoded_string,
        }));
        unsafe { HostApi::from_raw(api) }.expect("test API is complete")
    }

    fn assert_replacement_round_trip<T>(descriptor: Rpc<T>, value: T)
    where
        T: Clone + core::fmt::Debug + PartialEq,
    {
        let api = test_api();
        let id = descriptor.clone().id();
        let encoded = descriptor
            .clone()
            .encode(api, value.clone())
            .expect("test payload must encode");
        let mut raw = TestEvent::new(id, encoded.clone());
        let mut event = unsafe {
            Event::from_callback(api, (&mut raw as *mut TestEvent).cast::<RakRsEventV1>())
        }
        .expect("test event is not null");
        assert_eq!(
            descriptor
                .handle(&mut event, |decoded| {
                    assert_eq!(decoded, value);
                    RpcAction::Replace(decoded)
                })
                .expect("typed replacement must succeed"),
            RakRsHookAction::Continue
        );
        assert_eq!(raw.bit_len, encoded.bit_len);
        assert_eq!(raw.bytes, encoded.bytes);
    }

    fn encode_bytes<T>(descriptor: Rpc<T>, value: T) -> Vec<u8> {
        let RpcEncoder::Bytes(encode) = descriptor.encode else {
            panic!("test descriptor must use a byte-aligned encoder");
        };
        encode(value).expect("test payload must be valid")
    }

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

    fn test_vector3(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }

    fn test_spawn_info() -> incoming::SpawnInfo {
        incoming::SpawnInfo {
            team: 7,
            skin: 411,
            unused: 0xA5,
            position: test_vector3(1.0, 2.0, 3.0),
            rotation: 4.0,
            weapons: [22, 24, 31],
            ammo: [100, 200, 300],
        }
    }

    fn test_animation() -> incoming::Animation {
        incoming::Animation {
            animation_library: b"PED".to_vec(),
            animation_name: b"WALK".to_vec(),
            frame_delta: 4.0,
            looped: true,
            lock_x: false,
            lock_y: true,
            freeze: false,
            time: -1,
        }
    }

    #[test]
    fn r1_player_stream_in_includes_all_eleven_weapon_skill_levels() {
        let value = incoming::PlayerStreamIn {
            player_id: 42,
            team: 3,
            model: 411,
            position: test_vector3(1.0, 2.0, 3.0),
            rotation: 90.0,
            color: -1,
            fighting_style: 4,
            weapon_skill_levels: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        };
        let encoded = incoming::PLAYER_STREAM_IN
            .encode(test_api(), value)
            .expect("R1 player stream-in payload must encode");

        assert_eq!(encoded.len_bits(), 400);
        assert_eq!(
            encoded.as_bytes(),
            &[
                0x2A, 0x00, 0x03, 0x9B, 0x01, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00,
                0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0xB4, 0x42, 0xFF, 0xFF, 0xFF, 0xFF, 0x04,
                0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06, 0x00,
                0x07, 0x00, 0x08, 0x00, 0x09, 0x00, 0x0A, 0x00,
            ]
        );
        assert_replacement_round_trip(incoming::PLAYER_STREAM_IN, value);
    }

    #[test]
    fn r1_complex_incoming_rpc_helpers_decode_and_atomically_replace() {
        let settings = incoming::GameSettings {
            zone_names: true,
            use_cj_walk: false,
            allow_weapons: true,
            limit_global_chat_radius: false,
            global_chat_radius: 100.0,
            stunt_bonus: true,
            nametag_draw_distance: 70.0,
            disable_enter_exits: false,
            nametag_los: true,
            tire_popping: false,
            classes_available: 5,
            show_player_tags: true,
            player_markers_mode: 1,
            world_time: 12,
            world_weather: 7,
            gravity: 0.008,
            lan_mode: false,
            death_money_drop: 500,
            instagib: false,
            normal_onfoot_send_rate: 30,
            normal_incar_send_rate: 30,
            normal_firing_send_rate: 30,
            send_multiplier: 2,
            lag_compensation_mode: 1,
            vehicle_friendly_fire: true,
        };
        assert_replacement_round_trip(
            incoming::INIT_GAME,
            incoming::InitGame {
                player_id: 42,
                host_name: b"R1 host".to_vec(),
                settings,
                vehicle_models: [1; 212],
            },
        );
        assert_replacement_round_trip(
            incoming::REQUEST_CLASS_RESPONSE,
            incoming::RequestClassResponse {
                can_spawn: true,
                spawn: test_spawn_info(),
            },
        );
        assert_replacement_round_trip(
            incoming::PLAYER_STREAM_IN,
            incoming::PlayerStreamIn {
                player_id: 42,
                team: 3,
                model: 411,
                position: test_vector3(1.0, 2.0, 3.0),
                rotation: 90.0,
                color: -1,
                fighting_style: 4,
                weapon_skill_levels: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            },
        );
        assert_replacement_round_trip(
            incoming::CREATE_3D_TEXT,
            incoming::TextLabel3D {
                id: 4,
                color: -1,
                position: test_vector3(1.0, 2.0, 3.0),
                distance: 50.0,
                test_los: true,
                attached_player_id: u16::MAX,
                attached_vehicle_id: u16::MAX,
                text: b"encoded 3D text".to_vec(),
            },
        );
        assert_replacement_round_trip(
            incoming::CREATE_OBJECT,
            incoming::Object {
                object_id: 9,
                model_id: 1337,
                position: test_vector3(1.0, 2.0, 3.0),
                rotation: test_vector3(4.0, 5.0, 6.0),
                draw_distance: 300.0,
                no_camera_collision: true,
                attach_to_vehicle_id: u16::MAX,
                attach_to_object_id: u16::MAX,
                attachment: None,
                textures_count: 2,
                materials: vec![
                    incoming::ObjectMaterial::Texture(incoming::TextureMaterial {
                        material_id: 0,
                        model_id: 18646,
                        library_name: b"matcolours".to_vec(),
                        texture_name: b"grey-10-percent".to_vec(),
                        color: -1,
                    }),
                    incoming::ObjectMaterial::Text(incoming::TextMaterial {
                        material_id: 1,
                        material_size: 90,
                        font_name: b"Arial".to_vec(),
                        font_size: 20,
                        bold: 1,
                        font_color: -1,
                        background_color: 0,
                        align: 2,
                        text: b"material text".to_vec(),
                    }),
                ],
            },
        );
        assert_replacement_round_trip(incoming::SET_SPAWN_INFO, test_spawn_info());
        assert_replacement_round_trip(
            incoming::INIT_MENU,
            incoming::InitMenu {
                menu_id: 1,
                two_columns: true,
                title: *b"R1 menu                         ",
                position: Vector2 { x: 10.0, y: 20.0 },
                columns: vec![
                    incoming::MenuColumn {
                        width: 100.0,
                        title: *b"first                           ",
                        rows: vec![*b"one                             "],
                    },
                    incoming::MenuColumn {
                        width: 200.0,
                        title: *b"second                          ",
                        rows: vec![*b"two                             "],
                    },
                ],
                rows: [-1; incoming::MAX_MENU_ROWS],
                menu: false,
            },
        );
        assert_replacement_round_trip(
            incoming::INTERPOLATE_CAMERA,
            incoming::InterpolateCamera {
                set_position: true,
                from_position: test_vector3(1.0, 2.0, 3.0),
                destination: test_vector3(4.0, 5.0, 6.0),
                time_ms: 500,
                mode: 2,
            },
        );
        assert_replacement_round_trip(
            incoming::TOGGLE_SELECT_TEXT_DRAW,
            incoming::ToggleSelectTextDraw {
                enabled: true,
                hover_color: -1,
            },
        );
        assert_replacement_round_trip(
            incoming::SET_OBJECT_MATERIAL,
            incoming::ObjectMaterialUpdate {
                object_id: 9,
                material: incoming::ObjectMaterial::Texture(incoming::TextureMaterial {
                    material_id: 1,
                    model_id: 123,
                    library_name: b"lib".to_vec(),
                    texture_name: b"texture".to_vec(),
                    color: 0x1122_3344,
                }),
            },
        );
        assert_replacement_round_trip(
            incoming::SET_OBJECT_MATERIAL,
            incoming::ObjectMaterialUpdate {
                object_id: 9,
                material: incoming::ObjectMaterial::Text(incoming::TextMaterial {
                    material_id: 2,
                    material_size: 90,
                    font_name: b"Arial".to_vec(),
                    font_size: 20,
                    bold: 0,
                    font_color: -1,
                    background_color: 0,
                    align: 1,
                    text: b"encoded material update".to_vec(),
                }),
            },
        );
        assert_replacement_round_trip(
            incoming::APPLY_PLAYER_ANIMATION,
            incoming::PlayerAnimation {
                player_id: 7,
                animation: test_animation(),
            },
        );
        assert_replacement_round_trip(incoming::ENABLE_STUNT_BONUS, true);
        assert_replacement_round_trip(
            incoming::PLAY_CRIME_REPORT,
            incoming::CrimeReport {
                suspect_id: 7,
                in_vehicle: true,
                vehicle_model: 411,
                vehicle_color: 4,
                crime: 9,
                coordinates: test_vector3(1.0, 2.0, 3.0),
            },
        );
        assert_replacement_round_trip(
            incoming::SET_PLAYER_ATTACHED_OBJECT,
            incoming::PlayerAttachedObject {
                player_id: 7,
                index: 3,
                object: Some(incoming::AttachedObject {
                    model_id: 19327,
                    bone: 1,
                    offset: test_vector3(1.0, 2.0, 3.0),
                    rotation: test_vector3(4.0, 5.0, 6.0),
                    scale: test_vector3(1.0, 1.0, 1.0),
                    color1: -1,
                    color2: 0,
                }),
            },
        );
        assert_replacement_round_trip(
            incoming::ENTER_EDIT_OBJECT,
            incoming::EnterEditObject {
                player_object: true,
                object_id: 5,
            },
        );
        assert_replacement_round_trip(incoming::TOGGLE_PLAYER_SPECTATING, false);
        assert_replacement_round_trip(
            incoming::SHOW_TEXT_DRAW,
            incoming::ShowTextDraw {
                textdraw_id: 99,
                textdraw: incoming::TextDraw {
                    flags: 1,
                    letter_width: 0.5,
                    letter_height: 1.0,
                    letter_color: -1,
                    line_width: 2.0,
                    line_height: 3.0,
                    box_color: 0,
                    shadow: 1,
                    outline: 2,
                    background_color: 0,
                    style: 4,
                    selectable: 1,
                    position: Vector2 { x: 100.0, y: 200.0 },
                    model_id: 1234,
                    rotation: test_vector3(0.0, 0.0, 1.0),
                    zoom: 1.5,
                    color1: -1,
                    color2: 2,
                    text: b"textdraw".to_vec(),
                },
            },
        );
        assert_replacement_round_trip(incoming::TEXT_DRAW_HIDE, 99);
        assert_replacement_round_trip(
            incoming::UPDATE_SCORES_AND_PINGS,
            incoming::ScoresAndPings {
                entries: vec![incoming::ScorePing {
                    player_id: 7,
                    score: -100,
                    ping: 42,
                }],
            },
        );
        assert_replacement_round_trip(
            incoming::VEHICLE_STREAM_IN,
            incoming::VehicleStreamIn {
                vehicle_id: 9,
                vehicle: incoming::StreamedVehicle {
                    model: 411,
                    position: test_vector3(1.0, 2.0, 3.0),
                    rotation: 45.0,
                    body_color1: 1,
                    body_color2: 2,
                    health: 900.0,
                    interior_id: 3,
                    door_damage_status: 4,
                    panel_damage_status: 5,
                    light_damage_status: 6,
                    tire_damage_status: 7,
                    add_siren: 8,
                    mod_slots: [9; 14],
                    paint_job: 10,
                    interior_color1: 11,
                    interior_color2: 12,
                },
            },
        );
        assert_replacement_round_trip(incoming::DISABLE_VEHICLE_COLLISIONS, true);
        assert_replacement_round_trip(incoming::TOGGLE_CAMERA_TARGET_NOTIFYING, false);
        assert_replacement_round_trip(
            incoming::APPLY_ACTOR_ANIMATION,
            incoming::ActorAnimation {
                actor_id: 8,
                animation: test_animation(),
            },
        );
    }

    #[test]
    fn r1_remote_sync_and_markers_decode_and_atomically_replace() {
        assert_replacement_round_trip(
            packet::incoming::PLAYER_SYNC,
            packet::RemotePlayerSync {
                player_id: 1,
                left_right_keys: Some(2),
                up_down_keys: None,
                key_data: 3,
                position: test_vector3(1.0, 2.0, 3.0),
                quaternion: [-1.0, 0.0, 0.0, 0.0],
                health: 100,
                armour: 98,
                weapon: 24,
                special_action: 0,
                move_speed: test_vector3(0.0, 0.0, 0.0),
                surfing: Some(packet::RemotePlayerSurfing {
                    vehicle_id: 4,
                    offsets: test_vector3(4.0, 5.0, 6.0),
                }),
                animation: Some(packet::RemotePlayerAnimation { id: 7, flags: 8 }),
            },
        );
        assert_replacement_round_trip(
            packet::incoming::VEHICLE_SYNC,
            packet::RemoteVehicleSync {
                player_id: 1,
                vehicle_id: 2,
                left_right_keys: 3,
                up_down_keys: 4,
                key_data: 5,
                quaternion: [1.0, 0.0, 0.0, 0.0],
                position: test_vector3(1.0, 2.0, 3.0),
                // R1's compressed-vector zero components decode to -1 / 65536 after the
                // writer's integer conversion; use the exact representable values here.
                move_speed: test_vector3(1.0, -1.0 / 65_536.0, -1.0 / 65_536.0),
                vehicle_health: 900,
                player_health: 98,
                armour: 0,
                current_weapon: 24,
                siren: true,
                landing_gear: false,
                train_speed: Some(-7),
                trailer_id: Some(6),
            },
        );
        assert_replacement_round_trip(
            packet::incoming::MARKERS_SYNC,
            packet::MarkersSync {
                markers: vec![
                    packet::Marker {
                        player_id: 1,
                        coordinates: None,
                    },
                    packet::Marker {
                        player_id: 2,
                        coordinates: Some(packet::MarkerCoordinates { x: -1, y: -2, z: 3 }),
                    },
                ],
            },
        );
    }

    #[test]
    fn typed_helpers_reject_trailing_bits_before_invoking_the_callback() {
        let api = test_api();
        let mut raw = TestEvent::new(
            incoming::ENABLE_STUNT_BONUS.id(),
            EncodedPayload::from_bits(vec![0b1000_0000], 2).unwrap(),
        );
        let mut event = unsafe {
            Event::from_callback(api, (&mut raw as *mut TestEvent).cast::<RakRsEventV1>())
        }
        .unwrap();
        assert!(matches!(
            incoming::ENABLE_STUNT_BONUS.handle(&mut event, |_| panic!("must not dispatch")),
            Err(EventError::UnexpectedBitLength {
                bit_len: 1,
                expected: 0
            })
        ));
    }

    #[test]
    fn marker_sync_keeps_negative_r1_coordinates_as_signed_i16_values() {
        let payload = packet::incoming::MARKERS_SYNC
            .encode(
                test_api(),
                packet::MarkersSync {
                    markers: vec![
                        packet::Marker {
                            player_id: 1,
                            coordinates: None,
                        },
                        packet::Marker {
                            player_id: 2,
                            coordinates: Some(packet::MarkerCoordinates { x: -1, y: -2, z: 3 }),
                        },
                    ],
                },
            )
            .unwrap();
        assert_eq!(payload.len_bits(), 114);
        assert_eq!(
            payload.as_bytes(),
            &[
                2, 0, 0, 0, 1, 0, 1, 0, 0x7F, 0xFF, 0xFF, 0xBF, 0xC0, 0xC0, 0
            ]
        );
    }

    #[test]
    fn marker_sync_accepts_terminal_byte_alignment_padding() {
        let api = test_api();
        let value = packet::MarkersSync {
            markers: vec![packet::Marker {
                player_id: 1,
                coordinates: None,
            }],
        };
        let canonical = packet::incoming::MARKERS_SYNC
            .encode(api, value.clone())
            .expect("marker payload must encode");
        assert_eq!(canonical.len_bits(), 49);

        let mut bytes = canonical.as_bytes().to_vec();
        // The packet transport can leave its terminal byte's unused bits unspecified.
        *bytes.last_mut().expect("marker payload has a final byte") |= 0x40;
        let padded = EncodedPayload::from_bits(bytes, 56)
            .expect("the rounded marker payload remains in its buffer");
        let mut raw = TestEvent::new(packet::incoming::MARKERS_SYNC.id(), padded);
        let mut event = unsafe {
            Event::from_callback(api, (&mut raw as *mut TestEvent).cast::<RakRsEventV1>())
        }
        .expect("test event is not null");
        assert_eq!(
            packet::incoming::MARKERS_SYNC
                .handle(&mut event, |decoded| {
                    assert_eq!(decoded, value);
                    RpcAction::Replace(decoded)
                })
                .expect("terminal alignment padding must be accepted"),
            RakRsHookAction::Continue
        );
        assert_eq!(raw.bit_len, canonical.len_bits());
        assert_eq!(raw.bytes, canonical.as_bytes());

        let mut bytes = canonical.as_bytes().to_vec();
        bytes.push(0);
        let mut raw = TestEvent::new(
            packet::incoming::MARKERS_SYNC.id(),
            EncodedPayload::from_bits(bytes, 57).expect("the malformed suffix fits"),
        );
        let mut event = unsafe {
            Event::from_callback(api, (&mut raw as *mut TestEvent).cast::<RakRsEventV1>())
        }
        .expect("test event is not null");
        assert!(matches!(
            packet::incoming::MARKERS_SYNC.handle(&mut event, |_| panic!(
                "a full trailing byte must not dispatch"
            )),
            Err(EventError::UnexpectedBitLength {
                bit_len: 8,
                expected: 0
            })
        ));
    }

    #[test]
    fn set_player_skin_uses_rpc_153_and_two_i32_values() {
        assert_eq!(incoming::SET_PLAYER_SKIN.id(), 153);
        let RpcEncoder::Bytes(encode) = incoming::SET_PLAYER_SKIN.encode else {
            panic!("SetPlayerSkin must use a byte-aligned encoder");
        };
        let bytes = encode(incoming::PlayerSkin {
            player_id: 0,
            skin_id: 411,
        })
        .expect("valid i32 skin payload");

        assert_eq!(bytes, [0, 0, 0, 0, 0x9B, 0x01, 0, 0]);
        assert_eq!(EncodedPayload::from_bytes(bytes).unwrap().len_bits(), 64);
    }

    #[test]
    fn fixed_layout_incoming_rpc_helpers_use_their_protocol_ids() {
        let descriptors = [
            (incoming::CANCEL_EDIT.id(), 28),
            (incoming::SET_TOGGLE_CLOCK.id(), 30),
            (incoming::SET_PLAYER_DRUNK.id(), 35),
            (incoming::SET_RACE_CHECKPOINT.id(), 38),
            (incoming::PLAY_AUDIO_STREAM.id(), 41),
            (incoming::SET_OBJECT_POSITION.id(), 45),
            (incoming::SET_OBJECT_ROTATION.id(), 46),
            (incoming::DESTROY_OBJECT.id(), 47),
            (incoming::PLAYER_DEATH_NOTIFICATION.id(), 55),
            (incoming::SET_MAP_ICON.id(), 56),
            (incoming::REMOVE_VEHICLE_COMPONENT.id(), 57),
            (incoming::REMOVE_3D_TEXT_LABEL.id(), 58),
            (incoming::UPDATE_GLOBAL_TIMER.id(), 60),
            (incoming::DESTROY_PICKUP.id(), 63),
            (incoming::LINK_VEHICLE_TO_INTERIOR.id(), 65),
            (incoming::SET_PLAYER_COLOR.id(), 72),
        ];

        for (actual, expected) in descriptors {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn r1_complex_incoming_rpc_helpers_use_their_protocol_ids() {
        let descriptors = [
            (incoming::INIT_GAME.id(), 139),
            (incoming::REQUEST_CLASS_RESPONSE.id(), 128),
            (incoming::PLAYER_STREAM_IN.id(), 32),
            (incoming::CREATE_3D_TEXT.id(), 36),
            (incoming::CREATE_OBJECT.id(), 44),
            (incoming::SET_SPAWN_INFO.id(), 68),
            (incoming::INIT_MENU.id(), 76),
            (incoming::INTERPOLATE_CAMERA.id(), 82),
            (incoming::TOGGLE_SELECT_TEXT_DRAW.id(), 83),
            (incoming::SET_OBJECT_MATERIAL.id(), 84),
            (incoming::APPLY_PLAYER_ANIMATION.id(), 86),
            (incoming::ENABLE_STUNT_BONUS.id(), 104),
            (incoming::PLAY_CRIME_REPORT.id(), 112),
            (incoming::SET_PLAYER_ATTACHED_OBJECT.id(), 113),
            (incoming::ENTER_EDIT_OBJECT.id(), 117),
            (incoming::TOGGLE_PLAYER_SPECTATING.id(), 124),
            (incoming::SHOW_TEXT_DRAW.id(), 134),
            (incoming::TEXT_DRAW_HIDE.id(), 135),
            (incoming::UPDATE_SCORES_AND_PINGS.id(), 155),
            (incoming::VEHICLE_STREAM_IN.id(), 164),
            (incoming::DISABLE_VEHICLE_COLLISIONS.id(), 167),
            (incoming::TOGGLE_CAMERA_TARGET_NOTIFYING.id(), 170),
            (incoming::APPLY_ACTOR_ANIMATION.id(), 173),
        ];
        for (actual, expected) in descriptors {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn fixed_layout_incoming_rpc_helpers_encode_exact_vectors() {
        let race_checkpoint = encode_bytes(
            incoming::SET_RACE_CHECKPOINT,
            incoming::RaceCheckpoint {
                checkpoint_type: 2,
                position: Vector3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                next_position: Vector3 {
                    x: 4.0,
                    y: 5.0,
                    z: 6.0,
                },
                size: 7.0,
            },
        );
        assert_eq!(
            race_checkpoint,
            [
                2, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80, 0x40, 0, 0, 0xA0,
                0x40, 0, 0, 0xC0, 0x40, 0, 0, 0xE0, 0x40,
            ]
        );

        let audio_stream = encode_bytes(
            incoming::PLAY_AUDIO_STREAM,
            incoming::AudioStream {
                url: b"x.y".to_vec(),
                position: Vector3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                radius: 4.0,
                use_position: true,
            },
        );
        assert_eq!(
            audio_stream,
            [
                3, b'x', b'.', b'y', 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80,
                0x40, 1,
            ]
        );

        assert_eq!(
            encode_bytes(
                incoming::SET_MAP_ICON,
                incoming::MapIcon {
                    icon_id: 7,
                    position: Vector3 {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0,
                    },
                    icon_type: 4,
                    color: -1,
                    style: 2,
                },
            ),
            [
                7, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 4, 0xFF, 0xFF, 0xFF, 0xFF, 2,
            ]
        );
        assert_eq!(
            encode_bytes(
                incoming::PLAYER_DEATH_NOTIFICATION,
                incoming::PlayerDeathNotification {
                    killer_id: 0x1234,
                    killed_id: 0x5678,
                    reason: 9,
                },
            ),
            [0x34, 0x12, 0x78, 0x56, 9]
        );
        assert_eq!(
            encode_bytes(
                incoming::SET_PLAYER_COLOR,
                incoming::PlayerColor {
                    player_id: 0x1234,
                    color: -1,
                },
            ),
            [0x34, 0x12, 0xFF, 0xFF, 0xFF, 0xFF]
        );
    }

    #[test]
    fn remaining_outgoing_rpc_helpers_use_their_protocol_ids() {
        let descriptors = [
            (outgoing::SEND_CLIENT_JOIN.id(), 25),
            (outgoing::SEND_ENTER_EDIT_OBJECT.id(), 27),
            (outgoing::SEND_MONEY_INCREASE.id(), 31),
            (outgoing::SEND_NPC_JOIN.id(), 54),
            (outgoing::SEND_VEHICLE_TUNING.id(), 96),
            (outgoing::SEND_PICKED_UP_WEAPON.id(), 97),
            (outgoing::SEND_SERVER_STATISTICS_REQUEST.id(), 102),
            (outgoing::SEND_CLIENT_CHECK_RESPONSE.id(), 103),
            (outgoing::SEND_VEHICLE_DAMAGED.id(), 106),
            (outgoing::SEND_DAMAGE.id(), 115),
            (outgoing::SEND_EDIT_ATTACHED_OBJECT.id(), 116),
            (outgoing::SEND_EDIT_OBJECT.id(), 117),
            (outgoing::SEND_PICKED_UP_PICKUP.id(), 131),
            (outgoing::SEND_QUIT_MENU.id(), 140),
            (outgoing::SEND_CAMERA_TARGET_UPDATE.id(), 168),
            (outgoing::SEND_GIVE_ACTOR_DAMAGE.id(), 177),
        ];

        for (actual, expected) in descriptors {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn further_fixed_layout_incoming_rpc_helpers_encode_exact_vectors() {
        assert_eq!(incoming::SET_SHOP_NAME.id(), 33);
        assert_eq!(incoming::CREATE_GANG_ZONE.id(), 108);
        assert_eq!(incoming::SET_VEHICLE_PARAMS_EX.id(), 24);
        assert_eq!(incoming::CREATE_ACTOR.id(), 171);

        assert_eq!(
            encode_bytes(
                incoming::CREATE_GANG_ZONE,
                incoming::GangZone {
                    zone_id: 0x1234,
                    square_start: Vector2 { x: 1.0, y: 2.0 },
                    square_end: Vector2 { x: 3.0, y: 4.0 },
                    color: -1,
                },
            ),
            [
                0x34, 0x12, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80, 0x40,
                0xFF, 0xFF, 0xFF, 0xFF,
            ]
        );
        assert_eq!(
            encode_bytes(
                incoming::SET_VEHICLE_PARAMS_EX,
                incoming::VehicleParamsEx {
                    vehicle_id: 1,
                    params: [2; 8],
                    doors: [3; 4],
                    windows: [4; 4],
                },
            ),
            [1, 0, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4]
        );
        assert_eq!(
            encode_bytes(
                incoming::CREATE_ACTOR,
                incoming::Actor {
                    actor_id: 7,
                    skin_id: 411,
                    position: Vector3 {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0,
                    },
                    rotation: 4.0,
                    health: 5.0,
                },
            ),
            [
                7, 0, 0x9B, 1, 0, 0, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40, 0, 0, 0x80,
                0x40, 0, 0, 0xA0, 0x40,
            ]
        );
    }

    #[test]
    fn outgoing_damage_keeps_its_one_bit_boolean_and_exact_payload_length() {
        let payload = outgoing::encode_damage_payload(outgoing::Damage {
            player_id: 0x1234,
            damage: 1.0,
            weapon: 24,
            body_part: 9,
            take: true,
        })
        .expect("damage payload must encode");

        assert_eq!(payload.len_bits(), 113);
        assert_eq!(
            payload.as_bytes(),
            [
                0x9A, 0x09, 0x00, 0x00, 0x40, 0x1F, 0x8C, 0x00, 0x00, 0x00, 0x04, 0x80, 0x00, 0x00,
                0x00,
            ]
        );
    }

    #[test]
    fn packet_helpers_filter_the_documented_packet_ids() {
        assert_eq!(packet::outgoing::SEND_AUTHENTICATION_RESPONSE.id(), 12);
        assert_eq!(packet::outgoing::SEND_WEAPONS_UPDATE.id(), 204);
        assert_eq!(packet::outgoing::SEND_RCON_COMMAND.id(), 201);
        assert_eq!(packet::outgoing::SEND_STATS_UPDATE.id(), 205);
        assert_eq!(packet::outgoing::SEND_PLAYER_SYNC.id(), 207);
        assert_eq!(packet::outgoing::SEND_VEHICLE_SYNC.id(), 200);
        assert_eq!(packet::outgoing::SEND_PASSENGER_SYNC.id(), 211);
        assert_eq!(packet::outgoing::SEND_AIM_SYNC.id(), 203);
        assert_eq!(packet::outgoing::SEND_UNOCCUPIED_SYNC.id(), 209);
        assert_eq!(packet::outgoing::SEND_TRAILER_SYNC.id(), 210);
        assert_eq!(packet::outgoing::SEND_BULLET_SYNC.id(), 206);
        assert_eq!(packet::outgoing::SEND_SPECTATOR_SYNC.id(), 212);
        assert_eq!(packet::incoming::AIM_SYNC.id(), 203);
        assert_eq!(packet::incoming::VEHICLE_SYNC.id(), 200);
        assert_eq!(packet::incoming::BULLET_SYNC.id(), 206);
        assert_eq!(packet::incoming::PLAYER_SYNC.id(), 207);
        assert_eq!(packet::incoming::MARKERS_SYNC.id(), 208);
        assert_eq!(packet::incoming::UNOCCUPIED_SYNC.id(), 209);
        assert_eq!(packet::incoming::TRAILER_SYNC.id(), 210);
        assert_eq!(packet::incoming::PASSENGER_SYNC.id(), 211);
        assert_eq!(packet::incoming::AUTHENTICATION_REQUEST.id(), 12);
        assert_eq!(packet::incoming::CONNECTION_ACCEPTED.id(), 34);
        assert_eq!(packet::incoming::CONNECTION_LOST.id(), 33);
        assert_eq!(packet::incoming::CONNECTION_BANNED.id(), 36);
        assert_eq!(packet::incoming::CONNECTION_ATTEMPT_FAILED.id(), 29);
        assert_eq!(packet::incoming::CONNECTION_NO_FREE_SLOT.id(), 31);
        assert_eq!(packet::incoming::CONNECTION_PASSWORD_INVALID.id(), 37);
        assert_eq!(packet::incoming::CONNECTION_CLOSED.id(), 32);
    }

    #[test]
    fn packet_helpers_encode_exact_fixed_layout_vectors() {
        assert_eq!(
            encode_bytes(
                packet::outgoing::SEND_STATS_UPDATE,
                packet::StatsUpdate {
                    money: -1,
                    drunk_level: 42,
                },
            ),
            [0xFF, 0xFF, 0xFF, 0xFF, 42, 0, 0, 0]
        );
        assert_eq!(
            encode_bytes(
                packet::outgoing::SEND_WEAPONS_UPDATE,
                packet::WeaponsUpdate {
                    player_target: 1,
                    actor_target: 2,
                    weapons: vec![packet::WeaponSlot {
                        slot: 3,
                        weapon: 24,
                        ammo: 50,
                    }],
                },
            ),
            [1, 0, 2, 0, 3, 24, 50, 0]
        );

        let aim = packet::AimSync {
            camera_mode: 7,
            camera_front: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            camera_position: Vector3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
            aim_z: 7.0,
            zoom_and_weapon_state: 0b1010_0101,
            aspect_ratio: 9,
        };
        assert_eq!(
            encode_bytes(packet::outgoing::SEND_AIM_SYNC, aim),
            [
                7,
                0,
                0,
                0x80,
                0x3F,
                0,
                0,
                0,
                0x40,
                0,
                0,
                0x40,
                0x40,
                0,
                0,
                0x80,
                0x40,
                0,
                0,
                0xA0,
                0x40,
                0,
                0,
                0xC0,
                0x40,
                0,
                0,
                0xE0,
                0x40,
                0b1010_0101,
                9,
            ]
        );

        let bullet = packet::BulletSync {
            target_type: 1,
            target_id: 0x1234,
            origin: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            target: Vector3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
            center: Vector3 {
                x: 7.0,
                y: 8.0,
                z: 9.0,
            },
            weapon_id: 24,
        };
        let bytes = encode_bytes(packet::outgoing::SEND_BULLET_SYNC, bullet);
        assert_eq!(bytes.len(), 40);
        assert_eq!(
            &bytes[..15],
            &[
                1, 0x34, 0x12, 0, 0, 0x80, 0x3F, 0, 0, 0, 0x40, 0, 0, 0x40, 0x40
            ]
        );
        assert_eq!(
            &bytes[27..],
            &[0, 0, 0xE0, 0x40, 0, 0, 0, 0x41, 0, 0, 0x10, 0x41, 24]
        );

        let player = packet::PlayerSync {
            left_right_keys: 1,
            up_down_keys: 2,
            key_data: 3,
            position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            quaternion: [0.0; 4],
            health: 4,
            armour: 5,
            weapon_and_special_key: 6,
            special_action: 7,
            move_speed: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            surfing_offsets: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            surfing_vehicle_id: 8,
            animation_id: 9,
            animation_flags: 10,
        };
        let bytes = encode_bytes(packet::outgoing::SEND_PLAYER_SYNC, player);
        assert_eq!(bytes.len(), 68);
        assert_eq!(&bytes[..6], &[1, 0, 2, 0, 3, 0]);
        assert_eq!(&bytes[34..38], &[4, 5, 6, 7]);
        assert_eq!(&bytes[62..], &[8, 0, 9, 0, 10, 0]);
    }
}
