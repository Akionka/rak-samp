use super::{BackendState, StringReadDecoderFn, StringWriteEncoderFn};
use crate::{BitStream, SendError, runtime::CodecError};
use std::{ffi::c_void, mem, ptr, slice};

pub(super) struct RawBitStream {
    number_of_bits_used: i32,
    number_of_bits_allocated: i32,
    read_offset: i32,
    data: *mut u8,
    #[allow(dead_code)] // Required by RakNet's native x86 `BitStream` layout.
    copy_data: bool,
    stack_data: [u8; 256],
}
impl RawBitStream {
    pub(super) unsafe fn copy_to_owned(&self) -> Result<BitStream, SendError> {
        if self.number_of_bits_used < 0 || self.number_of_bits_allocated < self.number_of_bits_used
        {
            return Err(SendError::NativeCallFailed);
        }
        let used = self.number_of_bits_used as usize;
        let allocated = self.number_of_bits_allocated as usize;
        let byte_len = used.div_ceil(u8::BITS as usize);
        if byte_len > 0 && self.data.is_null() {
            return Err(SendError::NativeCallFailed);
        }
        let bytes = if byte_len == 0 {
            Vec::new()
        } else {
            unsafe { slice::from_raw_parts(self.data, byte_len) }.to_vec()
        };
        BitStream::from_bytes_with_capacity(bytes, used, allocated)
            .map_err(|_| SendError::NativeCallFailed)
    }

    pub(super) unsafe fn replace_from(&mut self, stream: &BitStream) -> Result<(), SendError> {
        let capacity = self.number_of_bits_allocated.max(0) as usize;
        if stream.len_bits() > capacity {
            return Err(SendError::PayloadTooLarge);
        }
        if stream.len_bytes() > 0 && self.data.is_null() {
            return Err(SendError::NativeCallFailed);
        }
        unsafe {
            ptr::copy_nonoverlapping(stream.as_bytes().as_ptr(), self.data, stream.len_bytes());
        }
        self.number_of_bits_used = stream.len_bits() as i32;
        self.read_offset = 0;
        Ok(())
    }
}

pub(super) struct NativeBitStream {
    data: Vec<u8>,
    raw: RawBitStream,
}

impl NativeBitStream {
    pub(super) fn new(stream: &BitStream) -> Result<Self, SendError> {
        let bit_len = native_bit_length(stream.len_bits())?;
        let mut data = stream.as_bytes().to_vec();
        let data_pointer = if data.is_empty() {
            ptr::null_mut()
        } else {
            data.as_mut_ptr()
        };
        Ok(Self {
            raw: RawBitStream {
                number_of_bits_used: bit_len,
                number_of_bits_allocated: bit_len,
                read_offset: 0,
                data: data_pointer,
                copy_data: false,
                stack_data: [0; 256],
            },
            data,
        })
    }

    pub(super) fn empty_with_capacity_bits(capacity_bits: usize) -> Result<Self, SendError> {
        let allocated = native_bit_length(capacity_bits)?;
        let mut data = vec![0_u8; capacity_bits.div_ceil(u8::BITS as usize)];
        let data_pointer = if data.is_empty() {
            ptr::null_mut()
        } else {
            data.as_mut_ptr()
        };
        Ok(Self {
            raw: RawBitStream {
                number_of_bits_used: 0,
                number_of_bits_allocated: allocated,
                read_offset: 0,
                data: data_pointer,
                copy_data: false,
                stack_data: [0; 256],
            },
            data,
        })
    }

    pub(super) fn from_readable_stream(stream: &BitStream) -> Result<Self, SendError> {
        let mut native = Self::new(stream)?;
        native.raw.read_offset = native_bit_length(stream.read_offset_bits())?;
        Ok(native)
    }

    pub(super) fn read_offset(&self) -> Option<usize> {
        let read_offset = usize::try_from(self.raw.read_offset).ok()?;
        (read_offset <= usize::try_from(self.raw.number_of_bits_used).ok()?).then_some(read_offset)
    }

    pub(super) fn into_stream(mut self) -> Result<BitStream, SendError> {
        let bit_len = usize::try_from(self.raw.number_of_bits_used)
            .map_err(|_| SendError::NativeCallFailed)?;
        if bit_len > self.data.len().saturating_mul(u8::BITS as usize) {
            return Err(SendError::NativeCallFailed);
        }
        if bit_len != 0 && self.raw.data != self.data.as_mut_ptr() {
            return Err(SendError::NativeCallFailed);
        }
        let bytes = self.data[..bit_len.div_ceil(u8::BITS as usize)].to_vec();
        BitStream::from_bytes_with_bits(bytes, bit_len).map_err(|_| SendError::NativeCallFailed)
    }

    pub(super) fn as_mut_ptr(&mut self) -> *mut RawBitStream {
        self.raw.data = if self.data.is_empty() {
            self.raw.stack_data.as_mut_ptr()
        } else {
            self.data.as_mut_ptr()
        };
        &mut self.raw
    }
}

pub(super) fn native_bit_length(bit_len: usize) -> Result<i32, SendError> {
    i32::try_from(bit_len).map_err(|_| SendError::PayloadTooLarge)
}

impl BackendState {
    pub(super) fn encode_string(&self, value: &[u8]) -> Result<BitStream, CodecError> {
        if value.contains(&0) {
            return Err(CodecError::InvalidArgument);
        }
        let max_chars = value
            .len()
            .checked_add(1)
            .and_then(|length| i32::try_from(length).ok())
            .ok_or(CodecError::PayloadTooLarge)?;
        let capacity_bits = value
            .len()
            .checked_mul(16)
            .and_then(|bits| bits.checked_add(16))
            .ok_or(CodecError::PayloadTooLarge)?;
        let mut input = Vec::with_capacity(value.len() + 1);
        input.extend_from_slice(value);
        input.push(0);
        let mut native = NativeBitStream::empty_with_capacity_bits(capacity_bits)
            .map_err(|_| CodecError::PayloadTooLarge)?;
        let _codec = self
            .string_codec
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let compressor = self.ready_string_compressor()?;
        let encode: StringWriteEncoderFn = unsafe {
            mem::transmute(self.module_base + self.addresses.string_write_encoder as usize)
        };
        unsafe {
            encode(
                compressor,
                input.as_ptr().cast(),
                max_chars,
                native.as_mut_ptr(),
                0,
            )
        };
        native
            .into_stream()
            .map_err(|_| CodecError::NativeCallFailed)
    }

    pub(super) fn decode_string(
        &self,
        payload: &mut BitStream,
        output: &mut [u8],
    ) -> Result<usize, CodecError> {
        let max_chars = i32::try_from(output.len()).map_err(|_| CodecError::PayloadTooLarge)?;
        if max_chars == 0 {
            return Err(CodecError::InvalidArgument);
        }
        output.fill(0);
        let mut native = NativeBitStream::from_readable_stream(payload)
            .map_err(|_| CodecError::PayloadTooLarge)?;
        let _codec = self
            .string_codec
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let compressor = self.ready_string_compressor()?;
        let decode: StringReadDecoderFn = unsafe {
            mem::transmute(self.module_base + self.addresses.string_read_decoder as usize)
        };
        if !unsafe {
            decode(
                compressor,
                output.as_mut_ptr().cast(),
                max_chars,
                native.as_mut_ptr(),
                0,
            )
        } {
            return Err(CodecError::NativeCallFailed);
        }
        let read_offset = native.read_offset().ok_or(CodecError::NativeCallFailed)?;
        let length = output
            .iter()
            .position(|byte| *byte == 0)
            .ok_or(CodecError::PayloadTooLarge)?;
        payload
            .set_read_offset_bits(read_offset)
            .map_err(|_| CodecError::NativeCallFailed)?;
        Ok(length)
    }

    pub(super) fn ready_string_compressor(&self) -> Result<*mut c_void, CodecError> {
        let pointer = self.module_base + self.addresses.compressor_ptr as usize;
        let compressor = unsafe { ptr::read_unaligned(pointer as *const *mut c_void) };
        if compressor.is_null() {
            Err(CodecError::ClientNotReady)
        } else {
            Ok(compressor)
        }
    }
}
