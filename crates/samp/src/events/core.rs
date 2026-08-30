use crate::{Action, Event};
use core::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ProtocolAction<T> {
    Continue,
    Block,
    Replace(T),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventError {
    Host(modkit_abi::ModResult),
    ValueOutOfRange { value: usize, maximum: usize },
    InvalidBitLength { bit_len: usize, byte_len: usize },
}

impl fmt::Display for EventError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(result) => write!(formatter, "SA-MP event operation failed: {result:?}"),
            Self::ValueOutOfRange { value, maximum } => {
                write!(formatter, "value {value} exceeds the maximum {maximum}")
            }
            Self::InvalidBitLength { bit_len, byte_len } => write!(
                formatter,
                "bit length {bit_len} exceeds the {byte_len}-byte payload"
            ),
        }
    }
}

impl std::error::Error for EventError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackFailurePhase {
    DecodeSource,
    DecodeMalformed,
    ReplacementEncode,
    ReplacementHost,
}

enum ProtocolEventError {
    DecodeSource,
    DecodeMalformed,
    ReplacementEncode,
    ReplacementHost,
}

impl ProtocolEventError {
    const fn phase(&self) -> CallbackFailurePhase {
        match self {
            Self::DecodeSource => CallbackFailurePhase::DecodeSource,
            Self::DecodeMalformed => CallbackFailurePhase::DecodeMalformed,
            Self::ReplacementEncode => CallbackFailurePhase::ReplacementEncode,
            Self::ReplacementHost => CallbackFailurePhase::ReplacementHost,
        }
    }
}

impl samp_protocol::BitRead for Event<'_> {
    type Error = EventError;

    fn remaining_bits(&self) -> usize {
        Event::remaining_bits(self)
    }

    fn read_left_aligned_bits_into(
        &mut self,
        output: &mut [u8],
        bit_len: usize,
    ) -> Result<(), Self::Error> {
        if output.len() != bit_len.div_ceil(u8::BITS as usize) {
            return Err(EventError::InvalidBitLength {
                bit_len,
                byte_len: output.len(),
            });
        }
        self.read_bits_into(output, bit_len)
            .map_err(EventError::Host)
    }
}

impl samp_protocol::EncodedStringRead for Event<'_> {
    fn read_encoded_string(&mut self, max_len: usize) -> Result<Vec<u8>, Self::Error> {
        let capacity = max_len.checked_add(1).ok_or(EventError::ValueOutOfRange {
            value: max_len,
            maximum: usize::MAX - 1,
        })?;
        let mut bytes = vec![0; capacity];
        let len = Event::read_encoded_string(self, &mut bytes).map_err(EventError::Host)?;
        if len > bytes.len() {
            return Err(EventError::Host(modkit_abi::MOD_NATIVE_CALL_FAILED));
        }
        bytes.truncate(len);
        Ok(bytes)
    }
}

struct HostEncodedStringWriter<'borrow, 'callback> {
    event: &'borrow Event<'callback>,
    stream: samp_protocol::BitStream,
}

impl<'borrow, 'callback> HostEncodedStringWriter<'borrow, 'callback> {
    fn new(event: &'borrow Event<'callback>) -> Self {
        Self {
            event,
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

impl samp_protocol::BitWrite for HostEncodedStringWriter<'_, '_> {
    type Error = EventError;

    fn write_left_aligned_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), Self::Error> {
        self.write_bits(bytes, bit_len)
    }
}

impl samp_protocol::EncodedStringWrite for HostEncodedStringWriter<'_, '_> {
    fn write_encoded_string(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        if value.len() > samp_protocol::limits::MAX_ENCODED_STRING_BYTES {
            return Err(EventError::ValueOutOfRange {
                value: value.len(),
                maximum: samp_protocol::limits::MAX_ENCODED_STRING_BYTES,
            });
        }
        let mut encoded = vec![0; 4_096];
        let (byte_len, bit_len) = self
            .event
            .encode_string(value, &mut encoded)
            .map_err(EventError::Host)?;
        if byte_len > encoded.len() {
            return Err(EventError::Host(modkit_abi::MOD_NATIVE_CALL_FAILED));
        }
        self.write_bits(&encoded[..byte_len], bit_len)
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

pub(crate) fn handle_protocol<D>(
    event: &mut Event<'_>,
    handler: impl FnOnce(D::Value) -> ProtocolAction<D::Value>,
) -> Result<Action, CallbackFailurePhase>
where
    D: samp_protocol::WireDescriptor,
{
    if event.id() != D::ID {
        return Ok(Action::Continue);
    }
    event
        .reset_read()
        .map_err(|_| ProtocolEventError::DecodeSource.phase())?;
    let value = D::decode_from(event).map_err(|error| match error {
        samp_protocol::DecodeError::Source(_) => ProtocolEventError::DecodeSource.phase(),
        _ => ProtocolEventError::DecodeMalformed.phase(),
    })?;
    match handler(value) {
        ProtocolAction::Continue => Ok(Action::Continue),
        ProtocolAction::Block => Ok(Action::Block),
        ProtocolAction::Replace(value) => {
            let payload = D::encode_bits(&value)
                .map_err(|_| ProtocolEventError::ReplacementEncode.phase())?;
            event
                .replace_bits(payload.as_bytes(), payload.len_bits())
                .map_err(|_| ProtocolEventError::ReplacementHost.phase())?;
            Ok(Action::Continue)
        }
    }
}

pub(crate) fn handle_encoded_string_protocol<D>(
    event: &mut Event<'_>,
    handler: impl FnOnce(D::Value) -> ProtocolAction<D::Value>,
) -> Result<Action, CallbackFailurePhase>
where
    D: samp_protocol::EncodedStringWireDescriptor,
{
    if event.id() != D::ID {
        return Ok(Action::Continue);
    }
    event
        .reset_read()
        .map_err(|_| ProtocolEventError::DecodeSource.phase())?;
    let value = D::decode_from(event).map_err(|error| match error {
        samp_protocol::DecodeError::Source(_) => ProtocolEventError::DecodeSource.phase(),
        _ => ProtocolEventError::DecodeMalformed.phase(),
    })?;
    match handler(value) {
        ProtocolAction::Continue => Ok(Action::Continue),
        ProtocolAction::Block => Ok(Action::Block),
        ProtocolAction::Replace(value) => {
            let payload = D::encode_bits(HostEncodedStringWriter::new(event), &value)
                .map_err(|_| ProtocolEventError::ReplacementEncode.phase())?;
            event
                .replace_bits(payload.as_bytes(), payload.len_bits())
                .map_err(|_| ProtocolEventError::ReplacementHost.phase())?;
            Ok(Action::Continue)
        }
    }
}
