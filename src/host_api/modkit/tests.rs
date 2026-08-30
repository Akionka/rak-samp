//! Tests for the modkit bootstrap and service discovery.

use super::*;
use modkit_abi::{
    MOD_BUSY, MOD_CALLBACK_IN_PROGRESS, MOD_NATIVE_CALL_FAILED, MOD_PENDING, MOD_QUEUE_FULL,
    MOD_TIMED_OUT, MOD_WAIT_REJECTED,
};

#[test]
fn get_api_writes_the_bootstrap_table_and_returns_ok() {
    let mut out: *const ModHostApiV1 = ptr::null();
    let result = unsafe { GtaModHost_GetApiV1(&mut out) };
    assert_eq!(result, MOD_OK);
    let api = unsafe { out.as_ref() }.expect("bootstrap table is non-null");
    assert_eq!(api.abi_version, MOD_HOST_ABI_VERSION_V1);
    assert_eq!(api.size, std::mem::size_of::<ModHostApiV1>() as u32);
}

#[test]
fn get_api_rejects_null_output() {
    assert_eq!(
        unsafe { GtaModHost_GetApiV1(ptr::null_mut()) },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn query_service_clears_output_before_lookup() {
    let mut out: *const ServiceHeader = ptr::null();
    let result = unsafe { query_service(modkit_abi::ServiceId(0xFFFF_FFFF), 1, &mut out) };
    assert_eq!(result, MOD_NOT_FOUND);
    assert!(out.is_null());
}

#[test]
fn query_service_rejects_null_output() {
    assert_eq!(
        unsafe { query_service(SERVICE_ID_CORE, 1, ptr::null_mut()) },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn query_service_returns_core_for_exact_version() {
    let mut out: *const ServiceHeader = ptr::null();
    let result = unsafe { query_service(SERVICE_ID_CORE, 1, &mut out) };
    assert_eq!(result, MOD_OK);
    let header = unsafe { out.as_ref() }.expect("core service is non-null");
    assert_eq!(header.service_id, SERVICE_ID_CORE);
    assert_eq!(header.version, 1);
    assert_eq!(header.size, std::mem::size_of::<CoreServiceV1>() as u32);
    assert_eq!(header.reserved, 0);
}

#[test]
fn query_service_distinguishes_unsupported_version_from_unknown_service() {
    let mut out: *const ServiceHeader = ptr::null();
    assert_eq!(
        unsafe { query_service(SERVICE_ID_CORE, 2, &mut out) },
        MOD_UNSUPPORTED_VERSION
    );
    assert!(out.is_null());

    assert_eq!(
        unsafe { query_service(modkit_abi::ServiceId(0x0000_0002), 1, &mut out) },
        MOD_NOT_FOUND
    );
    assert!(out.is_null());
}

#[test]
fn query_service_returns_legacy_samp_for_exact_version() {
    let mut out: *const ServiceHeader = ptr::null();
    let result = unsafe { query_service(SERVICE_ID_LEGACY_SAMP_ABI, 1, &mut out) };
    assert_eq!(result, MOD_OK);
    let header = unsafe { out.as_ref() }.expect("legacy service is non-null");
    assert_eq!(header.service_id, SERVICE_ID_LEGACY_SAMP_ABI);
    assert_eq!(header.version, 1);
    assert_eq!(
        header.size,
        std::mem::size_of::<LegacySampServiceV1>() as u32
    );
}

#[test]
fn query_service_returns_samp_net_for_exact_version() {
    let mut out: *const ServiceHeader = ptr::null();
    let result = unsafe { query_service(SERVICE_ID_SAMP_NETWORK, 1, &mut out) };
    assert_eq!(result, MOD_OK);
    let header = unsafe { out.as_ref() }.expect("SA-MP network service is non-null");
    assert_eq!(header.service_id, SERVICE_ID_SAMP_NETWORK);
    assert_eq!(header.version, SAMP_NET_SERVICE_VERSION_V1);
    assert_eq!(header.size, std::mem::size_of::<SampNetServiceV1>() as u32);
    assert_eq!(header.reserved, 0);
}

#[test]
fn query_service_returns_samp_for_exact_version() {
    let mut out: *const ServiceHeader = ptr::null();
    let result = unsafe { query_service(SERVICE_ID_SAMP, 1, &mut out) };
    assert_eq!(result, MOD_OK);
    let header = unsafe { out.as_ref() }.expect("SA-MP service is non-null");
    assert_eq!(header.service_id, SERVICE_ID_SAMP);
    assert_eq!(header.version, SAMP_SERVICE_VERSION_V1);
    assert_eq!(header.size, std::mem::size_of::<SampServiceV1>() as u32);
    assert_eq!(header.reserved, 0);
}

#[test]
fn samp_outputs_reject_null_storage() {
    assert_eq!(
        unsafe { samp_server_info(ptr::null_mut()) },
        MOD_INVALID_ARGUMENT
    );
    assert_eq!(
        unsafe { samp_local_player(ptr::null_mut()) },
        MOD_INVALID_ARGUMENT
    );
    assert_eq!(
        unsafe { samp_player_info(0, ptr::null_mut()) },
        MOD_INVALID_ARGUMENT
    );
    assert_eq!(
        unsafe {
            samp_submit_chat_add(
                modkit_abi::SAMP_CHAT_STYLE_INFO,
                ptr::null(),
                0,
                ptr::null(),
                0,
                0,
                0,
                ptr::null_mut(),
            )
        },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn samp_net_event_access_rejects_null_pointers() {
    let mut id = 0;
    assert_eq!(
        unsafe { samp_net_event_id(ptr::null(), &mut id) },
        MOD_INVALID_ARGUMENT
    );
    assert_eq!(
        unsafe { samp_net_event_id(ptr::dangling(), ptr::null_mut()) },
        MOD_INVALID_ARGUMENT
    );
    assert_eq!(
        unsafe { samp_net_event_read_bits(ptr::null_mut(), ptr::null_mut(), 0, 1) },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn legacy_service_wraps_the_existing_api_table() {
    let mut out: *const ServiceHeader = ptr::null();
    assert_eq!(
        unsafe { query_service(SERVICE_ID_LEGACY_SAMP_ABI, 1, &mut out) },
        MOD_OK
    );
    let legacy = unsafe { (out as *const LegacySampServiceV1).as_ref() }
        .expect("legacy service is non-null");
    assert!(!legacy.api.is_null());
    assert_eq!(
        legacy.api,
        (&super::super::SAMP_CLIENT_SDK_API_V1 as *const sdk_abi::SampClientSdkApiV1)
            .cast::<c_void>()
    );
}

#[test]
fn core_host_status_reports_waiting_before_ready() {
    let mut out = HostStatusV1 {
        state: u32::MAX,
        reserved: [u32::MAX; 3],
    };
    assert_eq!(unsafe { core_host_status(&mut out) }, MOD_OK);
    assert_eq!(out.state, HostStatusV1::STATE_WAITING);
    assert_eq!(out.reserved, [0; 3]);
}

#[test]
fn core_host_status_rejects_null_output() {
    assert_eq!(
        unsafe { core_host_status(ptr::null_mut()) },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn core_unregister_rejects_zero_id() {
    assert_eq!(
        unsafe { core_unregister(SubscriptionId(0)) },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn core_receipt_poll_rejects_zero_id_and_null_output() {
    assert_eq!(
        unsafe { core_receipt_poll(CommandReceiptId(0), ptr::null_mut()) },
        MOD_INVALID_ARGUMENT
    );
    let mut out = CommandCompletionV1::default();
    assert_eq!(
        unsafe { core_receipt_poll(CommandReceiptId(0), &mut out) },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn core_receipt_wait_rejects_zero_id_and_null_output() {
    assert_eq!(
        unsafe { core_receipt_wait(CommandReceiptId(0), 0, ptr::null_mut()) },
        MOD_INVALID_ARGUMENT
    );
    let mut out = CommandCompletionV1::default();
    assert_eq!(
        unsafe { core_receipt_wait(CommandReceiptId(0), 0, &mut out) },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn core_receipt_release_rejects_zero_id() {
    assert_eq!(
        unsafe { core_receipt_release(CommandReceiptId(0)) },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn core_log_utf8_rejects_null_with_nonzero_len() {
    assert_eq!(
        unsafe { core_log_utf8(2, ptr::null(), 1) },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn core_log_utf8_rejects_unknown_level() {
    let message = b"test";
    assert_eq!(
        unsafe { core_log_utf8(99, message.as_ptr(), message.len() as u32) },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn core_log_utf8_accepts_empty_message() {
    assert_eq!(unsafe { core_log_utf8(2, ptr::null(), 0) }, MOD_OK);
}

#[test]
fn core_log_utf8_rejects_unbounded_messages_before_reading_them() {
    assert_eq!(
        unsafe {
            core_log_utf8(
                2,
                std::ptr::dangling(),
                modkit_abi::MAX_LOG_MESSAGE_BYTES + 1,
            )
        },
        MOD_INVALID_ARGUMENT
    );
}

#[test]
fn timeout_sentinel_maps_to_an_unbounded_duration() {
    assert_eq!(
        timeout_duration(modkit_abi::TIMEOUT_INFINITE),
        Duration::MAX
    );
    assert_eq!(timeout_duration(0), Duration::ZERO);
    assert_eq!(timeout_duration(25), Duration::from_millis(25));
}

#[test]
fn command_error_mapping_is_stable() {
    assert_eq!(
        command_error_result(CommandError::QueueFull),
        MOD_QUEUE_FULL
    );
    assert_eq!(command_error_result(CommandError::IdExhausted), MOD_BUSY);
    assert_eq!(
        command_error_result(CommandError::ShuttingDown),
        MOD_SHUTTING_DOWN
    );
    assert_eq!(
        command_error_result(CommandError::NativeFailure),
        MOD_NATIVE_CALL_FAILED
    );
    assert_eq!(
        command_error_result(CommandError::UnknownReceipt),
        MOD_INVALID_ARGUMENT
    );
    assert_eq!(command_error_result(CommandError::TimedOut), MOD_TIMED_OUT);
    assert_eq!(
        command_error_result(CommandError::WaitRejected),
        MOD_WAIT_REJECTED
    );
}

#[test]
fn completion_maps_ok_to_default() {
    let completion = completion(Ok(()));
    assert!(completion.status.is_ok());
    assert_eq!(completion.reserved, 0);
    assert_eq!(completion.value0, 0);
    assert_eq!(completion.value1, 0);
}

#[test]
fn completion_maps_error_to_status() {
    let completion = completion(Err(CommandError::QueueFull));
    assert_eq!(completion.status, MOD_QUEUE_FULL);
}

#[test]
fn shutdown_marks_discovery_as_shutting_down() {
    begin_shutdown();
    let mut out: *const ServiceHeader = ptr::null();
    assert_eq!(
        unsafe { query_service(SERVICE_ID_CORE, 1, &mut out) },
        MOD_SHUTTING_DOWN
    );
    assert!(out.is_null());
    // Reset for other tests.
    host().shutting_down.store(false, Ordering::Release);
}

#[test]
fn unused_result_constants_are_available() {
    // These constants are part of the published ABI surface; referencing them
    // here guards against accidental removal.
    let _ = (MOD_PENDING, MOD_BUSY, MOD_CALLBACK_IN_PROGRESS);
}
