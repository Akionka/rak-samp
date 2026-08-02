//! Private Windows x86 hook implementation.
//!
//! The module is intentionally unavailable for other targets. Its only public
//! boundary is the safe `Runtime` API in the parent crate.

use crate::{
    AddressSet, AttachError, BitStream, Direction, SampVersion, SendError, SendOptions,
    event::{HookAction, Registry},
    runtime::{ClientHookStatus, PacketPriority, PacketReliability},
};
use minhook::MinHook;
use std::{
    ffi::c_void,
    mem, ptr, slice,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicUsize, Ordering},
    },
};
use windows_sys::Win32::System::{
    LibraryLoader::GetModuleHandleA,
    Memory::{PAGE_READWRITE, VirtualProtect},
};

const ID_TIMESTAMP: u8 = 40;
const ID_RPC: u8 = 20;
const OUTGOING_PACKET_SLOT: usize = 6;
const INCOMING_PACKET_SLOT: usize = 8;
const DEALLOCATE_PACKET_SLOT: usize = 9;
const OUTGOING_RPC_SLOT: usize = 25;
const PEER_PACKET_QUEUE_OFFSET: usize = 0xDB6;
const MAX_INCOMING_PACKET_BYTES: usize = 16 * 1024 * 1024;
const CLIENT_HOOK_PENDING: u32 = 0;
const CLIENT_HOOK_READY: u32 = 1;
const CLIENT_HOOK_FAILED: u32 = 2;

pub(crate) struct Backend {
    state: Arc<BackendState>,
}

struct BackendState {
    registry: Arc<Registry>,
    module_base: usize,
    addresses: AddressSet,
    rak_client: AtomicUsize,
    rpc_receiver: AtomicUsize,
    player_address: AtomicU32,
    player_port: AtomicU16,
    constructor_trampoline: AtomicUsize,
    incoming_rpc_trampoline: AtomicUsize,
    outgoing_packet_original: AtomicUsize,
    incoming_packet_original: AtomicUsize,
    deallocate_packet_original: AtomicUsize,
    outgoing_rpc_original: AtomicUsize,
    client_hook_status: AtomicU32,
    incoming_packet_diagnostic_logged: AtomicBool,
    hooks: Mutex<HookStorage>,
}

#[derive(Default)]
struct HookStorage {
    constructor: Option<InlineHook>,
    incoming_rpc: Option<InlineHook>,
    vtable: Option<VtableHook>,
}

static ACTIVE_BACKEND: OnceLock<Mutex<Option<Weak<BackendState>>>> = OnceLock::new();

pub(crate) fn attach(registry: Arc<Registry>) -> Result<Backend, AttachError> {
    let module_base = loaded_samp_module()?;
    let entry_point = unsafe { pe_entry_point(module_base)? };
    let version = SampVersion::from_entry_point(entry_point)
        .ok_or(AttachError::UnsupportedClient { entry_point })?;
    let addresses = AddressSet::for_version(version);

    let active = ACTIVE_BACKEND.get_or_init(|| Mutex::new(None));
    let mut active = active.lock().unwrap_or_else(|error| error.into_inner());
    if active.as_ref().and_then(Weak::upgrade).is_some() {
        return Err(AttachError::AlreadyAttached);
    }

    let state = Arc::new(BackendState {
        registry,
        module_base,
        addresses,
        rak_client: AtomicUsize::new(0),
        rpc_receiver: AtomicUsize::new(0),
        player_address: AtomicU32::new(0),
        player_port: AtomicU16::new(0),
        constructor_trampoline: AtomicUsize::new(0),
        incoming_rpc_trampoline: AtomicUsize::new(0),
        outgoing_packet_original: AtomicUsize::new(0),
        incoming_packet_original: AtomicUsize::new(0),
        deallocate_packet_original: AtomicUsize::new(0),
        outgoing_rpc_original: AtomicUsize::new(0),
        client_hook_status: AtomicU32::new(CLIENT_HOOK_PENDING),
        incoming_packet_diagnostic_logged: AtomicBool::new(false),
        hooks: Mutex::new(HookStorage::default()),
    });
    *active = Some(Arc::downgrade(&state));
    drop(active);

    if let Err(error) = state.install_constructor_hook() {
        clear_active_backend(&state);
        return Err(error);
    }
    Ok(Backend { state })
}

impl Backend {
    pub(crate) fn client_hook_status(&self) -> ClientHookStatus {
        match self.state.client_hook_status.load(Ordering::Acquire) {
            CLIENT_HOOK_READY => ClientHookStatus::Ready,
            CLIENT_HOOK_FAILED => ClientHookStatus::Failed,
            _ => ClientHookStatus::Pending,
        }
    }

    pub(crate) fn send_packet(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.state.send_packet(packet_id, payload, options)
    }

    pub(crate) fn send_rpc(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.state.send_rpc(rpc_id, payload, options)
    }

    pub(crate) fn emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.state.emulate_incoming_packet(packet_id, payload)
    }

    pub(crate) fn emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.state.emulate_incoming_rpc(rpc_id, payload)
    }

    pub(crate) fn shutdown(&mut self) {
        self.state.shutdown();
    }
}

impl BackendState {
    fn install_constructor_hook(self: &Arc<Self>) -> Result<(), AttachError> {
        let target = self.module_base + self.addresses.rak_client_constructor as usize;
        let (mut detour, trampoline) =
            InlineHook::create(target, rak_client_constructor_detour as *const () as usize)
                .map_err(|_| AttachError::HookInstallFailed("RakClient constructor detour"))?;
        self.constructor_trampoline
            .store(trampoline, Ordering::Release);
        if detour.enable().is_err() {
            self.constructor_trampoline.store(0, Ordering::Release);
            return Err(AttachError::HookInstallFailed(
                "enabling RakClient constructor detour",
            ));
        }
        self.hooks
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .constructor = Some(detour);
        Ok(())
    }

    fn install_client_hooks(&self, client: *mut c_void) -> Result<(), AttachError> {
        if client.is_null() {
            return Err(AttachError::ClientNotReady);
        }
        if self
            .rak_client
            .compare_exchange(0, client as usize, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(());
        }

        let incoming_target = self.module_base + self.addresses.incoming_rpc_handler as usize;
        let (mut incoming_rpc, trampoline) =
            InlineHook::create(incoming_target, incoming_rpc_detour as *const () as usize)
                .map_err(|_| {
                    self.rak_client.store(0, Ordering::Release);
                    AttachError::HookInstallFailed("incoming RPC detour")
                })?;
        self.incoming_rpc_trampoline
            .store(trampoline, Ordering::Release);
        if incoming_rpc.enable().is_err() {
            self.incoming_rpc_trampoline.store(0, Ordering::Release);
            self.rak_client.store(0, Ordering::Release);
            return Err(AttachError::HookInstallFailed(
                "enabling incoming RPC detour",
            ));
        }

        let vtable = match unsafe { VtableHook::install(client, self) } {
            Ok(vtable) => vtable,
            Err(error) => {
                incoming_rpc.disable();
                self.incoming_rpc_trampoline.store(0, Ordering::Release);
                self.rak_client.store(0, Ordering::Release);
                self.outgoing_packet_original.store(0, Ordering::Release);
                self.incoming_packet_original.store(0, Ordering::Release);
                self.deallocate_packet_original.store(0, Ordering::Release);
                self.outgoing_rpc_original.store(0, Ordering::Release);
                return Err(error);
            }
        };
        let mut hooks = self.hooks.lock().unwrap_or_else(|error| error.into_inner());
        hooks.incoming_rpc = Some(incoming_rpc);
        hooks.vtable = Some(vtable);
        self.client_hook_status
            .store(CLIENT_HOOK_READY, Ordering::Release);
        Ok(())
    }

    fn send_packet(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        let client = self.ready_client()?;
        let original = self.outgoing_packet_original.load(Ordering::Acquire);
        if original == 0 {
            return Err(SendError::ClientNotReady);
        }
        let stream = packet_stream(packet_id, payload)?;
        let mut native = NativeBitStream::new(&stream)?;
        let send: OutgoingPacketFn = unsafe { mem::transmute(original) };
        Ok(unsafe {
            send(
                client,
                native.as_mut_ptr(),
                priority_value(options.priority),
                reliability_value(options.reliability),
                options.ordering_channel as i8,
            )
        })
    }

    fn send_rpc(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        let client = self.ready_client()?;
        let original = self.outgoing_rpc_original.load(Ordering::Acquire);
        if original == 0 {
            return Err(SendError::ClientNotReady);
        }
        let mut native = NativeBitStream::new(payload)?;
        let send: OutgoingRpcFn = unsafe { mem::transmute(original) };
        let mut id = i32::from(rpc_id);
        Ok(unsafe {
            send(
                client,
                &mut id,
                native.as_mut_ptr(),
                priority_value(options.priority),
                reliability_value(options.reliability),
                options.ordering_channel as i8,
                options.timestamp,
            )
        })
    }

    fn emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        let peer = self.ready_rpc_receiver()?;
        let stream = packet_stream(packet_id, &payload)?;
        let byte_len = i32::try_from(stream.len_bytes()).map_err(|_| SendError::PayloadTooLarge)?;
        let bit_size = u32::try_from(stream.len_bits()).map_err(|_| SendError::PayloadTooLarge)?;
        let allocate: AllocatePacketFn =
            unsafe { mem::transmute(self.module_base + self.addresses.allocate_packet as usize) };
        let packet = unsafe { allocate(byte_len) };
        if packet.is_null() {
            return Err(SendError::NativeCallFailed);
        }
        unsafe {
            let packet_data = ptr::addr_of!((*packet).data).read_unaligned();
            if packet_data.is_null() {
                return Err(SendError::NativeCallFailed);
            }
            ptr::copy_nonoverlapping(stream.as_bytes().as_ptr(), packet_data, stream.len_bytes());
            ptr::addr_of_mut!((*packet).length).write_unaligned(stream.len_bytes() as u32);
            ptr::addr_of_mut!((*packet).bit_size).write_unaligned(bit_size);

            let lock: QueueWriteLockFn =
                mem::transmute(self.module_base + self.addresses.write_lock as usize);
            let unlock: QueueWriteUnlockFn =
                mem::transmute(self.module_base + self.addresses.write_unlock as usize);
            let slot = lock(peer.add(PEER_PACKET_QUEUE_OFFSET).cast());
            if slot.is_null() {
                return Err(SendError::NativeCallFailed);
            }
            *slot = packet;
            unlock(peer.add(PEER_PACKET_QUEUE_OFFSET).cast());
        }
        Ok(true)
    }

    fn emulate_incoming_rpc(&self, rpc_id: u8, mut payload: BitStream) -> Result<bool, SendError> {
        if self
            .registry
            .dispatch_rpc(Direction::Incoming, rpc_id, &mut payload)
            == HookAction::Block
        {
            return Ok(false);
        }
        let receiver = self.rpc_receiver.load(Ordering::Acquire) as *mut c_void;
        let original = self.incoming_rpc_trampoline.load(Ordering::Acquire);
        if receiver.is_null() || original == 0 {
            return Err(SendError::ClientNotReady);
        }
        let mut envelope = BitStream::new();
        envelope
            .write_u8(ID_RPC)
            .map_err(|_| SendError::PayloadTooLarge)?;
        envelope
            .write_u8(rpc_id)
            .map_err(|_| SendError::PayloadTooLarge)?;
        envelope
            .write_compressed_u32(payload.len_bits() as u32)
            .map_err(|_| SendError::PayloadTooLarge)?;
        envelope
            .write_stream(&payload)
            .map_err(|_| SendError::PayloadTooLarge)?;
        let original: IncomingRpcFn = unsafe { mem::transmute(original) };
        let envelope_len =
            i32::try_from(envelope.len_bytes()).map_err(|_| SendError::PayloadTooLarge)?;
        let player = RpcPlayerId {
            binary_address: self.player_address.load(Ordering::Acquire),
            port: self.player_port.load(Ordering::Acquire),
        };
        Ok(unsafe {
            original(
                receiver,
                envelope.as_bytes().as_ptr().cast_mut(),
                envelope_len,
                player,
            )
        })
    }

    fn ready_client(&self) -> Result<*mut c_void, SendError> {
        let client = self.rak_client.load(Ordering::Acquire) as *mut c_void;
        if client.is_null() {
            Err(SendError::ClientNotReady)
        } else {
            Ok(client)
        }
    }

    fn ready_rpc_receiver(&self) -> Result<*mut c_void, SendError> {
        let receiver = self.rpc_receiver.load(Ordering::Acquire) as *mut c_void;
        if receiver.is_null() {
            Err(SendError::ClientNotReady)
        } else {
            Ok(receiver)
        }
    }

    fn shutdown(&self) {
        let mut hooks = self.hooks.lock().unwrap_or_else(|error| error.into_inner());
        hooks.vtable.take();
        if let Some(detour) = hooks.incoming_rpc.take() {
            detour.disable();
        }
        if let Some(detour) = hooks.constructor.take() {
            detour.disable();
        }
        drop(hooks);

        // No new native calls can enter our detours after the vtable and inline
        // hooks have been removed. Existing detour calls hold an Arc from
        // active_state and can still reach their original functions safely.
        clear_active_backend(self);
        self.rak_client.store(0, Ordering::Release);
    }
}

#[repr(C)]
struct RawBitStream {
    number_of_bits_used: i32,
    number_of_bits_allocated: i32,
    read_offset: i32,
    data: *mut u8,
    copy_data: bool,
    stack_data: [u8; 256],
}

impl RawBitStream {
    unsafe fn copy_to_owned(&self) -> Result<BitStream, SendError> {
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

    unsafe fn replace_from(&mut self, stream: &BitStream) -> Result<(), SendError> {
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

struct NativeBitStream {
    data: Vec<u8>,
    raw: RawBitStream,
}

impl NativeBitStream {
    fn new(stream: &BitStream) -> Result<Self, SendError> {
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

    fn as_mut_ptr(&mut self) -> *mut RawBitStream {
        self.raw.data = if self.data.is_empty() {
            self.raw.stack_data.as_mut_ptr()
        } else {
            self.data.as_mut_ptr()
        };
        &mut self.raw
    }
}

fn native_bit_length(bit_len: usize) -> Result<i32, SendError> {
    i32::try_from(bit_len).map_err(|_| SendError::PayloadTooLarge)
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RpcPlayerId {
    binary_address: u32,
    port: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PacketPlayerId {
    binary_address: u32,
    port: u16,
}

#[repr(C, packed)]
struct RawPacket {
    player_index: u16,
    player_id: PacketPlayerId,
    length: u32,
    bit_size: u32,
    data: *mut u8,
    delete_data: bool,
}

#[cfg(test)]
mod layout_tests {
    use super::{PacketPlayerId, RawPacket};
    use std::mem::{MaybeUninit, align_of, offset_of, size_of};
    use std::ptr;

    unsafe extern "C" {
        fn rak_rs_fixture_player_id_size() -> usize;
        fn rak_rs_fixture_player_id_alignment() -> usize;
        fn rak_rs_fixture_packet_size() -> usize;
        fn rak_rs_fixture_packet_alignment() -> usize;
        fn rak_rs_fixture_packet_player_index_offset() -> usize;
        fn rak_rs_fixture_packet_player_id_offset() -> usize;
        fn rak_rs_fixture_packet_length_offset() -> usize;
        fn rak_rs_fixture_packet_bit_size_offset() -> usize;
        fn rak_rs_fixture_packet_data_offset() -> usize;
        fn rak_rs_fixture_packet_delete_data_offset() -> usize;
        fn rak_rs_fixture_initialize_packet(memory: *mut RawPacket, data: *mut u8);
    }

    #[test]
    fn raknet_packet_layout_matches_the_cpp_x86_abi() {
        unsafe {
            assert_eq!(size_of::<PacketPlayerId>(), rak_rs_fixture_player_id_size());
            assert_eq!(
                align_of::<PacketPlayerId>(),
                rak_rs_fixture_player_id_alignment()
            );

            assert_eq!(size_of::<RawPacket>(), rak_rs_fixture_packet_size());
            assert_eq!(align_of::<RawPacket>(), rak_rs_fixture_packet_alignment());
            assert_eq!(
                offset_of!(RawPacket, player_index),
                rak_rs_fixture_packet_player_index_offset()
            );
            assert_eq!(
                offset_of!(RawPacket, player_id),
                rak_rs_fixture_packet_player_id_offset()
            );
            assert_eq!(
                offset_of!(RawPacket, length),
                rak_rs_fixture_packet_length_offset()
            );
            assert_eq!(
                offset_of!(RawPacket, bit_size),
                rak_rs_fixture_packet_bit_size_offset()
            );
            assert_eq!(
                offset_of!(RawPacket, data),
                rak_rs_fixture_packet_data_offset()
            );
            assert_eq!(
                offset_of!(RawPacket, delete_data),
                rak_rs_fixture_packet_delete_data_offset()
            );
        }
    }

    #[test]
    fn reads_a_packet_initialized_by_cpp() {
        let mut data = [0xAA, 0xBB, 0xCC];
        let mut packet = MaybeUninit::<RawPacket>::uninit();
        unsafe {
            rak_rs_fixture_initialize_packet(packet.as_mut_ptr(), data.as_mut_ptr());
            let packet = packet.assume_init();
            assert_eq!(ptr::addr_of!(packet.player_index).read_unaligned(), 0x1234);
            assert_eq!(
                ptr::addr_of!(packet.player_id.binary_address).read_unaligned(),
                0x01020304
            );
            assert_eq!(
                ptr::addr_of!(packet.player_id.port).read_unaligned(),
                0x5678
            );
            assert_eq!(ptr::addr_of!(packet.length).read_unaligned(), 3);
            assert_eq!(ptr::addr_of!(packet.bit_size).read_unaligned(), 17);
            assert_eq!(
                ptr::addr_of!(packet.data).read_unaligned(),
                data.as_mut_ptr()
            );
            assert!(ptr::addr_of!(packet.delete_data).read_unaligned());
        }
    }
}

#[cfg(test)]
mod vtable_tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

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

    fn test_backend_state() -> BackendState {
        BackendState {
            registry: Registry::new(),
            module_base: 0,
            addresses: AddressSet::for_version(SampVersion::R1),
            rak_client: AtomicUsize::new(0),
            rpc_receiver: AtomicUsize::new(0),
            player_address: AtomicU32::new(0),
            player_port: AtomicU16::new(0),
            constructor_trampoline: AtomicUsize::new(0),
            incoming_rpc_trampoline: AtomicUsize::new(0),
            outgoing_packet_original: AtomicUsize::new(0),
            incoming_packet_original: AtomicUsize::new(0),
            deallocate_packet_original: AtomicUsize::new(0),
            outgoing_rpc_original: AtomicUsize::new(0),
            client_hook_status: AtomicU32::new(CLIENT_HOOK_PENDING),
            incoming_packet_diagnostic_logged: AtomicBool::new(false),
            hooks: Mutex::new(HookStorage::default()),
        }
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
        let state = test_backend_state();

        let hook = unsafe {
            VtableHook::install((&mut client as *mut FakeClient).cast::<c_void>(), &state).unwrap()
        };

        assert_eq!(
            table[OUTGOING_PACKET_SLOT],
            outgoing_packet_detour as *const () as usize
        );
        assert_eq!(
            table[INCOMING_PACKET_SLOT],
            incoming_packet_detour as *const () as usize
        );
        assert_eq!(
            table[OUTGOING_RPC_SLOT],
            outgoing_rpc_detour as *const () as usize
        );
        assert_eq!(table[untouched_slot], untouched_original);
        assert_eq!(
            state.outgoing_packet_original.load(Ordering::Acquire),
            original
        );

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
        assert!(call_outgoing_packet(
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
    fn client_hook_failure_is_observable_by_the_runtime() {
        let state = Arc::new(test_backend_state());
        let backend = Backend {
            state: Arc::clone(&state),
        };

        assert_eq!(backend.client_hook_status(), ClientHookStatus::Pending);
        state
            .client_hook_status
            .store(CLIENT_HOOK_FAILED, Ordering::Release);
        assert_eq!(backend.client_hook_status(), ClientHookStatus::Failed);
    }
}

#[cfg(test)]
mod inline_hook_tests {
    use super::*;
    use std::sync::{
        Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    };

    static TEST_TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[inline(never)]
    unsafe extern "C" fn target(value: i32) -> i32 {
        value + 1
    }

    #[inline(never)]
    unsafe extern "C" fn detour(value: i32) -> i32 {
        let trampoline = TEST_TRAMPOLINE.load(Ordering::Acquire);
        if trampoline == 0 {
            return i32::MIN;
        }
        let original: unsafe extern "C" fn(i32) -> i32 = unsafe { mem::transmute(trampoline) };
        unsafe { original(value) + 10 }
    }

    #[test]
    fn publishes_trampoline_before_enabling_and_can_recreate_inline_hook() {
        let _serial = TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let target = target as *const () as usize;
        let detour = detour as *const () as usize;

        let (mut hook, trampoline) = InlineHook::create(target, detour).unwrap();

        // Creation must leave the target disabled until the caller publishes
        // the trampoline used by the detour.
        assert_eq!(unsafe { self::target(7) }, 8);
        TEST_TRAMPOLINE.store(trampoline, Ordering::Release);
        hook.enable().unwrap();
        assert_eq!(unsafe { self::target(7) }, 18);

        hook.disable();
        assert_eq!(unsafe { self::target(7) }, 8);

        let (recreated, recreated_trampoline) = InlineHook::create(target, detour).unwrap();
        assert_ne!(recreated_trampoline, 0);
        recreated.disable();
        TEST_TRAMPOLINE.store(0, Ordering::Release);
    }
}

struct VtableHook {
    vtable: usize,
    entries: [VtableEntry; 3],
}

#[derive(Clone, Copy)]
struct VtableEntry {
    slot: usize,
    original: usize,
    detour: usize,
}

struct InlineHook {
    target: usize,
    enabled: bool,
}

impl InlineHook {
    fn create(target: usize, detour: usize) -> Result<(Self, usize), ()> {
        let trampoline = unsafe {
            MinHook::create_hook(target as *mut c_void, detour as *mut c_void).map_err(|_| ())?
        };
        Ok((
            Self {
                target,
                enabled: false,
            },
            trampoline as usize,
        ))
    }

    fn enable(&mut self) -> Result<(), ()> {
        unsafe { MinHook::enable_hook(self.target as *mut c_void) }.map_err(|_| ())?;
        self.enabled = true;
        Ok(())
    }

    fn disable(mut self) {
        self.remove();
    }

    fn remove(&mut self) {
        if self.target == 0 {
            return;
        }
        let target = self.target as *mut c_void;
        if self.enabled {
            let _ = unsafe { MinHook::disable_hook(target) };
        }
        let _ = unsafe { MinHook::remove_hook(target) };
        self.target = 0;
        self.enabled = false;
    }
}

impl Drop for InlineHook {
    fn drop(&mut self) {
        self.remove();
    }
}

impl VtableHook {
    unsafe fn install(client: *mut c_void, state: &BackendState) -> Result<Self, AttachError> {
        let object_vtable = client.cast::<*mut usize>();
        let vtable = unsafe { object_vtable.read() };
        if vtable.is_null() {
            return Err(AttachError::ClientNotReady);
        }

        let replacements = [
            (
                OUTGOING_PACKET_SLOT,
                outgoing_packet_detour as *const () as usize,
            ),
            (
                INCOMING_PACKET_SLOT,
                incoming_packet_detour as *const () as usize,
            ),
            (OUTGOING_RPC_SLOT, outgoing_rpc_detour as *const () as usize),
        ];
        let mut entries = [VtableEntry {
            slot: 0,
            original: 0,
            detour: 0,
        }; 3];
        for (index, (slot, detour)) in replacements.into_iter().enumerate() {
            let original = unsafe { vtable.add(slot).read() };
            if original == 0 {
                return Err(AttachError::ClientNotReady);
            }
            entries[index] = VtableEntry {
                slot,
                original,
                detour,
            };
        }

        state
            .outgoing_packet_original
            .store(entries[0].original, Ordering::Release);
        state
            .incoming_packet_original
            .store(entries[1].original, Ordering::Release);
        state.deallocate_packet_original.store(
            unsafe { vtable.add(DEALLOCATE_PACKET_SLOT).read() },
            Ordering::Release,
        );
        state
            .outgoing_rpc_original
            .store(entries[2].original, Ordering::Release);

        for (index, entry) in entries.iter().enumerate() {
            if unsafe { write_protected(vtable.add(entry.slot), entry.detour) }.is_err() {
                for restore in entries[..index].iter().rev() {
                    let _ = unsafe { write_protected(vtable.add(restore.slot), restore.original) };
                }
                return Err(AttachError::HookInstallFailed("patching RakClient vtable"));
            }
        }

        Ok(Self {
            vtable: vtable as usize,
            entries,
        })
    }
}

impl Drop for VtableHook {
    fn drop(&mut self) {
        let vtable = self.vtable as *mut usize;
        for entry in self.entries.iter().rev() {
            let slot = unsafe { vtable.add(entry.slot) };
            if unsafe { slot.read() } == entry.detour {
                let _ = unsafe { write_protected(slot, entry.original) };
            }
        }
    }
}

type RakClientConstructorFn = unsafe extern "C" fn() -> *mut c_void;
type OutgoingPacketFn =
    unsafe extern "thiscall" fn(*mut c_void, *mut RawBitStream, i32, i32, i8) -> bool;
type IncomingPacketFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut RawPacket;
type DeallocatePacketFn = unsafe extern "thiscall" fn(*mut c_void, *mut RawPacket);
type OutgoingRpcFn = unsafe extern "thiscall" fn(
    *mut c_void,
    *mut i32,
    *mut RawBitStream,
    i32,
    i32,
    i8,
    bool,
) -> bool;
type IncomingRpcFn = unsafe extern "thiscall" fn(*mut c_void, *mut u8, i32, RpcPlayerId) -> bool;
type AllocatePacketFn = unsafe extern "C" fn(i32) -> *mut RawPacket;
type QueueWriteLockFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut *mut RawPacket;
type QueueWriteUnlockFn = unsafe extern "thiscall" fn(*mut c_void);

#[derive(Clone, Copy)]
struct OutgoingRpcCall {
    client: *mut c_void,
    id: *mut i32,
    stream: *mut RawBitStream,
    priority: i32,
    reliability: i32,
    channel: i8,
    timestamp: bool,
}

unsafe extern "C" fn rak_client_constructor_detour() -> *mut c_void {
    let Some(state) = active_state() else {
        return ptr::null_mut();
    };
    let trampoline = state.constructor_trampoline.load(Ordering::Acquire);
    if trampoline == 0 {
        return ptr::null_mut();
    }
    let original: RakClientConstructorFn = unsafe { mem::transmute(trampoline) };
    let client = unsafe { original() };
    if !client.is_null()
        && let Err(error) = state.install_client_hooks(client)
    {
        state
            .client_hook_status
            .store(CLIENT_HOOK_FAILED, Ordering::Release);
        log::error!("RakClient hook installation failed: {error}");
    }
    client
}

unsafe extern "thiscall" fn outgoing_packet_detour(
    client: *mut c_void,
    native: *mut RawBitStream,
    priority: i32,
    reliability: i32,
    channel: i8,
) -> bool {
    let Some(state) = active_state() else {
        return false;
    };
    if !state.registry.has_packet_listener(Direction::Outgoing) {
        return call_outgoing_packet(&state, client, native, priority, reliability, channel);
    }
    let action = unsafe { dispatch_packet_stream(&state, Direction::Outgoing, native) };
    if action == HookAction::Block {
        return false;
    }
    call_outgoing_packet(&state, client, native, priority, reliability, channel)
}

unsafe extern "thiscall" fn incoming_packet_detour(client: *mut c_void) -> *mut RawPacket {
    let Some(state) = active_state() else {
        return ptr::null_mut();
    };
    loop {
        let packet = call_incoming_packet(&state, client);
        if packet.is_null() {
            return packet;
        }
        let action = unsafe { dispatch_raw_packet(&state, packet) };
        if action == HookAction::Continue {
            return packet;
        }
        deallocate_packet(&state, client, packet);
    }
}

unsafe extern "thiscall" fn outgoing_rpc_detour(
    client: *mut c_void,
    id: *mut i32,
    native: *mut RawBitStream,
    priority: i32,
    reliability: i32,
    channel: i8,
    timestamp: bool,
) -> bool {
    let Some(state) = active_state() else {
        return false;
    };
    let original_call = OutgoingRpcCall {
        client,
        id,
        stream: native,
        priority,
        reliability,
        channel,
        timestamp,
    };
    if id.is_null() {
        return call_outgoing_rpc(&state, original_call);
    }
    if !state.registry.has_rpc_listener(Direction::Outgoing) {
        return call_outgoing_rpc(&state, original_call);
    }
    let action = unsafe { dispatch_rpc_stream(&state, Direction::Outgoing, *id as u8, native) };
    if action == HookAction::Block {
        return false;
    }
    call_outgoing_rpc(&state, original_call)
}

unsafe extern "thiscall" fn incoming_rpc_detour(
    receiver: *mut c_void,
    data: *mut u8,
    length: i32,
    player: RpcPlayerId,
) -> bool {
    let Some(state) = active_state() else {
        return false;
    };
    state
        .rpc_receiver
        .store(receiver as usize, Ordering::Release);
    state
        .player_address
        .store(player.binary_address, Ordering::Release);
    state.player_port.store(player.port, Ordering::Release);
    let original = state.incoming_rpc_trampoline.load(Ordering::Acquire);
    if original == 0 || data.is_null() || length < 0 {
        return false;
    }
    let original: IncomingRpcFn = unsafe { mem::transmute(original) };
    if !state.registry.has_rpc_listener(Direction::Incoming) {
        return unsafe { original(receiver, data, length, player) };
    }
    let input = unsafe { slice::from_raw_parts(data, length as usize) };
    let Ok((rpc_id, mut payload, timestamp)) = parse_rpc_envelope(input) else {
        return unsafe { original(receiver, data, length, player) };
    };
    if state
        .registry
        .dispatch_rpc(Direction::Incoming, rpc_id, &mut payload)
        == HookAction::Block
    {
        return false;
    }
    let Ok(mut output) = build_rpc_envelope(rpc_id, &payload, timestamp) else {
        return unsafe { original(receiver, data, length, player) };
    };
    unsafe { original(receiver, output.as_mut_ptr(), output.len() as i32, player) }
}

unsafe fn dispatch_packet_stream(
    state: &BackendState,
    direction: Direction,
    native: *mut RawBitStream,
) -> HookAction {
    if native.is_null() {
        return HookAction::Continue;
    }
    let Ok(mut stream) = (unsafe { (&*native).copy_to_owned() }) else {
        return HookAction::Continue;
    };
    let Ok(id) = stream.read_u8() else {
        return HookAction::Continue;
    };
    let remaining_bits = stream.remaining_bits();
    let capacity_bits = stream.capacity_bits().unwrap_or(remaining_bits);
    let mut payload = remaining_stream_bounded(
        &mut stream,
        remaining_bits,
        capacity_bits.saturating_sub(u8::BITS as usize),
    );
    let action = state.registry.dispatch_packet(direction, id, &mut payload);
    if action == HookAction::Continue
        && let Ok(combined) = packet_stream(id, &payload)
    {
        let _ = unsafe { (&mut *native).replace_from(&combined) };
    }
    action
}

unsafe fn dispatch_rpc_stream(
    state: &BackendState,
    direction: Direction,
    id: u8,
    native: *mut RawBitStream,
) -> HookAction {
    if native.is_null() {
        return HookAction::Continue;
    }
    let Ok(mut payload) = (unsafe { (&*native).copy_to_owned() }) else {
        return HookAction::Continue;
    };
    let action = state.registry.dispatch_rpc(direction, id, &mut payload);
    if action == HookAction::Continue {
        let _ = unsafe { (&mut *native).replace_from(&payload) };
    }
    action
}

unsafe fn dispatch_raw_packet(state: &BackendState, packet: *mut RawPacket) -> HookAction {
    if !state.registry.has_packet_listener(Direction::Incoming) {
        return HookAction::Continue;
    }
    if packet.is_null() {
        return HookAction::Continue;
    }
    let length = unsafe { ptr::addr_of!((*packet).length).read_unaligned() };
    let bit_size = unsafe { ptr::addr_of!((*packet).bit_size).read_unaligned() } as usize;
    let data = unsafe { ptr::addr_of!((*packet).data).read_unaligned() };
    let byte_len = validated_packet_byte_len(length, bit_size);
    let metadata_is_valid = !data.is_null() && byte_len.is_some();
    if !state
        .incoming_packet_diagnostic_logged
        .swap(true, Ordering::AcqRel)
    {
        if metadata_is_valid {
            log::debug!(
                "first incoming packet metadata is valid: length={length}, bit_size={bit_size}"
            );
        } else {
            log::warn!(
                "rejected invalid incoming packet metadata: length={length}, bit_size={bit_size}, data_is_null={} (traffic passed through unchanged)",
                data.is_null()
            );
        }
    }
    let Some(byte_len) = byte_len else {
        return HookAction::Continue;
    };
    if data.is_null() {
        return HookAction::Continue;
    }
    let bytes = unsafe { slice::from_raw_parts(data, byte_len) }.to_vec();
    let Ok(mut stream) = BitStream::from_bytes_with_capacity(bytes, bit_size, bit_size) else {
        return HookAction::Continue;
    };
    let Ok(id) = stream.read_u8() else {
        return HookAction::Continue;
    };
    let remaining_bits = stream.remaining_bits();
    let mut payload = remaining_stream_bounded(
        &mut stream,
        remaining_bits,
        bit_size.saturating_sub(u8::BITS as usize),
    );
    let action = state
        .registry
        .dispatch_packet(Direction::Incoming, id, &mut payload);
    if action == HookAction::Continue
        && let Ok(combined) = packet_stream(id, &payload)
        && combined.len_bits() <= bit_size
    {
        unsafe {
            ptr::copy_nonoverlapping(combined.as_bytes().as_ptr(), data, combined.len_bytes())
        };
        unsafe {
            ptr::addr_of_mut!((*packet).bit_size).write_unaligned(combined.len_bits() as u32)
        };
        unsafe { ptr::addr_of_mut!((*packet).length).write_unaligned(combined.len_bytes() as u32) };
    }
    action
}

fn validated_packet_byte_len(length: u32, bit_size: usize) -> Option<usize> {
    if bit_size < u8::BITS as usize {
        return None;
    }
    let byte_len = bit_size.checked_add(u8::BITS as usize - 1)? / u8::BITS as usize;
    if byte_len > length as usize
        || byte_len > MAX_INCOMING_PACKET_BYTES
        || byte_len > isize::MAX as usize
    {
        return None;
    }
    Some(byte_len)
}

#[cfg(test)]
mod packet_metadata_tests {
    use super::{MAX_INCOMING_PACKET_BYTES, native_bit_length, validated_packet_byte_len};
    use crate::SendError;

    #[test]
    fn accepts_byte_aligned_and_partial_byte_packets() {
        assert_eq!(validated_packet_byte_len(2, 16), Some(2));
        assert_eq!(validated_packet_byte_len(2, 9), Some(2));
    }

    #[test]
    fn rejects_metadata_that_cannot_describe_the_buffer() {
        assert_eq!(validated_packet_byte_len(1, 7), None);
        assert_eq!(validated_packet_byte_len(1, 9), None);
        assert_eq!(
            validated_packet_byte_len(
                (MAX_INCOMING_PACKET_BYTES + 1) as u32,
                (MAX_INCOMING_PACKET_BYTES + 1) * 8
            ),
            None
        );
    }

    #[test]
    fn rejects_bit_lengths_that_overflow_native_i32() {
        assert_eq!(native_bit_length(i32::MAX as usize), Ok(i32::MAX));
        assert_eq!(
            native_bit_length(i32::MAX as usize + 1),
            Err(SendError::PayloadTooLarge)
        );
    }
}

fn packet_stream(id: u8, payload: &BitStream) -> Result<BitStream, SendError> {
    let mut stream = BitStream::new();
    stream
        .write_u8(id)
        .map_err(|_| SendError::PayloadTooLarge)?;
    stream
        .write_stream(payload)
        .map_err(|_| SendError::PayloadTooLarge)?;
    Ok(stream)
}

fn remaining_stream(stream: &mut BitStream, bit_len: usize) -> BitStream {
    let mut payload = BitStream::new();
    copy_remaining(stream, bit_len, &mut payload);
    payload
}

fn remaining_stream_bounded(
    stream: &mut BitStream,
    bit_len: usize,
    capacity_bits: usize,
) -> BitStream {
    let mut payload = BitStream::with_capacity_bits(capacity_bits);
    copy_remaining(stream, bit_len, &mut payload);
    payload
}

fn copy_remaining(stream: &mut BitStream, bit_len: usize, payload: &mut BitStream) {
    for _ in 0..bit_len {
        if let Ok(bit) = stream.read_bool() {
            let _ = payload.write_bool(bit);
        }
    }
}

fn parse_rpc_envelope(input: &[u8]) -> Result<(u8, BitStream, Option<[u8; 4]>), SendError> {
    let mut stream = BitStream::from_bytes(input.to_vec());
    let first = stream.read_u8().map_err(|_| SendError::NativeCallFailed)?;
    let timestamp = if first == ID_TIMESTAMP {
        let bytes = stream
            .read_bytes(4)
            .map_err(|_| SendError::NativeCallFailed)?;
        let mut timestamp = [0; 4];
        timestamp.copy_from_slice(&bytes);
        if stream.read_u8().map_err(|_| SendError::NativeCallFailed)? != ID_RPC {
            return Err(SendError::NativeCallFailed);
        }
        Some(timestamp)
    } else if first == ID_RPC {
        None
    } else {
        return Err(SendError::NativeCallFailed);
    };
    let id = stream.read_u8().map_err(|_| SendError::NativeCallFailed)?;
    let payload_bits = stream
        .read_compressed_u32()
        .map_err(|_| SendError::NativeCallFailed)? as usize;
    if payload_bits > stream.remaining_bits() {
        return Err(SendError::NativeCallFailed);
    }
    Ok((id, remaining_stream(&mut stream, payload_bits), timestamp))
}

fn build_rpc_envelope(
    id: u8,
    payload: &BitStream,
    timestamp: Option<[u8; 4]>,
) -> Result<Vec<u8>, SendError> {
    let payload_bits = u32::try_from(payload.len_bits()).map_err(|_| SendError::PayloadTooLarge)?;
    let mut stream = BitStream::new();
    if let Some(timestamp) = timestamp {
        stream
            .write_u8(ID_TIMESTAMP)
            .map_err(|_| SendError::PayloadTooLarge)?;
        stream
            .write_bytes(&timestamp)
            .map_err(|_| SendError::PayloadTooLarge)?;
    }
    stream
        .write_u8(ID_RPC)
        .map_err(|_| SendError::PayloadTooLarge)?;
    stream
        .write_u8(id)
        .map_err(|_| SendError::PayloadTooLarge)?;
    stream
        .write_compressed_u32(payload_bits)
        .map_err(|_| SendError::PayloadTooLarge)?;
    stream
        .write_stream(payload)
        .map_err(|_| SendError::PayloadTooLarge)?;
    Ok(stream.as_bytes().to_vec())
}

fn active_state() -> Option<Arc<BackendState>> {
    ACTIVE_BACKEND.get().and_then(|slot| {
        slot.lock()
            .ok()
            .and_then(|state| state.as_ref().and_then(Weak::upgrade))
    })
}

fn clear_active_backend(target: &BackendState) {
    let Some(slot) = ACTIVE_BACKEND.get() else {
        return;
    };
    let mut active = slot.lock().unwrap_or_else(|error| error.into_inner());
    if active
        .as_ref()
        .and_then(Weak::upgrade)
        .is_some_and(|state| ptr::eq(Arc::as_ptr(&state), target))
    {
        *active = None;
    }
}

fn loaded_samp_module() -> Result<usize, AttachError> {
    let handle = unsafe { GetModuleHandleA(c"samp.dll".as_ptr().cast()) };
    if handle.is_null() {
        Err(AttachError::SampNotLoaded)
    } else {
        Ok(handle as usize)
    }
}

unsafe fn pe_entry_point(base: usize) -> Result<u32, AttachError> {
    let image = base as *const u8;
    if unsafe { image.cast::<u16>().read_unaligned() } != 0x5A4D {
        return Err(AttachError::UnsupportedClient { entry_point: 0 });
    }
    let nt_offset = unsafe { image.add(0x3C).cast::<u32>().read_unaligned() } as usize;
    let nt_header = unsafe { image.add(nt_offset) };
    if unsafe { nt_header.cast::<u32>().read_unaligned() } != 0x0000_4550 {
        return Err(AttachError::UnsupportedClient { entry_point: 0 });
    }
    if unsafe { nt_header.add(24).cast::<u16>().read_unaligned() } != 0x10B {
        return Err(AttachError::UnsupportedClient { entry_point: 0 });
    }
    Ok(unsafe { nt_header.add(40).cast::<u32>().read_unaligned() })
}

unsafe fn write_protected<T>(address: *mut T, value: T) -> Result<(), AttachError> {
    let mut old_protection = 0;
    if unsafe {
        VirtualProtect(
            address.cast(),
            mem::size_of::<T>(),
            PAGE_READWRITE,
            &mut old_protection,
        )
    } == 0
    {
        return Err(AttachError::HookInstallFailed("changing vtable protection"));
    }
    unsafe { address.write(value) };
    let mut ignored = 0;
    let _ = unsafe {
        VirtualProtect(
            address.cast(),
            mem::size_of::<T>(),
            old_protection,
            &mut ignored,
        )
    };
    Ok(())
}

fn call_outgoing_packet(
    state: &BackendState,
    client: *mut c_void,
    stream: *mut RawBitStream,
    priority: i32,
    reliability: i32,
    channel: i8,
) -> bool {
    let original = state.outgoing_packet_original.load(Ordering::Acquire);
    if original == 0 {
        return false;
    }
    let original: OutgoingPacketFn = unsafe { mem::transmute(original) };
    unsafe { original(client, stream, priority, reliability, channel) }
}

fn call_incoming_packet(state: &BackendState, client: *mut c_void) -> *mut RawPacket {
    let original = state.incoming_packet_original.load(Ordering::Acquire);
    if original == 0 {
        return ptr::null_mut();
    }
    let original: IncomingPacketFn = unsafe { mem::transmute(original) };
    unsafe { original(client) }
}

fn deallocate_packet(state: &BackendState, client: *mut c_void, packet: *mut RawPacket) {
    let original = state.deallocate_packet_original.load(Ordering::Acquire);
    if original != 0 {
        let original: DeallocatePacketFn = unsafe { mem::transmute(original) };
        unsafe { original(client, packet) };
    }
}

fn call_outgoing_rpc(state: &BackendState, call: OutgoingRpcCall) -> bool {
    let original = state.outgoing_rpc_original.load(Ordering::Acquire);
    if original == 0 {
        return false;
    }
    let original: OutgoingRpcFn = unsafe { mem::transmute(original) };
    unsafe {
        original(
            call.client,
            call.id,
            call.stream,
            call.priority,
            call.reliability,
            call.channel,
            call.timestamp,
        )
    }
}

const fn priority_value(priority: PacketPriority) -> i32 {
    match priority {
        PacketPriority::System => 0,
        PacketPriority::High => 1,
        PacketPriority::Medium => 2,
        PacketPriority::Low => 3,
    }
}

const fn reliability_value(reliability: PacketReliability) -> i32 {
    match reliability {
        PacketReliability::Unreliable => 6,
        PacketReliability::UnreliableSequenced => 7,
        PacketReliability::Reliable => 8,
        PacketReliability::ReliableOrdered => 9,
        PacketReliability::ReliableSequenced => 10,
    }
}
