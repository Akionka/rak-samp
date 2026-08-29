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

#[test]
fn subscription_set_retains_each_failed_shutdown_for_retry() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let api = test_support::test_api();
    let mut subscriptions = SubscriptionSet::new();
    subscriptions.push(
        api.on_packet(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed"),
    );
    subscriptions.push(
        api.on_rpc(SampClientSdkDirection::Outgoing, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed"),
    );
    test_support::set_unregister_and_wait_result(SampClientSdkResult::CallbackInProgress);

    let error = subscriptions
        .unregister_and_wait()
        .expect_err("failed callbacks must remain available for retry");
    assert_eq!(error.failures().len(), 2);
    assert!(
        error
            .failures()
            .iter()
            .all(|failure| failure.result() == SampClientSdkResult::CallbackInProgress)
    );
    assert_eq!(test_support::registration_stats().registered_callbacks, 2);

    test_support::set_unregister_and_wait_result(SampClientSdkResult::Ok);
    error
        .into_subscriptions()
        .unregister_and_wait()
        .expect("retry must synchronize every callback");
    let stats = test_support::registration_stats();
    assert_eq!(stats.unregister_and_wait_calls, 4);
    assert_eq!(stats.registered_callbacks, 0);
}

#[test]
fn subscription_set_preserves_earlier_registrations_after_a_registration_failure() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let subscription = test_support::test_api()
        .on_packet(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");

    let error = SubscriptionSet::new()
        .try_add(Ok(subscription))
        .and_then(|subscriptions| subscriptions.try_add(Err(SampClientSdkResult::NotReady)))
        .expect_err("the synthetic second registration must fail");
    assert_eq!(error.result(), SampClientSdkResult::NotReady);
    let subscriptions = error.into_subscriptions();
    assert_eq!(subscriptions.len(), 1);
    subscriptions
        .unregister_and_wait()
        .expect("retained subscription must remain cleanly removable");
}

#[test]
fn failed_synchronized_shutdown_keeps_the_subscription_for_retry() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let subscription = test_support::test_api()
        .on_rpc(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");
    test_support::set_unregister_and_wait_result(SampClientSdkResult::CallbackInProgress);

    let error = subscription
        .unregister_and_wait()
        .expect_err("callback-thread shutdown must remain retryable");
    assert_eq!(error.result(), SampClientSdkResult::CallbackInProgress);
    let subscription = error.into_subscription();
    test_support::set_unregister_and_wait_result(SampClientSdkResult::Ok);
    subscription
        .unregister_and_wait()
        .expect("retry must synchronize");
}

#[test]
fn failed_registration_releases_the_handler() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    test_support::set_register_result(SampClientSdkResult::NotReady);
    let drops = Arc::new(AtomicUsize::new(0));
    let counter = DropCounter(Arc::clone(&drops));

    let result = test_support::test_api().on_packet(SampClientSdkDirection::Incoming, move |_| {
        let _ = &counter;
        SampClientSdkHookAction::Continue
    });
    assert_eq!(result.unwrap_err(), SampClientSdkResult::NotReady);
    assert_eq!(drops.load(Ordering::Acquire), 1);
}

#[test]
fn dropping_a_subscription_detaches_without_freeing_callback_state() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let drops = Arc::new(AtomicUsize::new(0));
    let counter = DropCounter(Arc::clone(&drops));
    let subscription = test_support::test_api()
        .on_packet(SampClientSdkDirection::Incoming, move |_| {
            let _ = &counter;
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");

    drop(subscription);
    assert_eq!(drops.load(Ordering::Acquire), 0);
    assert_eq!(test_support::invoke_registered_callback(1), None);
    assert_eq!(test_support::registration_stats().unregister_calls, 1);
}
