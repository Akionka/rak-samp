//! Raw mock event ABI callbacks for SDK tests.

use super::*;

#[repr(C)]
pub(crate) struct TestEvent {
    id: u8,
    pub(crate) bytes: Vec<u8>,
    pub(crate) bit_len: usize,
    read_offset: usize,
    reset_read_result: SampClientSdkResult,
    replace_result: SampClientSdkResult,
    pub(crate) replacement_calls: usize,
}

impl TestEvent {
    pub(crate) fn new(id: u8, payload: EncodedPayload) -> Self {
        Self {
            id,
            bytes: payload.bytes,
            bit_len: payload.bit_len,
            read_offset: 0,
            reset_read_result: SampClientSdkResult::Ok,
            replace_result: SampClientSdkResult::Ok,
            replacement_calls: 0,
        }
    }

    pub(crate) fn with_results(
        id: u8,
        payload: EncodedPayload,
        reset_read_result: SampClientSdkResult,
        replace_result: SampClientSdkResult,
    ) -> Self {
        Self {
            reset_read_result,
            replace_result,
            ..Self::new(id, payload)
        }
    }
}

unsafe fn test_event<'a>(event: *mut SampClientSdkEventV1) -> &'a mut TestEvent {
    unsafe { &mut *event.cast::<TestEvent>() }
}

pub(super) unsafe extern "system" fn test_event_id(event: *const SampClientSdkEventV1) -> u8 {
    unsafe { (&*event.cast::<TestEvent>()).id }
}

pub(super) unsafe extern "system" fn test_event_reset_read(
    event: *mut SampClientSdkEventV1,
) -> SampClientSdkResult {
    let event = unsafe { test_event(event) };
    if event.reset_read_result == SampClientSdkResult::Ok {
        event.read_offset = 0;
    }
    event.reset_read_result
}

pub(super) unsafe extern "system" fn test_event_clear(
    event: *mut SampClientSdkEventV1,
) -> SampClientSdkResult {
    let event = unsafe { test_event(event) };
    event.bytes.clear();
    event.bit_len = 0;
    event.read_offset = 0;
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn test_event_read_bits(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
    bit_len: usize,
) -> SampClientSdkResult {
    let event = unsafe { test_event(event) };
    if event.read_offset.saturating_add(bit_len) > event.bit_len {
        return SampClientSdkResult::ReadOutOfBounds;
    }
    let byte_len = bit_len.div_ceil(u8::BITS as usize);
    if byte_len != 0 {
        unsafe { ptr::write_bytes(output, 0, byte_len) };
    }
    for bit in 0..bit_len {
        let source =
            event.bytes[(event.read_offset + bit) / 8] & (0x80 >> ((event.read_offset + bit) % 8));
        if source != 0 {
            unsafe { *output.add(bit / 8) |= 0x80 >> (bit % 8) };
        }
    }
    event.read_offset += bit_len;
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn test_event_read_u8(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
) -> SampClientSdkResult {
    unsafe { test_event_read_bits(event, output, 8) }
}

pub(super) unsafe extern "system" fn test_event_read_u16(
    event: *mut SampClientSdkEventV1,
    output: *mut u16,
) -> SampClientSdkResult {
    let mut bytes = [0; 2];
    let result = unsafe { test_event_read_bits(event, bytes.as_mut_ptr(), 16) };
    if result == SampClientSdkResult::Ok {
        unsafe { output.write(u16::from_le_bytes(bytes)) };
    }
    result
}

pub(super) unsafe extern "system" fn test_event_read_u32(
    event: *mut SampClientSdkEventV1,
    output: *mut u32,
) -> SampClientSdkResult {
    let mut bytes = [0; 4];
    let result = unsafe { test_event_read_bits(event, bytes.as_mut_ptr(), 32) };
    if result == SampClientSdkResult::Ok {
        unsafe { output.write(u32::from_le_bytes(bytes)) };
    }
    result
}

pub(super) unsafe extern "system" fn test_event_read_f32(
    event: *mut SampClientSdkEventV1,
    output: *mut f32,
) -> SampClientSdkResult {
    let mut bits = 0;
    let result = unsafe { test_event_read_u32(event, &raw mut bits) };
    if result == SampClientSdkResult::Ok {
        unsafe { output.write(f32::from_bits(bits)) };
    }
    result
}

pub(super) unsafe extern "system" fn test_event_read_bytes(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
    byte_len: usize,
) -> SampClientSdkResult {
    unsafe { test_event_read_bits(event, output, byte_len * 8) }
}

pub(super) unsafe extern "system" fn test_event_write_u8(
    _event: *mut SampClientSdkEventV1,
    _value: u8,
) -> SampClientSdkResult {
    SampClientSdkResult::NativeCallFailed
}

pub(super) unsafe extern "system" fn test_event_write_u16(
    _event: *mut SampClientSdkEventV1,
    _value: u16,
) -> SampClientSdkResult {
    SampClientSdkResult::NativeCallFailed
}

pub(super) unsafe extern "system" fn test_event_write_u32(
    _event: *mut SampClientSdkEventV1,
    _value: u32,
) -> SampClientSdkResult {
    SampClientSdkResult::NativeCallFailed
}

pub(super) unsafe extern "system" fn test_event_write_f32(
    _event: *mut SampClientSdkEventV1,
    _value: f32,
) -> SampClientSdkResult {
    SampClientSdkResult::NativeCallFailed
}

pub(super) unsafe extern "system" fn test_event_write_bytes(
    _event: *mut SampClientSdkEventV1,
    _value: *const u8,
    _byte_len: usize,
) -> SampClientSdkResult {
    SampClientSdkResult::NativeCallFailed
}

pub(super) unsafe extern "system" fn test_event_replace_bytes(
    event: *mut SampClientSdkEventV1,
    bytes: *const u8,
    byte_len: usize,
) -> SampClientSdkResult {
    unsafe { test_event_replace_bits(event, bytes, byte_len, byte_len * 8) }
}

pub(super) unsafe extern "system" fn test_event_replace_bits(
    event: *mut SampClientSdkEventV1,
    bytes: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> SampClientSdkResult {
    if bit_len > byte_len.saturating_mul(8) {
        return SampClientSdkResult::InvalidArgument;
    }
    let event = unsafe { test_event(event) };
    event.replacement_calls += 1;
    if event.replace_result != SampClientSdkResult::Ok {
        return event.replace_result;
    }
    event.bytes = if byte_len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(bytes, byte_len) }.to_vec()
    };
    event.bit_len = bit_len;
    event.read_offset = 0;
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn test_event_remaining_bits(
    event: *mut SampClientSdkEventV1,
) -> usize {
    let event = unsafe { test_event(event) };
    event.bit_len - event.read_offset
}

pub(super) unsafe extern "system" fn test_encoded_string(
    value: *const u8,
    value_len: usize,
    output: *mut u8,
    output_capacity: usize,
    bit_len: *mut usize,
) -> SampClientSdkResult {
    if (value.is_null() && value_len != 0) || output.is_null() || bit_len.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    if value_len > u16::MAX as usize {
        return SampClientSdkResult::PayloadTooLarge;
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
        return SampClientSdkResult::PayloadTooLarge;
    }
    unsafe {
        ptr::copy_nonoverlapping(encoded.bytes.as_ptr(), output, encoded.bytes.len());
        bit_len.write(encoded.bit_len);
    }
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn test_read_encoded_string(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
    output_capacity: usize,
    output_len: *mut usize,
) -> SampClientSdkResult {
    if output.is_null() || output_len.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let mut length = 0;
    let result = unsafe { test_event_read_u16(event, &raw mut length) };
    if result != SampClientSdkResult::Ok {
        return result;
    }
    let length = usize::from(length);
    if length > output_capacity {
        return SampClientSdkResult::PayloadTooLarge;
    }
    let result = unsafe { test_event_read_bytes(event, output, length) };
    if result == SampClientSdkResult::Ok {
        unsafe { output_len.write(length) };
    }
    result
}
