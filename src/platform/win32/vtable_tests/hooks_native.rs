//! Native hook and emulation lifecycle tests.

use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

const FAKE_VTABLE_SLOTS: usize = 55;
static ORIGINAL_PACKET_CALLED: AtomicBool = AtomicBool::new(false);

#[repr(C)]
struct FakeClient {
    vtable: *mut usize,
}

unsafe extern "C" fn fake_method() {}
unsafe extern "C" fn later_method() {}
unsafe extern "thiscall" fn fake_outgoing_packet(
    _client: *mut c_void,
    _stream: *mut RawBitStream,
    _priority: i32,
    _reliability: i32,
    _channel: i8,
) -> bool {
    ORIGINAL_PACKET_CALLED.store(true, Ordering::Release);
    true
}

#[test]
fn incoming_emulation_readiness_requires_the_receiver_and_rpc_trampoline() {
    let state = test_backend_state();
    assert!(!state.incoming_emulation_ready());

    state.rpc_receiver.store(1, Ordering::Release);
    assert!(!state.incoming_emulation_ready());

    state.incoming_rpc_trampoline.store(1, Ordering::Release);
    assert!(state.incoming_emulation_ready());
}

#[test]
fn patches_only_owned_slots_and_preserves_a_later_hook() {
    let original = fake_method as *const () as usize;
    let mut table = vec![original; FAKE_VTABLE_SLOTS].into_boxed_slice();
    let untouched_slot = FAKE_VTABLE_SLOTS - 1;
    let untouched_original = table[untouched_slot];
    let mut client = FakeClient {
        vtable: table.as_mut_ptr(),
    };

    let (hook, originals) =
        unsafe { VtableHook::install((&mut client as *mut FakeClient).cast::<c_void>()).unwrap() };

    assert_eq!(
        table[OUTGOING_PACKET_SLOT],
        samp_native::hooks::outgoing_packet_detour as *const () as usize
    );
    assert_eq!(
        table[INCOMING_PACKET_SLOT],
        samp_native::hooks::incoming_packet_detour as *const () as usize
    );
    assert_eq!(
        table[OUTGOING_RPC_SLOT],
        samp_native::hooks::outgoing_rpc_detour as *const () as usize
    );
    assert_eq!(table[untouched_slot], untouched_original);
    assert_eq!(originals.outgoing_packet, original);

    let later_hook = later_method as *const () as usize;
    table[OUTGOING_PACKET_SLOT] = later_hook;
    drop(hook);

    assert_eq!(table[OUTGOING_PACKET_SLOT], later_hook);
    assert_eq!(table[INCOMING_PACKET_SLOT], original);
    assert_eq!(table[OUTGOING_RPC_SLOT], original);
    assert_eq!(table[untouched_slot], untouched_original);
}

#[test]
fn captured_state_calls_original_after_active_slot_is_cleared() {
    ORIGINAL_PACKET_CALLED.store(false, Ordering::Release);
    let state = Arc::new(test_backend_state());
    state.outgoing_packet_original.store(
        fake_outgoing_packet as *const () as usize,
        Ordering::Release,
    );
    let active = ACTIVE_BACKEND.get_or_init(|| Mutex::new(None));
    *active.lock().unwrap_or_else(|error| error.into_inner()) = Some(Arc::downgrade(&state));

    let captured = Arc::clone(&state);
    clear_active_backend(&state);
    assert!(active_state().is_none());
    assert!(hooks::call_outgoing_packet(
        &captured,
        ptr::null_mut(),
        ptr::null_mut(),
        0,
        0,
        0,
    ));
    assert!(ORIGINAL_PACKET_CALLED.load(Ordering::Acquire));
}

#[test]
fn packet_emulation_requires_the_captured_rpc_receiver() {
    let state = test_backend_state();
    state.rak_client.store(0x1000, Ordering::Release);

    assert_eq!(state.ready_rpc_receiver(), Err(SendError::ClientNotReady));

    state.rpc_receiver.store(0x2000, Ordering::Release);
    assert_eq!(
        state.ready_rpc_receiver().map(|receiver| receiver as usize),
        Ok(0x2000)
    );
}

#[test]
fn incoming_rpc_emulation_blocks_before_native_readiness_checks() {
    let state = test_backend_state();
    let _listener = state.registry.register_rpc(Direction::Incoming, |event| {
        assert_eq!(event.id(), 42);
        HookAction::Block
    });

    assert_eq!(
        state.emulate_incoming_rpc_native(42, BitStream::new()),
        Ok(false)
    );
}

#[test]
fn client_hook_failure_is_observable_by_the_runtime() {
    let state = Arc::new(test_backend_state());
    let backend = Backend {
        state: Arc::clone(&state),
    };

    assert_eq!(backend.client_hook_status(), ClientHookStatus::Pending);
    state
        .client_hook_status
        .store(ClientHookInstallState::Failed.as_raw(), Ordering::Release);
    assert_eq!(backend.client_hook_status(), ClientHookStatus::Failed);
}
