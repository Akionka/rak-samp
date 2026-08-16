//! Shared CNetGame connection operations backed by immutable profile data.

use super::{
    memory::{
        bounded_c_string, copy_bytes, read_pointer, read_unaligned, readable_range, writable_range,
        write_unaligned, zero_bytes,
    },
    profile::{GameStateCodec, NativeClientProfile},
};
use crate::runtime::{DirectClientError, ServerInfoSnapshot};
use std::{ffi::c_void, mem};

type R1NetGameGetStateFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type ProfileRakClientDisconnectFn = unsafe extern "thiscall" fn(*mut c_void, u32, u8);
type ProfileNetGameShutdownFn = unsafe extern "thiscall" fn(*mut c_void);

impl GameStateCodec {
    pub(crate) const fn encode(self, state: i32) -> Option<i32> {
        match self {
            Self::Identity => match state {
                0 | 9 | 13 | 14 | 15 | 18 => Some(state),
                _ => None,
            },
            Self::Classic => match state {
                0 => Some(0),
                9 => Some(1),
                13 => Some(2),
                14 => Some(5),
                15 => Some(6),
                18 => Some(11),
                _ => None,
            },
        }
    }

    pub(crate) const fn decode(self, state: i32) -> Option<i32> {
        match self {
            Self::Identity => match state {
                0 | 9 | 13 | 14 | 15 | 18 => Some(state),
                _ => None,
            },
            Self::Classic => match state {
                0 => Some(0),
                1 => Some(9),
                2 => Some(13),
                5 => Some(14),
                6 => Some(15),
                11 => Some(18),
                _ => None,
            },
        }
    }
}

impl NativeClientProfile {
    pub(crate) fn rakpeer_address(
        self,
        rakclient: *mut c_void,
    ) -> Result<*mut c_void, DirectClientError> {
        let peer = (rakclient as usize)
            .checked_sub(self.spec.handles.rakpeer_size.get())
            .ok_or(DirectClientError::NotReady)? as *mut c_void;
        if peer.is_null() || !readable_range(peer.cast(), self.spec.handles.rakpeer_size.get() + 1)
        {
            return Err(DirectClientError::NotReady);
        }
        Ok(peer)
    }

    pub(crate) fn game_state(self) -> Result<i32, DirectClientError> {
        let net_game = self.net_game()?;
        let native = match self.spec.net_game.get_state_rva {
            Some(rva) => {
                let target = self
                    .module_base
                    .checked_add(rva.get())
                    .filter(|target| readable_range(*target as *const u8, 1))
                    .ok_or(DirectClientError::NotReady)?;
                let get_state: R1NetGameGetStateFn = unsafe { mem::transmute(target) };
                unsafe { get_state(net_game) }
            }
            None => {
                let field = (net_game as usize)
                    .checked_add(self.spec.net_game.game_state_offset.get())
                    .ok_or(DirectClientError::NotReady)?;
                unsafe { read_unaligned::<i32>(field) }.ok_or(DirectClientError::NotReady)?
            }
        };
        self.spec
            .strategies
            .game_state_codec
            .decode(native)
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn set_game_state(self, state: i32) -> Result<(), DirectClientError> {
        let native_state = self
            .spec
            .strategies
            .game_state_codec
            .encode(state)
            .ok_or(DirectClientError::NotReady)?;
        let net_game = self.net_game()?;
        let field = (net_game as usize)
            .checked_add(self.spec.net_game.game_state_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        unsafe { write_unaligned(field, native_state) }
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn connect_to_server(
        self,
        address: &[u8],
        port: u16,
    ) -> Result<(), DirectClientError> {
        if address.is_empty()
            || address.len() >= self.spec.net_game.host_string_capacity.get()
            || address.contains(&0)
            || port == 0
        {
            return Err(DirectClientError::NotReady);
        }
        let native_state = self
            .spec
            .strategies
            .game_state_codec
            .encode(9)
            .ok_or(DirectClientError::NotReady)?;
        let net_game = self.net_game()? as usize;
        let host = net_game
            .checked_add(self.spec.net_game.host_address_offset.get())
            .ok_or(DirectClientError::NotReady)? as *mut u8;
        let port_field = net_game
            .checked_add(self.spec.net_game.port_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let state_field = net_game
            .checked_add(self.spec.net_game.game_state_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let capacity = self.spec.net_game.host_string_capacity.get();
        if !writable_range(host.cast_const(), capacity)
            || !writable_range(port_field as *const u8, mem::size_of::<i32>())
            || !writable_range(state_field as *const u8, mem::size_of::<i32>())
        {
            return Err(DirectClientError::NotReady);
        }
        if !unsafe { zero_bytes(host, capacity) }
            || !unsafe { copy_bytes(host, address) }
            || !unsafe { write_unaligned(port_field, i32::from(port)) }
            || !unsafe { write_unaligned(state_field, native_state) }
        {
            return Err(DirectClientError::NotReady);
        }
        Ok(())
    }

    pub(crate) fn disconnect_with_reason(
        self,
        rak_client: *mut c_void,
        block_duration: u32,
    ) -> Result<(), DirectClientError> {
        if rak_client.is_null() {
            return Err(DirectClientError::NotReady);
        }
        let vtable = unsafe { read_pointer(rak_client as usize) }
            .filter(|pointer| !pointer.is_null())
            .ok_or(DirectClientError::NotReady)?;
        let slot = self
            .spec
            .net_game
            .rak_client_disconnect_vtable_slot
            .checked_mul(mem::size_of::<usize>())
            .ok_or(DirectClientError::NotReady)?;
        let function_field = (vtable as usize)
            .checked_add(slot)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(vtable, slot + mem::size_of::<usize>()) {
            return Err(DirectClientError::NotReady);
        }
        let function = unsafe { read_pointer(function_field) }
            .filter(|pointer| !pointer.is_null() && readable_range(pointer.cast(), 1))
            .ok_or(DirectClientError::NotReady)?;
        let disconnect: ProfileRakClientDisconnectFn = unsafe { mem::transmute(function) };
        unsafe { disconnect(rak_client, block_duration, 0) };

        let net_game = self.net_game()?;
        let shutdown = self
            .module_base
            .checked_add(self.spec.net_game.shutdown_for_restart_rva.get())
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)?;
        let shutdown: ProfileNetGameShutdownFn = unsafe { mem::transmute(shutdown) };
        unsafe { shutdown(net_game) };
        Ok(())
    }

    pub(crate) fn server_info(self) -> Result<ServerInfoSnapshot, DirectClientError> {
        let net_game = self.net_game()? as usize;
        let address = unsafe {
            bounded_c_string(
                net_game
                    .checked_add(self.spec.net_game.host_address_offset.get())
                    .ok_or(DirectClientError::NotReady)? as *const u8,
                self.spec.net_game.host_string_capacity.get(),
            )
        }
        .filter(|value| !value.is_empty())
        .ok_or(DirectClientError::NotReady)?;
        let hostname = unsafe {
            bounded_c_string(
                net_game
                    .checked_add(self.spec.net_game.hostname_offset.get())
                    .ok_or(DirectClientError::NotReady)? as *const u8,
                self.spec.net_game.host_string_capacity.get(),
            )
        }
        .filter(|value| !value.is_empty())
        .ok_or(DirectClientError::NotReady)?;
        let port = unsafe {
            read_unaligned::<i32>(
                net_game
                    .checked_add(self.spec.net_game.port_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .and_then(|value| u16::try_from(value).ok())
        .filter(|value| *value != 0)
        .ok_or(DirectClientError::NotReady)?;
        Ok(ServerInfoSnapshot {
            address,
            hostname,
            port,
        })
    }

    fn net_game(self) -> Result<*mut c_void, DirectClientError> {
        let singleton = self
            .module_base
            .checked_add(self.spec.net_game.singleton_rva.get())
            .ok_or(DirectClientError::NotReady)?;
        let net_game: *mut c_void = unsafe { read_pointer(singleton) }
            .filter(|pointer| !pointer.is_null())
            .ok_or(DirectClientError::NotReady)?
            .cast();
        let minimum_size = self
            .spec
            .net_game
            .hostname_offset
            .get()
            .checked_add(self.spec.net_game.host_string_capacity.get())
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(net_game.cast(), minimum_size) {
            return Err(DirectClientError::NotReady);
        }
        Ok(net_game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;

    fn selected_profile(
        version: SampVersion,
        entry_point: u32,
    ) -> (NativeClientProfile, Vec<u8>, Vec<u8>) {
        let mut module = vec![0_u8; 0x2A_CA_24 + mem::size_of::<usize>()];
        let profile =
            NativeClientProfile::select(module.as_mut_ptr() as usize, version, entry_point)
                .expect("supported profile");
        let mut net_game = vec![0_u8; 0x500];
        unsafe {
            (module
                .as_mut_ptr()
                .add(profile.spec.net_game.singleton_rva.get())
                .cast::<usize>())
            .write_unaligned(net_game.as_mut_ptr() as usize);
        }
        (profile, module, net_game)
    }

    fn profiles() -> [(SampVersion, u32); 4] {
        [
            (SampVersion::R1, 0x31DF13),
            (SampVersion::R3_1, 0x0CC4D0),
            (SampVersion::R5_1, 0x0CBC90),
            (SampVersion::Dl, 0x0FDB60),
        ]
    }

    #[test]
    fn game_state_codecs_preserve_the_public_contract() {
        for state in [0, 9, 13, 14, 15, 18] {
            assert_eq!(GameStateCodec::Identity.decode(state), Some(state));
            assert_eq!(GameStateCodec::Identity.encode(state), Some(state));
        }
        for (public, native) in [(0, 0), (9, 1), (13, 2), (14, 5), (15, 6), (18, 11)] {
            assert_eq!(GameStateCodec::Classic.encode(public), Some(native));
            assert_eq!(GameStateCodec::Classic.decode(native), Some(public));
        }
        assert_eq!(GameStateCodec::Classic.encode(3), None);
        assert_eq!(GameStateCodec::Classic.decode(3), None);
    }

    #[test]
    fn server_metadata_has_identical_owned_result_rules_for_every_profile() {
        for (version, entry_point) in profiles() {
            let (profile, _module, mut net_game) = selected_profile(version, entry_point);
            let spec = profile.spec.net_game;
            net_game[spec.host_address_offset.get()..][..9].copy_from_slice(b"127.0.0.1");
            net_game[spec.hostname_offset.get()..][..11].copy_from_slice(b"test server");
            unsafe {
                (net_game
                    .as_mut_ptr()
                    .add(spec.port_offset.get())
                    .cast::<i32>())
                .write_unaligned(7777);
            }

            assert_eq!(
                profile.server_info(),
                Ok(ServerInfoSnapshot {
                    address: b"127.0.0.1".to_vec(),
                    hostname: b"test server".to_vec(),
                    port: 7777,
                })
            );

            net_game[spec.hostname_offset.get()] = 0;
            assert_eq!(profile.server_info(), Err(DirectClientError::NotReady));
        }
    }

    #[test]
    fn reconnect_validates_every_field_before_mutating_each_profile() {
        for (version, entry_point) in profiles() {
            let (profile, _module, mut net_game) = selected_profile(version, entry_point);
            let spec = profile.spec.net_game;
            let host = spec.host_address_offset.get();
            let port = spec.port_offset.get();
            let state = spec.game_state_offset.get();
            net_game[host..host + 4].copy_from_slice(b"old\0");
            unsafe {
                (net_game.as_mut_ptr().add(port).cast::<i32>()).write_unaligned(7777);
                (net_game.as_mut_ptr().add(state).cast::<i32>()).write_unaligned(0);
            }

            assert_eq!(
                profile.connect_to_server(b"bad\0address", 7778),
                Err(DirectClientError::NotReady)
            );
            assert_eq!(&net_game[host..host + 4], b"old\0");
            assert_eq!(
                unsafe { (net_game.as_ptr().add(port).cast::<i32>()).read_unaligned() },
                7777
            );

            profile.connect_to_server(b"new.example", 7778).unwrap();
            assert_eq!(&net_game[host..host + 12], b"new.example\0");
            assert_eq!(
                unsafe { (net_game.as_ptr().add(port).cast::<i32>()).read_unaligned() },
                7778
            );
            assert_eq!(
                unsafe { (net_game.as_ptr().add(state).cast::<i32>()).read_unaligned() },
                profile.spec.strategies.game_state_codec.encode(9).unwrap()
            );
        }
    }

    #[test]
    fn rakpeer_resolution_uses_each_profile_handle_size() {
        for (version, entry_point) in profiles() {
            let (profile, _module, _net_game) = selected_profile(version, entry_point);
            let size = profile.spec.handles.rakpeer_size.get();
            let mut peer = vec![0_u8; size + 1];
            let rak_client = unsafe { peer.as_mut_ptr().add(size) }.cast();
            assert_eq!(
                profile.rakpeer_address(rak_client),
                Ok(peer.as_mut_ptr().cast())
            );
            assert_eq!(
                profile.rakpeer_address(std::ptr::null_mut()),
                Err(DirectClientError::NotReady)
            );
        }
    }
}
