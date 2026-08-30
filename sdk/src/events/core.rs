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
    /// An injected encoded-string descriptor could not encode a replacement.
    ReplacementEncodedStringEncode(samp_protocol::EncodeError<EventError>),
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
            Self::ReplacementEncodedStringEncode(_) => CallbackFailurePhase::ReplacementEncode,
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
        self.read_bits_into(&mut bytes, bit_len)?;
        Ok(bytes)
    }

    fn read_bits_into(&mut self, output: &mut [u8], bit_len: usize) -> Result<(), EventError> {
        if output.len() != bit_len.div_ceil(u8::BITS as usize) {
            return Err(EventError::InvalidBitLength {
                bit_len,
                byte_len: output.len(),
            });
        }
        self.host_result(unsafe {
            (self.api.raw().event_read_bits)(self.raw, output.as_mut_ptr(), bit_len)
        })
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

    fn read_left_aligned_bits_into(
        &mut self,
        output: &mut [u8],
        bit_len: usize,
    ) -> Result<(), Self::Error> {
        self.read_bits_into(output, bit_len)
    }
}

impl<'callback> samp_protocol::EncodedStringRead for Event<'callback> {
    fn read_encoded_string(&mut self, max_len: usize) -> Result<Vec<u8>, Self::Error> {
        let capacity = max_len.checked_add(1).ok_or(EventError::ValueOutOfRange {
            value: max_len,
            maximum: usize::MAX - 1,
        })?;
        Event::read_encoded_string(self, capacity)
    }
}

pub(super) struct HostEncodedStringWriter {
    api: HostApi,
    stream: samp_protocol::BitStream,
}

impl HostEncodedStringWriter {
    pub(super) fn new(api: HostApi) -> Self {
        Self {
            api,
            stream: samp_protocol::BitStream::new(),
        }
    }

    fn write_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), EventError> {
        samp_protocol::BitWrite::write_left_aligned_bits(&mut self.stream, bytes, bit_len).map_err(
            |error| match error {
                samp_protocol::BitStreamError::InvalidBitLength { bit_len, byte_len } => {
                    EventError::InvalidBitLength { bit_len, byte_len }
                }
                samp_protocol::BitStreamError::OutOfBounds {
                    requested_bits,
                    available_bits,
                } => EventError::ValueOutOfRange {
                    value: requested_bits,
                    maximum: available_bits,
                },
                samp_protocol::BitStreamError::PayloadTooLarge { requested_bits } => {
                    EventError::ValueOutOfRange {
                        value: requested_bits,
                        maximum: samp_protocol::MAX_BIT_STREAM_BITS,
                    }
                }
            },
        )
    }
}

impl samp_protocol::BitWrite for HostEncodedStringWriter {
    type Error = EventError;

    fn write_left_aligned_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), Self::Error> {
        self.write_bits(bytes, bit_len)
    }
}

impl samp_protocol::EncodedStringWrite for HostEncodedStringWriter {
    fn write_encoded_string(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        if value.len() > samp_protocol::limits::MAX_ENCODED_STRING_BYTES {
            return Err(EventError::ValueOutOfRange {
                value: value.len(),
                maximum: samp_protocol::limits::MAX_ENCODED_STRING_BYTES,
            });
        }
        let encoded = self.api.encode_string(value).map_err(EventError::Host)?;
        self.write_bits(encoded.as_bytes(), encoded.len_bits())
    }

    fn finish_encoded_bits(
        self,
    ) -> Result<samp_protocol::EncodedBits, samp_protocol::EncodedBitsError> {
        samp_protocol::EncodedBits::from_bits(
            self.stream.as_bytes().to_vec(),
            self.stream.len_bits(),
        )
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

/// Handles one Protocol descriptor that injects the Host encoded-string codec.
pub(crate) fn handle_encoded_string_protocol<D>(
    event: &mut Event<'_>,
    handler: impl FnOnce(D::Value) -> ProtocolAction<D::Value>,
) -> Result<SampClientSdkHookAction, ProtocolEventError>
where
    D: samp_protocol::EncodedStringWireDescriptor,
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
            let payload = D::encode_bits(HostEncodedStringWriter::new(event.api), &value)
                .map_err(ProtocolEventError::ReplacementEncodedStringEncode)?;
            event
                .replace_bits(payload.as_bytes(), payload.len_bits())
                .map_err(ProtocolEventError::ReplacementHost)?;
            Ok(SampClientSdkHookAction::Continue)
        }
    }
}
