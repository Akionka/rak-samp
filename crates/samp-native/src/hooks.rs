//! SA-MP RakClient hook ownership and host callback forwarding.

use modkit_win32::InlineHook;
use std::{
    ffi::c_void,
    mem, ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

pub const OUTGOING_PACKET_SLOT: usize = 6;
pub const INCOMING_PACKET_SLOT: usize = 8;
pub const DEALLOCATE_PACKET_SLOT: usize = 9;
pub const OUTGOING_RPC_SLOT: usize = 25;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RpcPlayerId {
    pub binary_address: u32,
    pub port: u16,
}

pub struct HookCallbacks {
    pub outgoing_packet: unsafe extern "thiscall" fn(
        client: *mut c_void,
        native: *mut c_void,
        priority: i32,
        reliability: i32,
        channel: i8,
    ) -> bool,
    pub outgoing_rpc: unsafe extern "thiscall" fn(
        client: *mut c_void,
        id: *mut i32,
        native: *mut c_void,
        priority: i32,
        reliability: i32,
        channel: i8,
        timestamp: bool,
    ) -> bool,
    pub incoming_packet: unsafe extern "thiscall" fn(client: *mut c_void) -> *mut c_void,
    pub incoming_rpc: unsafe extern "thiscall" fn(
        receiver: *mut c_void,
        data: *mut u8,
        length: i32,
        player: RpcPlayerId,
    ) -> bool,
    pub dialog_close: unsafe extern "thiscall" fn(dialog: *mut c_void, button: u8),
    pub rak_client_constructor: unsafe extern "C" fn() -> *mut c_void,
}

static CALLBACKS: AtomicPtr<HookCallbacks> = AtomicPtr::new(ptr::null_mut());

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackRegistrationError {
    DifferentTableRegistered,
}

pub fn register_hook_callbacks(
    callbacks: &'static HookCallbacks,
) -> Result<(), CallbackRegistrationError> {
    let pointer = ptr::from_ref(callbacks).cast_mut();
    match CALLBACKS.compare_exchange(
        ptr::null_mut(),
        pointer,
        Ordering::AcqRel,
        Ordering::Acquire,
    ) {
        Ok(_) => Ok(()),
        Err(current) if current == pointer => Ok(()),
        Err(_) => Err(CallbackRegistrationError::DifferentTableRegistered),
    }
}

fn callbacks() -> Option<&'static HookCallbacks> {
    let pointer = CALLBACKS.load(Ordering::Acquire);
    if pointer.is_null() {
        None
    } else {
        // SAFETY: registration accepts only a static table and never replaces it.
        Some(unsafe { &*pointer })
    }
}

/// RakClient vtable detour. Installed only by [`VtableHook::install`].
///
/// # Safety
///
/// RakClient must invoke this function with its verified x86 ABI and arguments.
pub unsafe extern "thiscall" fn outgoing_packet_detour(
    client: *mut c_void,
    native: *mut c_void,
    priority: i32,
    reliability: i32,
    channel: i8,
) -> bool {
    callbacks().is_some_and(|callbacks| unsafe {
        (callbacks.outgoing_packet)(client, native, priority, reliability, channel)
    })
}

/// RakClient vtable detour. Installed only by [`VtableHook::install`].
///
/// # Safety
///
/// RakClient must invoke this function with its verified x86 ABI and arguments.
pub unsafe extern "thiscall" fn outgoing_rpc_detour(
    client: *mut c_void,
    id: *mut i32,
    native: *mut c_void,
    priority: i32,
    reliability: i32,
    channel: i8,
    timestamp: bool,
) -> bool {
    callbacks().is_some_and(|callbacks| unsafe {
        (callbacks.outgoing_rpc)(
            client,
            id,
            native,
            priority,
            reliability,
            channel,
            timestamp,
        )
    })
}

/// RakClient vtable detour. Installed only by [`VtableHook::install`].
///
/// # Safety
///
/// RakClient must invoke this function with its verified x86 ABI and arguments.
pub unsafe extern "thiscall" fn incoming_packet_detour(client: *mut c_void) -> *mut c_void {
    callbacks().map_or(ptr::null_mut(), |callbacks| unsafe {
        (callbacks.incoming_packet)(client)
    })
}

/// SA-MP incoming-RPC detour forwarded to host listener composition.
///
/// # Safety
///
/// SA-MP must invoke this function with the verified profile ABI and arguments.
pub unsafe extern "thiscall" fn incoming_rpc_detour(
    receiver: *mut c_void,
    data: *mut u8,
    length: i32,
    player: RpcPlayerId,
) -> bool {
    callbacks().is_some_and(|callbacks| unsafe {
        (callbacks.incoming_rpc)(receiver, data, length, player)
    })
}

/// SA-MP dialog-close detour forwarded to host cache composition.
///
/// # Safety
///
/// SA-MP must invoke this function with the verified profile ABI and arguments.
pub unsafe extern "thiscall" fn dialog_close_detour(dialog: *mut c_void, button: u8) {
    if let Some(callbacks) = callbacks() {
        unsafe { (callbacks.dialog_close)(dialog, button) };
    }
}

/// SA-MP RakClient-constructor detour forwarded to host lifecycle composition.
///
/// # Safety
///
/// SA-MP must invoke this function with the verified profile ABI.
pub unsafe extern "C" fn rak_client_constructor_detour() -> *mut c_void {
    callbacks().map_or(ptr::null_mut(), |callbacks| unsafe {
        (callbacks.rak_client_constructor)()
    })
}

#[derive(Default)]
pub struct HookStorage {
    pub constructor: Option<InlineHook>,
    pub incoming_rpc: Option<InlineHook>,
    pub dialog_close: Option<InlineHook>,
    pub vtable: Option<VtableHook>,
}

pub struct VtableHook {
    vtable: usize,
    entries: [VtableEntry; 3],
}

#[derive(Clone, Copy)]
struct VtableEntry {
    slot: usize,
    original: usize,
    detour: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VtableHookError {
    ClientNotReady,
    PatchFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RakClientOriginals {
    pub outgoing_packet: usize,
    pub incoming_packet: usize,
    pub deallocate_packet: usize,
    pub outgoing_rpc: usize,
}

impl VtableHook {
    /// Patches the three host-owned RakClient vtable slots and captures originals.
    ///
    /// # Safety
    ///
    /// `client` must be the active RakClient object for the selected SA-MP
    /// profile. The caller must serialize installation and retain the returned
    /// hook until callbacks have drained.
    pub unsafe fn install(
        client: *mut c_void,
    ) -> Result<(Self, RakClientOriginals), VtableHookError> {
        let Some(vtable) = (unsafe { modkit_win32::read_unaligned::<usize>(client as usize) })
        else {
            return Err(VtableHookError::ClientNotReady);
        };
        if vtable == 0 {
            return Err(VtableHookError::ClientNotReady);
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
        let maximum_slot = replacements
            .iter()
            .map(|(slot, _)| *slot)
            .chain(std::iter::once(DEALLOCATE_PACKET_SLOT))
            .max()
            .ok_or(VtableHookError::ClientNotReady)?;
        let required_bytes = maximum_slot
            .checked_add(1)
            .and_then(|count| count.checked_mul(mem::size_of::<usize>()))
            .ok_or(VtableHookError::ClientNotReady)?;
        if !modkit_win32::readable_range(vtable as *const u8, required_bytes) {
            return Err(VtableHookError::ClientNotReady);
        }

        let mut entries = [VtableEntry {
            slot: 0,
            original: 0,
            detour: 0,
        }; 3];
        for (index, (slot, detour)) in replacements.into_iter().enumerate() {
            let slot_address =
                vtable_slot_address(vtable, slot).ok_or(VtableHookError::ClientNotReady)?;
            let original = unsafe { modkit_win32::read_unaligned::<usize>(slot_address) }
                .ok_or(VtableHookError::ClientNotReady)?;
            if original == 0 {
                return Err(VtableHookError::ClientNotReady);
            }
            entries[index] = VtableEntry {
                slot,
                original,
                detour,
            };
        }

        let deallocate_packet = unsafe {
            modkit_win32::read_unaligned::<usize>(
                vtable_slot_address(vtable, DEALLOCATE_PACKET_SLOT)
                    .ok_or(VtableHookError::ClientNotReady)?,
            )
        }
        .ok_or(VtableHookError::ClientNotReady)?;
        if deallocate_packet == 0 {
            return Err(VtableHookError::ClientNotReady);
        }

        for (index, entry) in entries.iter().enumerate() {
            let slot = vtable_slot_address(vtable, entry.slot)
                .ok_or(VtableHookError::ClientNotReady)? as *mut usize;
            if unsafe { modkit_win32::write_protected(slot, entry.detour) }.is_err() {
                for restore in entries[..index].iter().rev() {
                    if let Some(address) = vtable_slot_address(vtable, restore.slot)
                        && let Err(error) = unsafe {
                            modkit_win32::write_protected(address as *mut usize, restore.original)
                        }
                    {
                        log::warn!(
                            "failed to roll back RakClient vtable slot {}: {error:?}",
                            restore.slot
                        );
                    }
                }
                return Err(VtableHookError::PatchFailed);
            }
        }

        let originals = RakClientOriginals {
            outgoing_packet: entries[0].original,
            incoming_packet: entries[1].original,
            deallocate_packet,
            outgoing_rpc: entries[2].original,
        };
        Ok((Self { vtable, entries }, originals))
    }
}

impl Drop for VtableHook {
    fn drop(&mut self) {
        for entry in self.entries.iter().rev() {
            let Some(slot) = vtable_slot_address(self.vtable, entry.slot) else {
                continue;
            };
            if unsafe { modkit_win32::read_unaligned::<usize>(slot) } == Some(entry.detour)
                && let Err(error) =
                    unsafe { modkit_win32::write_protected(slot as *mut usize, entry.original) }
            {
                log::warn!(
                    "failed to restore RakClient vtable slot {}: {error:?}",
                    entry.slot
                );
            }
        }
    }
}

fn vtable_slot_address(vtable: usize, slot: usize) -> Option<usize> {
    slot.checked_mul(mem::size_of::<usize>())
        .and_then(|offset| vtable.checked_add(offset))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    static DIALOG_CALLED: AtomicBool = AtomicBool::new(false);

    unsafe extern "thiscall" fn outgoing_packet_callback(
        _client: *mut c_void,
        _native: *mut c_void,
        _priority: i32,
        _reliability: i32,
        _channel: i8,
    ) -> bool {
        true
    }

    unsafe extern "thiscall" fn outgoing_rpc_callback(
        _client: *mut c_void,
        _id: *mut i32,
        _native: *mut c_void,
        _priority: i32,
        _reliability: i32,
        _channel: i8,
        _timestamp: bool,
    ) -> bool {
        true
    }

    unsafe extern "thiscall" fn incoming_packet_callback(client: *mut c_void) -> *mut c_void {
        client
    }

    unsafe extern "thiscall" fn incoming_rpc_callback(
        _receiver: *mut c_void,
        _data: *mut u8,
        length: i32,
        _player: RpcPlayerId,
    ) -> bool {
        length == 7
    }

    unsafe extern "thiscall" fn dialog_close_callback(_dialog: *mut c_void, _button: u8) {
        DIALOG_CALLED.store(true, Ordering::Release);
    }

    unsafe extern "C" fn constructor_callback() -> *mut c_void {
        ptr::dangling_mut::<c_void>()
    }

    static TEST_CALLBACKS: HookCallbacks = HookCallbacks {
        outgoing_packet: outgoing_packet_callback,
        outgoing_rpc: outgoing_rpc_callback,
        incoming_packet: incoming_packet_callback,
        incoming_rpc: incoming_rpc_callback,
        dialog_close: dialog_close_callback,
        rak_client_constructor: constructor_callback,
    };

    #[test]
    fn detours_forward_through_the_registered_host_table() {
        register_hook_callbacks(&TEST_CALLBACKS).unwrap();
        let client = ptr::dangling_mut::<c_void>();
        assert!(unsafe { outgoing_packet_detour(client, ptr::null_mut(), 0, 0, 0) });
        assert!(unsafe {
            outgoing_rpc_detour(client, ptr::null_mut(), ptr::null_mut(), 0, 0, 0, false)
        });
        assert_eq!(unsafe { incoming_packet_detour(client) }, client);
        assert!(unsafe {
            incoming_rpc_detour(
                ptr::null_mut(),
                ptr::null_mut(),
                7,
                RpcPlayerId {
                    binary_address: 0,
                    port: 0,
                },
            )
        });
        unsafe { dialog_close_detour(ptr::null_mut(), 0) };
        assert!(DIALOG_CALLED.load(Ordering::Acquire));
        assert_eq!(
            unsafe { rak_client_constructor_detour() },
            ptr::dangling_mut::<c_void>()
        );
    }

    #[test]
    fn vtable_slot_address_checks_multiplication_and_addition() {
        assert_eq!(vtable_slot_address(0x1000, 6), Some(0x1018));
        assert_eq!(vtable_slot_address(usize::MAX, 1), None);
        assert_eq!(vtable_slot_address(1, usize::MAX), None);
    }
}
