//! Typed SA-MP Packet and RPC helpers modeled after MoonLoader's `samp.events`.
//!
//! Public consumers register typed descriptors through [`crate::Net`]. The private callback
//! adapter filters IDs, decodes payloads, and converts [`ProtocolAction`] to the Host ABI action.
//! `Replace` serializes the complete Packet or RPC payload before one atomic Host mutation.
//!
//! Text fields deliberately use `Vec<u8>`: SA-MP text is not guaranteed to be UTF-8. Use
//! [`std::str::from_utf8`] only when the server's encoding is known.

use crate::{HostApi, SampClientSdkEventV1, SampClientSdkHookAction, SampClientSdkResult};
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

/// The typed decision returned by a Packet or RPC helper callback.
#[derive(Clone, Debug, PartialEq)]
pub enum ProtocolAction<T> {
    /// Preserve the original Packet or RPC payload.
    Continue,
    /// Do not pass the Packet or RPC to SA-MP.
    Block,
    /// Replace the complete Packet or RPC payload with this typed value.
    Replace(T),
}

/// An error while decoding or rewriting a callback-local RPC event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventError {
    /// The host rejected an operation on the callback event.
    Host(SampClientSdkResult),
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
                write!(
                    formatter,
                    "samp-client-sdk host event operation failed: {result:?}"
                )
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
            Self::NullEvent => {
                formatter.write_str("samp-client-sdk supplied a null callback event")
            }
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

/// A classified Protocol failure while adapting a callback-local event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProtocolEventError {
    /// The Host rejected a source read before a typed value was produced.
    DecodeSource(EventError),
    /// Protocol validation rejected the callback payload.
    DecodeMalformed(samp_protocol::DecodeError<EventError>),
    /// The Protocol descriptor could not canonically encode a replacement.
    ReplacementEncode(samp_protocol::EncodeError<samp_protocol::BitStreamError>),
    /// The Host rejected the completed replacement payload.
    ReplacementHost(EventError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackFailurePhase {
    DecodeSource,
    DecodeMalformed,
    ReplacementEncode,
    ReplacementHost,
}

impl ProtocolEventError {
    pub(crate) fn phase(&self) -> CallbackFailurePhase {
        match self {
            Self::DecodeSource(_) => CallbackFailurePhase::DecodeSource,
            Self::DecodeMalformed(_) => CallbackFailurePhase::DecodeMalformed,
            Self::ReplacementEncode(_) => CallbackFailurePhase::ReplacementEncode,
            Self::ReplacementHost(_) => CallbackFailurePhase::ReplacementHost,
        }
    }
}

/// A callback-local view over an opaque host event.
///
/// Raw [`crate::Net::on_packet`] and [`crate::Net::on_rpc`] callbacks receive this value. It may
/// not be retained after that handler returns.
pub struct Event<'callback> {
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    _callback: PhantomData<&'callback mut SampClientSdkEventV1>,
}

impl<'callback> Event<'callback> {
    /// Creates an event view for the raw pointer supplied to a host callback.
    ///
    /// # Safety
    ///
    /// `raw` must be the event pointer received from a currently executing `samp_client_sdk` callback and
    /// the returned value must not outlive that callback.
    pub(crate) unsafe fn from_callback(
        api: HostApi,
        raw: *mut SampClientSdkEventV1,
    ) -> Result<Self, EventError> {
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
            return Err(EventError::Host(SampClientSdkResult::NativeCallFailed));
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

    fn host_result(&self, result: SampClientSdkResult) -> Result<(), EventError> {
        if result == SampClientSdkResult::Ok {
            Ok(())
        } else {
            Err(EventError::Host(result))
        }
    }
}

impl<'callback> samp_protocol::BitRead for Event<'callback> {
    type Error = EventError;

    fn remaining_bits(&self) -> usize {
        self.remaining_bits()
    }

    fn read_left_aligned_bits(&mut self, bit_len: usize) -> Result<Vec<u8>, Self::Error> {
        self.read_bits(bit_len)
    }
}

pub(super) struct PayloadWriter {
    bytes: Vec<u8>,
    bit_len: usize,
}

impl PayloadWriter {
    pub(super) fn new() -> Self {
        Self {
            bytes: Vec::new(),
            bit_len: 0,
        }
    }

    pub(super) fn finish_bits(self) -> EncodedPayload {
        EncodedPayload {
            bytes: self.bytes,
            bit_len: self.bit_len,
        }
    }

    pub(super) fn u8(&mut self, value: u8) {
        self.bytes(&[value]);
    }

    pub(super) fn u16(&mut self, value: u16) {
        self.bytes(&value.to_le_bytes());
    }

    pub(super) fn u32(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }

    pub(super) fn f32(&mut self, value: f32) {
        self.u32(value.to_bits());
    }

    pub(super) fn bytes(&mut self, value: &[u8]) {
        self.bits(value, value.len() * u8::BITS as usize);
    }

    pub(super) fn bits(&mut self, value: &[u8], bit_len: usize) {
        debug_assert!(bit_len <= value.len() * u8::BITS as usize);
        for bit_offset in 0..bit_len {
            self.bit(value[bit_offset / 8] & (0x80 >> (bit_offset % 8)) != 0);
        }
    }

    pub(super) fn bit(&mut self, value: bool) {
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

    pub(super) fn string8(&mut self, value: &[u8]) -> Result<(), EventError> {
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

    pub(super) fn vector3(&mut self, value: Vector3) {
        self.f32(value.x);
        self.f32(value.y);
        self.f32(value.z);
    }

    pub(super) fn encoded_string(&mut self, api: HostApi, value: &[u8]) -> Result<(), EventError> {
        self.encoded_string_with_limit(api, value, MAX_ENCODED_STRING_BYTES)
    }

    pub(super) fn encoded_string_with_limit(
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

impl samp_protocol::BitWrite for PayloadWriter {
    type Error = EventError;

    fn write_left_aligned_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), Self::Error> {
        if bit_len > bytes.len().saturating_mul(u8::BITS as usize) {
            return Err(EventError::InvalidBitLength {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        self.bits(bytes, bit_len);
        Ok(())
    }
}

/// A complete callback replacement with an exact bit length.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EncodedPayload {
    pub(super) bytes: Vec<u8>,
    pub(super) bit_len: usize,
}

impl EncodedPayload {
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Result<Self, EventError> {
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

    #[cfg(test)]
    pub(crate) fn from_bits(bytes: Vec<u8>, bit_len: usize) -> Result<Self, EventError> {
        if bit_len > bytes.len().saturating_mul(u8::BITS as usize) {
            return Err(EventError::InvalidBitLength {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        Ok(Self { bytes, bit_len })
    }

    #[must_use]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub(crate) const fn len_bits(&self) -> usize {
        self.bit_len
    }
}

pub(super) enum RpcEncoder<T> {
    Bytes(fn(T) -> Result<Vec<u8>, EventError>),
    Bits(fn(HostApi, T) -> Result<EncodedPayload, EventError>),
}

impl<T> Copy for RpcEncoder<T> {}

impl<T> Clone for RpcEncoder<T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A typed RPC descriptor with its SA-MP RPC ID and read/write layout.
pub(crate) struct Rpc<T> {
    pub(super) id: u8,
    pub(super) decode: fn(&mut Event<'_>) -> Result<T, EventError>,
    pub(super) encode: RpcEncoder<T>,
}

impl<T> Copy for Rpc<T> {}

impl<T> Clone for Rpc<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Rpc<T> {
    /// Creates a descriptor for one RPC ID.
    pub(crate) const fn new(
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
    pub(crate) const fn new_bits(
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
    pub(crate) fn encode(self, api: HostApi, value: T) -> Result<EncodedPayload, EventError> {
        match self.encode {
            RpcEncoder::Bytes(encode) => EncodedPayload::from_bytes(encode(value)?),
            RpcEncoder::Bits(encode) => encode(api, value),
        }
    }

    /// Handles this RPC when `event` has the matching ID.
    ///
    /// A non-matching event passes through without invoking `handler`. Decode failures are returned
    /// to the plugin so it can fail open and report the incompatible payload if appropriate.
    pub(crate) fn handle(
        self,
        event: &mut Event<'_>,
        handler: impl FnOnce(T) -> ProtocolAction<T>,
    ) -> Result<SampClientSdkHookAction, EventError> {
        self.handle_classified(event, handler)
            .map_err(|(_, error)| error)
    }

    pub(crate) fn handle_classified(
        self,
        event: &mut Event<'_>,
        handler: impl FnOnce(T) -> ProtocolAction<T>,
    ) -> Result<SampClientSdkHookAction, (CallbackFailurePhase, EventError)> {
        if event.id() != self.id {
            return Ok(SampClientSdkHookAction::Continue);
        }
        event
            .reset_read()
            .map_err(|error| (CallbackFailurePhase::DecodeSource, error))?;
        let value = (self.decode)(event).map_err(|error| {
            let phase = if matches!(&error, EventError::Host(_)) {
                CallbackFailurePhase::DecodeSource
            } else {
                CallbackFailurePhase::DecodeMalformed
            };
            (phase, error)
        })?;
        if event.remaining_bits() != 0 {
            return Err((
                CallbackFailurePhase::DecodeMalformed,
                EventError::UnexpectedBitLength {
                    bit_len: event.remaining_bits(),
                    expected: 0,
                },
            ));
        }
        match handler(value) {
            ProtocolAction::Continue => Ok(SampClientSdkHookAction::Continue),
            ProtocolAction::Block => Ok(SampClientSdkHookAction::Block),
            ProtocolAction::Replace(value) => {
                let payload = self
                    .encode(event.api, value)
                    .map_err(|error| (CallbackFailurePhase::ReplacementEncode, error))?;
                event
                    .replace_bits(payload.as_bytes(), payload.len_bits())
                    .map_err(|error| (CallbackFailurePhase::ReplacementHost, error))?;
                Ok(SampClientSdkHookAction::Continue)
            }
        }
    }
}

macro_rules! directional_descriptor {
    ($name:ident) => {
        #[doc = concat!("A typed ", stringify!($name), " descriptor.")]
        pub struct $name<T>(Rpc<T>);

        impl<T> Copy for $name<T> {}

        impl<T> Clone for $name<T> {
            fn clone(&self) -> Self {
                *self
            }
        }

        #[allow(
            dead_code,
            reason = "legacy descriptor directions share one private adapter shape"
        )]
        impl<T> $name<T> {
            /// Creates a descriptor for one ID with a byte-aligned payload.
            pub(crate) const fn new(
                id: u8,
                decode: fn(&mut Event<'_>) -> Result<T, EventError>,
                encode: fn(T) -> Result<Vec<u8>, EventError>,
            ) -> Self {
                Self(Rpc::new(id, decode, encode))
            }

            /// Creates a descriptor with an exact-bit payload.
            pub(crate) const fn new_bits(
                id: u8,
                decode: fn(&mut Event<'_>) -> Result<T, EventError>,
                encode: fn(HostApi, T) -> Result<EncodedPayload, EventError>,
            ) -> Self {
                Self(Rpc::new_bits(id, decode, encode))
            }

            /// Returns this descriptor's packet or RPC ID.
            #[must_use]
            pub const fn id(self) -> u8 {
                self.0.id()
            }

            /// Serializes one complete payload without mutating a callback event.
            pub(crate) fn encode(
                self,
                api: HostApi,
                value: T,
            ) -> Result<EncodedPayload, EventError> {
                self.0.encode(api, value)
            }

            /// Handles this descriptor when `event` has the matching ID.
            pub(crate) fn handle(
                self,
                event: &mut Event<'_>,
                handler: impl FnOnce(T) -> ProtocolAction<T>,
            ) -> Result<SampClientSdkHookAction, EventError> {
                self.0.handle(event, handler)
            }

            pub(crate) fn handle_classified(
                self,
                event: &mut Event<'_>,
                handler: impl FnOnce(T) -> ProtocolAction<T>,
            ) -> Result<SampClientSdkHookAction, (CallbackFailurePhase, EventError)> {
                self.0.handle_classified(event, handler)
            }
        }

        impl<T> TypedDescriptor<T> for $name<T> {
            fn into_rpc(self) -> Rpc<T> {
                self.0
            }
        }
    };
}

directional_descriptor!(IncomingPacket);
directional_descriptor!(OutgoingPacket);
directional_descriptor!(IncomingRpc);
directional_descriptor!(OutgoingRpc);

pub(crate) trait TypedDescriptor<T> {
    fn into_rpc(self) -> Rpc<T>;
}

impl<T> TypedDescriptor<T> for Rpc<T> {
    fn into_rpc(self) -> Rpc<T> {
        self
    }
}

/// Handles one Protocol Packet or RPC descriptor from a raw callback event.
///
/// The callback payload stays borrowed by [`Event`] during decoding. Replacements are separately
/// serialized into owned bytes before the atomic ABI replacement call.
pub(crate) fn handle_protocol<D>(
    event: &mut Event<'_>,
    handler: impl FnOnce(D::Value) -> ProtocolAction<D::Value>,
) -> Result<SampClientSdkHookAction, ProtocolEventError>
where
    D: samp_protocol::WireDescriptor,
{
    if event.id() != D::ID {
        return Ok(SampClientSdkHookAction::Continue);
    }
    event
        .reset_read()
        .map_err(ProtocolEventError::DecodeSource)?;
    let value = D::decode_from(event).map_err(|error| match error {
        samp_protocol::DecodeError::Source(error) => ProtocolEventError::DecodeSource(error),
        error => ProtocolEventError::DecodeMalformed(error),
    })?;
    match handler(value) {
        ProtocolAction::Continue => Ok(SampClientSdkHookAction::Continue),
        ProtocolAction::Block => Ok(SampClientSdkHookAction::Block),
        ProtocolAction::Replace(value) => {
            let payload = D::encode_bits(&value).map_err(ProtocolEventError::ReplacementEncode)?;
            event
                .replace_bits(payload.as_bytes(), payload.len_bits())
                .map_err(ProtocolEventError::ReplacementHost)?;
            Ok(SampClientSdkHookAction::Continue)
        }
    }
}

/// Calls a typed descriptor from a raw callback event.
///
/// # Safety
///
/// `raw` must be the event pointer supplied to the currently executing callback. On an error,
/// return [`SampClientSdkHookAction::Continue`] so malformed traffic remains fail-open.
pub(crate) unsafe fn handle<T, D>(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    descriptor: D,
    handler: impl FnOnce(T) -> ProtocolAction<T>,
) -> Result<SampClientSdkHookAction, EventError>
where
    D: TypedDescriptor<T>,
{
    let mut event = unsafe { Event::from_callback(api, raw) }?;
    descriptor.into_rpc().handle(&mut event, handler)
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
