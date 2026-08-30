use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use samp_protocol::{
    BitRead, BitReader, BitStream, BitWrite, DecodeError, EncodeError, EncodedBits,
    ExactBytesPolicy, IncomingPacket, WireCodec, WireDescriptor, WireReadExt, WireWriteExt,
};

struct CountingAllocator;

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn allocations<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::Relaxed);
    let result = operation();
    COUNTING.store(false, Ordering::Relaxed);
    (result, ALLOCATIONS.load(Ordering::Relaxed))
}

struct ByteValue;

type BytePacket = IncomingPacket<250, ByteValue, ExactBytesPolicy>;

impl WireCodec for ByteValue {
    type Value = u8;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let mut output = [0; 1];
        reader
            .read_left_aligned_bits_into(&mut output, u8::BITS as usize)
            .map_err(DecodeError::Source)?;
        Ok(output[0])
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        writer
            .write_left_aligned_bits(&[*value], u8::BITS as usize)
            .map_err(EncodeError::Source)
    }
}

#[test]
fn built_in_bit_io_avoids_temporary_allocations() {
    let source = [0b1010_0000];
    let (reader, allocation_count) = allocations(|| BitReader::from_bits(&source, 3));
    assert_eq!(allocation_count, 0);
    let mut reader = reader.unwrap();
    let mut output = [0; 1];
    assert_eq!(
        allocations(|| reader.read_left_aligned_bits_into(&mut output, 3)).1,
        0
    );
    assert_eq!(output, [0b1010_0000]);

    let mut writer = BitStream::new();
    writer.write_bit_bool(true).unwrap();
    writer.write_u8(0x7f).unwrap();
    writer.write_u16_le(0x1234).unwrap();
    writer.write_f32_le(1.5).unwrap();
    let (bytes, bit_len) = writer.into_parts();
    let mut reader = BitReader::from_bits(&bytes, bit_len).unwrap();
    let (values, allocation_count) = allocations(|| {
        (
            reader.read_bit_bool(),
            reader.read_u8(),
            reader.read_u16_le(),
            reader.read_f32_le(),
        )
    });
    assert_eq!(allocation_count, 0);
    assert_eq!(values, (Ok(true), Ok(0x7f), Ok(0x1234), Ok(1.5)));

    let mut stream = BitStream::from_bytes([0x34, 0x12]).unwrap();
    let (value, allocation_count) = allocations(|| stream.read_u16());
    assert_eq!(allocation_count, 0);
    assert_eq!(value, Ok(0x1234));

    let source = [1, 2, 3];
    let mut reader = BitReader::from_bytes(&source).unwrap();
    let (bytes, allocation_count) = allocations(|| reader.read_bytes(source.len()));
    assert_eq!(allocation_count, 1);
    assert_eq!(bytes, Ok(source.to_vec()));

    let bits = EncodedBits::from_bits([0xa5], u8::BITS as usize).unwrap();
    let (decoded, allocation_count) = allocations(|| BytePacket::decode_bits(&bits));
    assert_eq!(allocation_count, 0);
    assert_eq!(decoded, Ok(0xa5));

    let (encoded, allocation_count) = allocations(|| BytePacket::encode_bits(&0xa5));
    assert_eq!(allocation_count, 1);
    assert_eq!(encoded.unwrap().as_bytes(), &[0xa5]);
}
