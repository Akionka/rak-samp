//! Private SA-MP 0.3.7 R3-1 profile for verified cached CNetGame scalars.
//!
//! This deliberately covers only the read-only server metadata and game-state
//! fields. R3 player, UI, pool, and sync layouts remain unavailable until each
//! family has an independent fixture and live validation of its own.

use super::r1_client::memory::{bounded_c_string, read_pointer, read_unaligned, readable_range};
use crate::runtime::{DirectClientError, ServerInfoSnapshot};
use std::{ffi::c_void, mem};

const SAMP_R3_1_ENTRY_POINT: u32 = 0x0C_C4_D0;
const NET_GAME_SINGLETON_RVA: usize = 0x26_E8_DC;
const NET_GAME_HOST_ADDRESS_OFFSET: usize = 0x30;
const NET_GAME_HOSTNAME_OFFSET: usize = 0x131;
const NET_GAME_PORT_OFFSET: usize = 0x235;
const NET_GAME_GAME_STATE_OFFSET: usize = 0x3CD;
const NET_GAME_HOST_STRING_CAPACITY: usize = 257;
const NET_GAME_SCALAR_READABLE_SIZE: usize = NET_GAME_GAME_STATE_OFFSET + mem::size_of::<i32>();

/// The narrowly verified R3-1 CNetGame scalar profile.
#[derive(Clone, Copy, Debug)]
pub(super) struct R3ClientProfile {
    module_base: usize,
}

impl R3ClientProfile {
    /// Selects this partial profile only for the pinned R3-1 executable.
    pub(super) fn verify(module_base: usize, entry_point: u32) -> Option<Self> {
        (module_base != 0 && entry_point == SAMP_R3_1_ENTRY_POINT).then_some(Self { module_base })
    }

    /// Captures the R3-1 CNetGame state with a guarded scalar read.
    pub(super) fn game_state(self) -> Result<i32, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let field = (net_game as usize)
            .checked_add(NET_GAME_GAME_STATE_OFFSET)
            .ok_or(DirectClientError::NotReady)?;
        unsafe { read_unaligned::<i32>(field) }.ok_or(DirectClientError::NotReady)
    }

    /// Captures copied R3-1 server metadata from the guarded CNetGame fields.
    pub(super) fn server_info(self) -> Result<ServerInfoSnapshot, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let address = unsafe {
            bounded_c_string(
                net_game
                    .cast::<u8>()
                    .wrapping_add(NET_GAME_HOST_ADDRESS_OFFSET),
                NET_GAME_HOST_STRING_CAPACITY,
            )
        }
        .filter(|address| !address.is_empty())
        .ok_or(DirectClientError::NotReady)?;
        let hostname = unsafe {
            bounded_c_string(
                net_game.cast::<u8>().wrapping_add(NET_GAME_HOSTNAME_OFFSET),
                NET_GAME_HOST_STRING_CAPACITY,
            )
        }
        .filter(|hostname| !hostname.is_empty())
        .ok_or(DirectClientError::NotReady)?;
        let port_field = (net_game as usize)
            .checked_add(NET_GAME_PORT_OFFSET)
            .ok_or(DirectClientError::NotReady)?;
        let port = unsafe { read_unaligned::<i32>(port_field) }
            .and_then(|port| u16::try_from(port).ok())
            .filter(|port| *port != 0)
            .ok_or(DirectClientError::NotReady)?;
        Ok(ServerInfoSnapshot {
            address,
            hostname,
            port,
        })
    }

    fn net_game(self) -> Option<*mut c_void> {
        let net_game: *mut c_void =
            unsafe { read_pointer(self.module_base.checked_add(NET_GAME_SINGLETON_RVA)?) }?.cast();
        (!net_game.is_null() && readable_range(net_game.cast(), NET_GAME_SCALAR_READABLE_SIZE))
            .then_some(net_game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn reads_verified_r3_netgame_scalars() {
        let mut module = vec![0_u8; NET_GAME_SINGLETON_RVA + std::mem::size_of::<usize>()];
        let mut net_game = vec![0_u8; NET_GAME_GAME_STATE_OFFSET + std::mem::size_of::<i32>()];
        let module_base = module.as_mut_ptr() as usize;
        let net_game_pointer = net_game.as_mut_ptr();
        unsafe {
            ptr::write_unaligned(
                module
                    .as_mut_ptr()
                    .add(NET_GAME_SINGLETON_RVA)
                    .cast::<usize>(),
                net_game_pointer as usize,
            );
            ptr::write_unaligned(
                net_game_pointer.add(NET_GAME_PORT_OFFSET).cast::<i32>(),
                7777,
            );
            ptr::write_unaligned(
                net_game_pointer
                    .add(NET_GAME_GAME_STATE_OFFSET)
                    .cast::<i32>(),
                6,
            );
        }
        net_game[NET_GAME_HOST_ADDRESS_OFFSET..NET_GAME_HOST_ADDRESS_OFFSET + 9]
            .copy_from_slice(b"127.0.0.1");
        net_game[NET_GAME_HOSTNAME_OFFSET..NET_GAME_HOSTNAME_OFFSET + 8]
            .copy_from_slice(b"R3 probe");

        let profile = R3ClientProfile::verify(module_base, SAMP_R3_1_ENTRY_POINT).unwrap();

        assert_eq!(profile.game_state(), Ok(6));
        assert_eq!(
            profile.server_info(),
            Ok(ServerInfoSnapshot {
                address: b"127.0.0.1".to_vec(),
                hostname: b"R3 probe".to_vec(),
                port: 7777,
            })
        );
    }

    #[test]
    fn rejects_other_entry_points() {
        assert!(R3ClientProfile::verify(0x10000, SAMP_R3_1_ENTRY_POINT).is_some());
        assert!(R3ClientProfile::verify(0x10000, SAMP_R3_1_ENTRY_POINT - 1).is_none());
    }
}
