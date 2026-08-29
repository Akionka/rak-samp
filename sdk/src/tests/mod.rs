mod abi;
mod callbacks;
mod host_api;
mod protocol;
mod subscriptions;

use super::*;
use crate::events::{ProtocolAction, test_support};
use samp_protocol::EncodedBits;
use samp_protocol::rpc::incoming::{common as protocol_common, r1 as protocol_r1};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

static REGISTRATION_TEST_LOCK: Mutex<()> = Mutex::new(());

struct FailingReplacementCodec;

impl samp_protocol::WireCodec for FailingReplacementCodec {
    type Value = bool;

    fn decode<R: samp_protocol::BitRead>(
        reader: &mut R,
    ) -> Result<Self::Value, samp_protocol::DecodeError<R::Error>> {
        reader
            .read_left_aligned_bits(8)
            .map(|bits| bits[0] != 0)
            .map_err(samp_protocol::DecodeError::Source)
    }

    fn encode<W: samp_protocol::BitWrite>(
        _writer: &mut W,
        _value: &Self::Value,
    ) -> Result<(), samp_protocol::EncodeError<W::Error>> {
        Err(samp_protocol::EncodeError::LengthExceedsLimit {
            length: 2,
            limit: 1,
        })
    }
}

type FailingIncomingRpc =
    samp_protocol::IncomingRpc<201, FailingReplacementCodec, samp_protocol::ExactBitsPolicy>;

struct ThreeBitCodec;

impl samp_protocol::WireCodec for ThreeBitCodec {
    type Value = u8;

    fn decode<R: samp_protocol::BitRead>(
        reader: &mut R,
    ) -> Result<Self::Value, samp_protocol::DecodeError<R::Error>> {
        let bytes = reader
            .read_left_aligned_bits(3)
            .map_err(samp_protocol::DecodeError::Source)?;
        Ok(bytes[0] >> 5)
    }

    fn encode<W: samp_protocol::BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), samp_protocol::EncodeError<W::Error>> {
        writer
            .write_left_aligned_bits(&[*value << 5], 3)
            .map_err(samp_protocol::EncodeError::Source)
    }
}

type NonByteAlignedOutgoingRpc =
    samp_protocol::OutgoingRpc<201, ThreeBitCodec, samp_protocol::ExactBytesPolicy>;

struct ByteCodec;

impl samp_protocol::WireCodec for ByteCodec {
    type Value = u8;

    fn decode<R: samp_protocol::BitRead>(
        reader: &mut R,
    ) -> Result<Self::Value, samp_protocol::DecodeError<R::Error>> {
        reader
            .read_left_aligned_bits(8)
            .map(|bits| bits[0])
            .map_err(samp_protocol::DecodeError::Source)
    }

    fn encode<W: samp_protocol::BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), samp_protocol::EncodeError<W::Error>> {
        writer
            .write_left_aligned_bits(&[*value], 8)
            .map_err(samp_protocol::EncodeError::Source)
    }
}

type CustomOutgoingPacket =
    samp_protocol::OutgoingPacket<204, ByteCodec, samp_protocol::ExactBitsPolicy>;

struct DropCounter(Arc<AtomicUsize>);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Release);
    }
}

fn assert_default_is_zeroed<T: Default>() {
    let value = T::default();
    let bytes = unsafe {
        core::slice::from_raw_parts((&value as *const T).cast::<u8>(), core::mem::size_of::<T>())
    };
    assert!(bytes.iter().all(|byte| *byte == 0));
}
