//! Explicitly unsafe native-address accessors.
//!
//! These functions never construct Rust references to client memory. A returned
//! address is valid only while the loaded SA-MP client and host remain present;
//! callers must validate every native layout, range, and lifetime before use.

use core::{ffi::c_void, ptr::NonNull};

const R1_DIALOG_SINGLETON_RVA: usize = 0x21A0B8;
const R1_CHAT_SINGLETON_RVA: usize = 0x21A0E4;
const R1_INPUT_SINGLETON_RVA: usize = 0x21A0E8;
const R1_DEATH_WINDOW_SINGLETON_RVA: usize = 0x21A0EC;
const R1_NET_GAME_SINGLETON_RVA: usize = 0x21A0F8;
const R1_GAME_SINGLETON_RVA: usize = 0x21A10C;
const R1_NET_GAME_POOLS_OFFSET: usize = 0x3CD;
const R1_NET_GAME_SERVER_SETTINGS_OFFSET: usize = 0x3C5;
const R1_PICKUP_POOL_OFFSET: usize = 0x20;
const R1_OBJECT_POOL_OFFSET: usize = 0x04;
const R1_GANGZONE_POOL_OFFSET: usize = 0x08;
const R1_TEXT_LABEL_POOL_OFFSET: usize = 0x0C;
const R1_TEXTDRAW_POOL_OFFSET: usize = 0x10;
const R1_RAKPEER_RPC_NODE_TABLE_OFFSET: usize = 0x801;
const R1_RPC_NODE_CALLBACK_OFFSET: usize = 0x01;
const RAKCLIENT_VTABLE_SLOT_COUNT: usize = 26;

/// Returns the loaded `samp.dll` module base as an opaque native address.
///
/// # Safety
///
/// The returned pointer has no Rust provenance over SA-MP memory and must not
/// be dereferenced as a Rust reference. It becomes invalid if SA-MP unloads.
#[must_use]
pub unsafe fn base() -> Option<NonNull<c_void>> {
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;

    let module = unsafe { GetModuleHandleA(c"samp.dll".as_ptr().cast()) };
    NonNull::new(module.cast())
}

/// Returns the host-captured R1 RakClient interface as an opaque native address.
///
/// # Safety
///
/// `samp` must refer to a live host attached to SA-MP 0.3.7 R1. The returned
/// address must never be made into a Rust reference and becomes invalid when
/// the RakClient instance or SA-MP unloads.
pub unsafe fn rakclient(samp: crate::Samp) -> Result<NonNull<c_void>, crate::SampClientSdkResult> {
    samp.api().raw_rakclient()
}

/// Returns the validated R1 RakPeer base as an opaque native address.
///
/// # Safety
///
/// `samp` must refer to a live host attached to SA-MP 0.3.7 R1. The returned
/// address must never be made into a Rust reference and becomes invalid when
/// the RakClient instance or SA-MP unloads.
pub unsafe fn rakpeer(samp: crate::Samp) -> Result<NonNull<c_void>, crate::SampClientSdkResult> {
    samp.api().raw_rakpeer()
}

/// Returns the latest game-thread-captured R1 player pool as an opaque native address.
///
/// # Safety
///
/// `samp` must refer to a live host attached to SA-MP 0.3.7 R1. The address is
/// refreshed only from the host game thread, must never be made into a Rust
/// reference, and becomes invalid when the pool or SA-MP unloads.
pub unsafe fn player_pool(
    samp: crate::Samp,
) -> Result<NonNull<c_void>, crate::SampClientSdkResult> {
    samp.api().raw_player_pool()
}

/// Returns the latest game-thread-captured R1 vehicle pool as an opaque native address.
///
/// # Safety
///
/// `samp` must refer to a live host attached to SA-MP 0.3.7 R1. The address is
/// refreshed only from the host game thread, must never be made into a Rust
/// reference, and becomes invalid when the pool or SA-MP unloads.
pub unsafe fn vehicle_pool(
    samp: crate::Samp,
) -> Result<NonNull<c_void>, crate::SampClientSdkResult> {
    samp.api().raw_vehicle_pool()
}

/// Returns the latest game-thread-captured local-player object as an opaque
/// native address.
///
/// # Safety
///
/// `samp` must refer to a live host attached to a supported SA-MP 0.3.7 build.
/// The address is refreshed only on the host game thread, must never be made
/// into a Rust reference, and becomes invalid when the local player or SA-MP
/// unloads.
pub unsafe fn player(samp: crate::Samp) -> Result<NonNull<c_void>, crate::SampClientSdkResult> {
    samp.api().raw_local_player()
}

/// Returns one address from the host-captured RakClient vtable.
///
/// `index` is bounded to the 26 slots the host recognizes (0 through 25).
/// A missing vtable entry returns `Ok(None)`.
///
/// # Safety
///
/// `samp` must refer to a live host attached to SA-MP. The returned code
/// address must not be called with an invented signature or retained after
/// RakClient/SA-MP unloads. This accessor reads native vtable memory and does
/// not create a Rust reference to it.
pub unsafe fn rakclient_function(
    samp: crate::Samp,
    index: usize,
) -> Result<Option<NonNull<c_void>>, crate::SampClientSdkResult> {
    if index >= RAKCLIENT_VTABLE_SLOT_COUNT {
        return Ok(None);
    }
    let client = unsafe { rakclient(samp) }?;
    Ok(unsafe { vtable_function(client, index) })
}

/// Returns the packed R1 RakPeer RPC node for one RPC index.
///
/// A missing RPC node returns `Ok(None)`.
///
/// # Safety
///
/// `samp` must refer to a live host attached to SA-MP 0.3.7 R1. The returned
/// address must not be made into a Rust reference or retained after
/// RakClient/RakPeer or SA-MP unloads. This accessor reads the native packed
/// RPC-node table and does not create a Rust reference to it.
pub unsafe fn rpc_node(
    samp: crate::Samp,
    rpc_id: u8,
) -> Result<Option<NonNull<c_void>>, crate::SampClientSdkResult> {
    let peer = unsafe { rakpeer(samp) }?;
    Ok(unsafe { r1_rpc_node(peer, rpc_id) })
}

/// Returns the callback address registered for one R1 RakPeer RPC index.
///
/// A missing RPC node or callback returns `Ok(None)`.
///
/// # Safety
///
/// `samp` must refer to a live host attached to SA-MP 0.3.7 R1. The returned
/// code address must not be called with an invented signature or retained
/// after RakClient/RakPeer or SA-MP unloads. This accessor reads the native
/// packed RPC-node table and does not create a Rust reference to it.
pub unsafe fn rpc_callback(
    samp: crate::Samp,
    rpc_id: u8,
) -> Result<Option<NonNull<c_void>>, crate::SampClientSdkResult> {
    let peer = unsafe { rakpeer(samp) }?;
    Ok(unsafe { r1_rpc_node(peer, rpc_id) }
        .and_then(|node| unsafe { r1_field(node, R1_RPC_NODE_CALLBACK_OFFSET) }))
}

/// Returns the address of an owned [`samp_protocol::BitStream`]'s byte storage.
///
/// This mirrors `raknetBitStreamGetDataPtr` for the SDK's owned, bounded
/// stream representation. The address may be dangling when the stream has no
/// bytes and must not be dereferenced in that case.
///
/// # Safety
///
/// The returned pointer is valid only while `stream` remains alive and is not
/// mutably borrowed or changed in a way that can reallocate its backing
/// storage. It must not outlive the stream or be used to read beyond
/// [`samp_protocol::BitStream::len_bytes`].
#[must_use]
pub unsafe fn bitstream_data(stream: &samp_protocol::BitStream) -> *const u8 {
    stream.as_bytes().as_ptr()
}

/// Returns the R1 `CChat` singleton as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn chat() -> Option<NonNull<c_void>> {
    unsafe { r1_singleton(R1_CHAT_SINGLETON_RVA) }
}

/// Returns the R1 death-window singleton as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn death_window() -> Option<NonNull<c_void>> {
    unsafe { r1_singleton(R1_DEATH_WINDOW_SINGLETON_RVA) }
}

/// Returns the R1 dialog singleton as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn dialog() -> Option<NonNull<c_void>> {
    unsafe { r1_singleton(R1_DIALOG_SINGLETON_RVA) }
}

/// Returns the R1 GTA-game/misc singleton as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn misc() -> Option<NonNull<c_void>> {
    unsafe { r1_singleton(R1_GAME_SINGLETON_RVA) }
}

/// Returns the R1 chat-input singleton as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn chat_input() -> Option<NonNull<c_void>> {
    unsafe { r1_singleton(R1_INPUT_SINGLETON_RVA) }
}

/// Returns the R1 `CNetGame` singleton as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn net_game() -> Option<NonNull<c_void>> {
    unsafe { r1_singleton(R1_NET_GAME_SINGLETON_RVA) }
}

/// Returns the R1 `CNetGame` server-settings object as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when the client unloads or replaces its
/// current connection settings.
#[must_use]
pub unsafe fn server_settings() -> Option<NonNull<c_void>> {
    unsafe { r1_field(net_game()?, R1_NET_GAME_SERVER_SETTINGS_OFFSET) }
}

/// Returns the R1 `CNetGame` pools aggregate as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn pools() -> Option<NonNull<c_void>> {
    unsafe { r1_field(net_game()?, R1_NET_GAME_POOLS_OFFSET) }
}

/// Returns the R1 3D-text-label pool as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn text_label_pool() -> Option<NonNull<c_void>> {
    unsafe { r1_field(pools()?, R1_TEXT_LABEL_POOL_OFFSET) }
}

/// Returns the R1 textdraw pool as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn textdraw_pool() -> Option<NonNull<c_void>> {
    unsafe { r1_field(pools()?, R1_TEXTDRAW_POOL_OFFSET) }
}

/// Returns the R1 pickup pool as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when the pool or SA-MP unloads.
#[must_use]
pub unsafe fn pickup_pool() -> Option<NonNull<c_void>> {
    unsafe { r1_field(pools()?, R1_PICKUP_POOL_OFFSET) }
}

/// Returns the R1 object pool as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn object_pool() -> Option<NonNull<c_void>> {
    unsafe { r1_field(pools()?, R1_OBJECT_POOL_OFFSET) }
}

/// Returns the R1 gangzone pool as an opaque native address.
///
/// # Safety
///
/// Requires SA-MP 0.3.7 R1. The result must never be made into a Rust
/// reference and becomes invalid when SA-MP unloads.
#[must_use]
pub unsafe fn gangzone_pool() -> Option<NonNull<c_void>> {
    unsafe { r1_field(pools()?, R1_GANGZONE_POOL_OFFSET) }
}

unsafe fn r1_singleton(rva: usize) -> Option<NonNull<c_void>> {
    let base = unsafe { base() }?;
    let slot = (base.as_ptr() as usize).checked_add(rva)? as *const *mut c_void;
    NonNull::new(unsafe { slot.read_unaligned() })
}

unsafe fn r1_field(object: NonNull<c_void>, offset: usize) -> Option<NonNull<c_void>> {
    let slot = (object.as_ptr() as usize).checked_add(offset)? as *const *mut c_void;
    NonNull::new(unsafe { slot.read_unaligned() })
}

unsafe fn vtable_function(object: NonNull<c_void>, index: usize) -> Option<NonNull<c_void>> {
    let vtable = unsafe { object.cast::<*const c_void>().as_ptr().read_unaligned() };
    let vtable = NonNull::new(vtable as *mut c_void)?;
    let slot = unsafe { vtable.cast::<*const c_void>().as_ptr().add(index) };
    NonNull::new(unsafe { slot.read_unaligned().cast_mut() })
}

unsafe fn r1_rpc_node(peer: NonNull<c_void>, rpc_id: u8) -> Option<NonNull<c_void>> {
    let offset = R1_RAKPEER_RPC_NODE_TABLE_OFFSET
        .checked_add(usize::from(rpc_id).checked_mul(core::mem::size_of::<*mut c_void>())?)?;
    unsafe { r1_field(peer, offset) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rakclient_uses_the_host_captured_address() {
        let samp = crate::Samp::from_api(crate::events::test_support::test_api());
        assert_eq!(
            unsafe { rakclient(samp) }.unwrap().as_ptr() as usize,
            0x1000
        );
    }

    #[test]
    fn rakpeer_uses_the_host_validated_address() {
        let samp = crate::Samp::from_api(crate::events::test_support::test_api());
        assert_eq!(unsafe { rakpeer(samp) }.unwrap().as_ptr() as usize, 0x2222);
    }

    #[test]
    fn pools_use_game_thread_captured_addresses() {
        let samp = crate::Samp::from_api(crate::events::test_support::test_api());
        assert_eq!(
            unsafe { player_pool(samp) }.unwrap().as_ptr() as usize,
            0x1001
        );
        assert_eq!(
            unsafe { vehicle_pool(samp) }.unwrap().as_ptr() as usize,
            0x1002
        );
        assert_eq!(unsafe { player(samp) }.unwrap().as_ptr() as usize, 0x1003);
    }

    #[test]
    fn vtable_address_is_bounded_and_opaque() {
        let function = 0x2000usize as *const c_void;
        let table = [function; RAKCLIENT_VTABLE_SLOT_COUNT];
        let mut object = table.as_ptr();
        let object = NonNull::from(&mut object).cast();

        assert_eq!(
            unsafe { vtable_function(object, 25) }.unwrap().as_ptr() as usize,
            0x2000
        );
    }

    #[test]
    fn rpc_node_and_callback_read_the_packed_r1_rpc_node_table() {
        let callback = 0x3000usize as *mut c_void;
        let rpc_id = 17;
        let mut node =
            vec![0_u8; R1_RPC_NODE_CALLBACK_OFFSET + core::mem::size_of::<*mut c_void>()];
        let mut peer = vec![
            0_u8;
            R1_RAKPEER_RPC_NODE_TABLE_OFFSET
                + (usize::from(u8::MAX) + 1) * core::mem::size_of::<*mut c_void>()
        ];
        unsafe {
            node.as_mut_ptr()
                .add(R1_RPC_NODE_CALLBACK_OFFSET)
                .cast::<*mut c_void>()
                .write_unaligned(callback);
            peer.as_mut_ptr()
                .add(
                    R1_RAKPEER_RPC_NODE_TABLE_OFFSET
                        + usize::from(rpc_id) * core::mem::size_of::<*mut c_void>(),
                )
                .cast::<*mut c_void>()
                .write_unaligned(node.as_mut_ptr().cast());
            peer.as_mut_ptr()
                .add(
                    R1_RAKPEER_RPC_NODE_TABLE_OFFSET
                        + usize::from(u8::MAX) * core::mem::size_of::<*mut c_void>(),
                )
                .cast::<*mut c_void>()
                .write_unaligned(node.as_mut_ptr().cast());
        }
        let peer = NonNull::new(peer.as_mut_ptr()).unwrap().cast();

        assert_eq!(
            unsafe { r1_rpc_node(peer, rpc_id) }.unwrap().as_ptr(),
            node.as_mut_ptr().cast()
        );
        assert_eq!(unsafe { r1_rpc_node(peer, rpc_id + 1) }, None,);
        assert_eq!(
            unsafe { r1_rpc_node(peer, u8::MAX) }.unwrap().as_ptr(),
            node.as_mut_ptr().cast()
        );
        assert_eq!(
            unsafe { r1_rpc_node(peer, rpc_id) }
                .and_then(|node| unsafe { r1_field(node, R1_RPC_NODE_CALLBACK_OFFSET) })
                .unwrap()
                .as_ptr(),
            callback,
        );
    }

    #[test]
    fn bitstream_data_borrows_the_owned_stream_storage() {
        let stream = samp_protocol::BitStream::from_bytes([0x12, 0x34]).unwrap();
        let data = unsafe { super::bitstream_data(&stream) };
        assert_eq!(data, stream.as_bytes().as_ptr());
        assert_eq!(
            unsafe { std::slice::from_raw_parts(data, stream.len_bytes()) },
            [0x12, 0x34]
        );
    }
}
