//! Event bitstream and native string-codec ABI entry points.

use super::{AbiEvent, clone_initialized, host};
use crate::{BitStream, BitStreamError, runtime::CodecError};
use sdk_abi::{SampClientSdkEventV1, SampClientSdkResult};
use std::ptr;

pub(super) const MAX_CODEC_INPUT_BITS: usize = 16 * 1024 * 1024 * u8::BITS as usize;
pub(super) const MAX_CODEC_OUTPUT_BYTES: usize = 4_096;
pub(super) unsafe extern "system" fn event_id(event: *const SampClientSdkEventV1) -> u8 {
    if event.is_null() {
        return 0;
    }
    unsafe { event.cast::<AbiEvent>().as_ref() }.map_or(0, |event| event.id)
}

pub(super) unsafe extern "system" fn event_reset_read(
    event: *mut SampClientSdkEventV1,
) -> SampClientSdkResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe { &mut *event.payload }.reset_read();
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn event_clear(
    event: *mut SampClientSdkEventV1,
) -> SampClientSdkResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe { &mut *event.payload }.clear();
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn event_read_u8(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u8() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

pub(super) unsafe extern "system" fn event_read_u16(
    event: *mut SampClientSdkEventV1,
    output: *mut u16,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u16() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

pub(super) unsafe extern "system" fn event_read_u32(
    event: *mut SampClientSdkEventV1,
    output: *mut u32,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u32() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

pub(super) unsafe extern "system" fn event_read_f32(
    event: *mut SampClientSdkEventV1,
    output: *mut f32,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_f32() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

pub(super) unsafe extern "system" fn event_read_bytes(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
    len: usize,
) -> SampClientSdkResult {
    if output.is_null() && len != 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_bytes(len) {
        Ok(bytes) => {
            if len != 0 {
                unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), output, len) };
            }
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

pub(super) unsafe extern "system" fn event_write_u8(
    event: *mut SampClientSdkEventV1,
    value: u8,
) -> SampClientSdkResult {
    write_event(event, |stream| stream.write_u8(value))
}

pub(super) unsafe extern "system" fn event_write_u16(
    event: *mut SampClientSdkEventV1,
    value: u16,
) -> SampClientSdkResult {
    write_event(event, |stream| stream.write_u16(value))
}

pub(super) unsafe extern "system" fn event_write_u32(
    event: *mut SampClientSdkEventV1,
    value: u32,
) -> SampClientSdkResult {
    write_event(event, |stream| stream.write_u32(value))
}

pub(super) unsafe extern "system" fn event_write_f32(
    event: *mut SampClientSdkEventV1,
    value: f32,
) -> SampClientSdkResult {
    write_event(event, |stream| stream.write_f32(value))
}

pub(super) unsafe extern "system" fn event_write_bytes(
    event: *mut SampClientSdkEventV1,
    value: *const u8,
    len: usize,
) -> SampClientSdkResult {
    if value.is_null() && len != 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, len) }
    };
    write_event(event, |stream| stream.write_bytes(bytes))
}

pub(super) unsafe extern "system" fn event_replace_bytes(
    event: *mut SampClientSdkEventV1,
    value: *const u8,
    len: usize,
) -> SampClientSdkResult {
    if value.is_null() && len != 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, len) }
    };
    write_event(event, |stream| stream.replace_bytes(bytes))
}

pub(super) unsafe extern "system" fn event_remaining_bits(
    event: *mut SampClientSdkEventV1,
) -> usize {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return 0;
    };
    unsafe { &*event.payload }.remaining_bits()
}

pub(super) unsafe extern "system" fn event_read_bits(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
    bit_len: usize,
) -> SampClientSdkResult {
    if output.is_null() && bit_len != 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_bits(bit_len) {
        Ok(bytes) => {
            if !bytes.is_empty() {
                unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len()) };
            }
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

pub(super) unsafe extern "system" fn event_replace_bits(
    event: *mut SampClientSdkEventV1,
    value: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> SampClientSdkResult {
    if value.is_null() && byte_len != 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    if bit_len > byte_len.saturating_mul(u8::BITS as usize) {
        return SampClientSdkResult::InvalidArgument;
    }
    let bytes = if byte_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, byte_len) }
    };
    write_event(event, |stream| stream.replace_bits(bytes, bit_len))
}

pub(super) unsafe extern "system" fn encode_string(
    value: *const u8,
    value_len: usize,
    output: *mut u8,
    output_capacity: usize,
    output_bit_len: *mut usize,
) -> SampClientSdkResult {
    if (value.is_null() && value_len != 0)
        || (output.is_null() && output_capacity != 0)
        || output_bit_len.is_null()
    {
        return SampClientSdkResult::InvalidArgument;
    }
    let value = if value_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, value_len) }
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let encoded = match runtime.encode_string(value) {
        Ok(encoded) => encoded,
        Err(error) => return codec_result(error),
    };
    if encoded.len_bytes() > output_capacity {
        return SampClientSdkResult::PayloadTooLarge;
    }
    if encoded.len_bytes() != 0 {
        unsafe {
            ptr::copy_nonoverlapping(encoded.as_bytes().as_ptr(), output, encoded.len_bytes())
        };
    }
    unsafe { output_bit_len.write(encoded.len_bits()) };
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn event_read_encoded_string(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
    output_capacity: usize,
    output_len: *mut usize,
) -> SampClientSdkResult {
    if output.is_null() || output_capacity == 0 || output_len.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let output = unsafe { std::slice::from_raw_parts_mut(output, output_capacity) };
    match runtime.decode_string(unsafe { &mut *event.payload }, output) {
        Ok(length) => {
            unsafe { output_len.write(length) };
            SampClientSdkResult::Ok
        }
        Err(error) => codec_result(error),
    }
}

pub(super) unsafe extern "system" fn decode_string(
    input: *const u8,
    input_byte_len: usize,
    input_bit_len: usize,
    input_read_offset: usize,
    output: *mut u8,
    output_capacity: usize,
    output_len: *mut usize,
    output_read_offset: *mut usize,
) -> SampClientSdkResult {
    if (input.is_null() && input_byte_len != 0)
        || output.is_null()
        || output_capacity == 0
        || output_len.is_null()
        || output_read_offset.is_null()
        || input_bit_len > input_byte_len.saturating_mul(u8::BITS as usize)
        || input_read_offset > input_bit_len
    {
        return SampClientSdkResult::InvalidArgument;
    }
    if input_bit_len > MAX_CODEC_INPUT_BITS || output_capacity > MAX_CODEC_OUTPUT_BYTES {
        return SampClientSdkResult::PayloadTooLarge;
    }
    let input_len = input_bit_len.div_ceil(u8::BITS as usize);
    let input = if input_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(input, input_len) }
    };
    let Ok(mut payload) = BitStream::from_bytes_with_bits(input.to_vec(), input_bit_len) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if payload.set_read_offset_bits(input_read_offset).is_err() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let output = unsafe { std::slice::from_raw_parts_mut(output, output_capacity) };
    match runtime.decode_string(&mut payload, output) {
        Ok(length) => {
            let read_offset = payload.read_offset_bits();
            if length >= output_capacity || read_offset > input_bit_len {
                return SampClientSdkResult::NativeCallFailed;
            }
            unsafe {
                output_len.write(length);
                output_read_offset.write(read_offset);
            }
            SampClientSdkResult::Ok
        }
        Err(error) => codec_result(error),
    }
}

fn write_event(
    event: *mut SampClientSdkEventV1,
    operation: impl FnOnce(&mut BitStream) -> Result<(), BitStreamError>,
) -> SampClientSdkResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    operation(unsafe { &mut *event.payload })
        .map_or_else(bitstream_result, |_| SampClientSdkResult::Ok)
}

unsafe fn abi_event(event: *mut SampClientSdkEventV1) -> Result<&'static mut AbiEvent, ()> {
    let event = unsafe { event.cast::<AbiEvent>().as_mut() }.ok_or(())?;
    if event.payload.is_null() {
        return Err(());
    }
    Ok(event)
}

fn bitstream_result(error: BitStreamError) -> SampClientSdkResult {
    match error {
        BitStreamError::ReadOutOfBounds { .. } => SampClientSdkResult::ReadOutOfBounds,
        BitStreamError::CapacityExceeded { .. } => SampClientSdkResult::PayloadTooLarge,
        BitStreamError::InvalidOffset { .. } => SampClientSdkResult::InvalidArgument,
    }
}

fn codec_result(error: CodecError) -> SampClientSdkResult {
    match error {
        CodecError::ClientNotReady => SampClientSdkResult::NotReady,
        CodecError::InvalidArgument => SampClientSdkResult::InvalidArgument,
        CodecError::PayloadTooLarge => SampClientSdkResult::PayloadTooLarge,
        CodecError::NativeCallFailed => SampClientSdkResult::NativeCallFailed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_capacity_failure_preserves_host_payload_and_cursor() {
        let mut payload = BitStream::from_bytes_with_capacity(vec![0xA5], 8, 8).unwrap();
        payload.set_read_offset_bits(3).unwrap();
        let mut event = AbiEvent {
            id: 35,
            payload: &raw mut payload,
        };
        let replacement = [0x12, 0x34];

        let result = unsafe {
            event_replace_bits(
                (&raw mut event).cast::<SampClientSdkEventV1>(),
                replacement.as_ptr(),
                replacement.len(),
                16,
            )
        };

        assert_eq!(result, SampClientSdkResult::PayloadTooLarge);
        assert_eq!(payload.as_bytes(), &[0xA5]);
        assert_eq!(payload.len_bits(), 8);
        assert_eq!(payload.read_offset_bits(), 3);
    }
}
