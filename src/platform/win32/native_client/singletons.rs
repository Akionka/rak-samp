//! Guarded singleton resolution shared by immutable client profiles.

use super::{
    memory::{read_pointer, readable_range},
    profile::{NativeClientProfile, NativeRva},
};
use std::ffi::c_void;

impl NativeClientProfile {
    pub(crate) fn singleton(self, rva: NativeRva, minimum_size: usize) -> Option<*mut c_void> {
        let address = self.module_base.checked_add(rva.get())?;
        let pointer: *mut c_void = unsafe { read_pointer(address) }?.cast();
        (!pointer.is_null() && readable_range(pointer.cast(), minimum_size)).then_some(pointer)
    }

    pub(crate) fn dialog_close_target(self) -> Option<usize> {
        self.module_base
            .checked_add(self.spec.ui.dialog.close_rva.get())
    }

    pub(crate) fn net_game_with_range(self, minimum_size: usize) -> Option<*mut c_void> {
        self.singleton(self.spec.net_game.singleton_rva, minimum_size)
    }

    pub(crate) fn dialog(self) -> Option<*mut c_void> {
        self.singleton(
            self.spec.ui.dialog.singleton_rva,
            self.spec.ui.dialog.active_offset.get().checked_add(4)?,
        )
    }

    pub(crate) fn chat(self) -> Option<*mut c_void> {
        self.singleton(self.spec.ui.chat.singleton_rva, 1)
    }

    pub(crate) fn input(self) -> Option<*mut c_void> {
        self.singleton(
            self.spec.ui.input.singleton_rva,
            self.spec.ui.input.enabled_offset.get().checked_add(4)?,
        )
    }

    pub(crate) fn scoreboard(self) -> Option<*mut c_void> {
        self.singleton(
            self.spec.ui.scoreboard.singleton_rva,
            self.spec.ui.scoreboard.readable_size.get(),
        )
    }

    pub(crate) fn death_window(self) -> Option<*mut c_void> {
        self.singleton(self.spec.ui.death_window.singleton_rva?, 1)
    }

    pub(crate) fn game(self) -> Option<*mut c_void> {
        self.singleton(
            self.spec.ui.game.singleton_rva,
            self.spec.ui.game.cursor_mode_offset.get().checked_add(4)?,
        )
    }

    pub(crate) fn dialog_is_ready(self) -> bool {
        self.dialog().is_some()
    }

    pub(crate) fn chat_is_ready(self) -> bool {
        self.chat().is_some()
    }

    pub(crate) fn input_is_ready(self) -> bool {
        self.input().is_some()
    }

    pub(crate) fn scoreboard_is_ready(self) -> bool {
        self.scoreboard().is_some()
    }

    pub(crate) fn death_window_is_ready(self) -> bool {
        self.death_window().is_some()
    }

    pub(crate) fn game_is_ready(self) -> bool {
        self.game().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;
    use std::ptr;
    use windows_sys::Win32::System::Memory::{
        MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_NOACCESS, VirtualAlloc, VirtualFree,
    };

    #[test]
    fn resolves_only_non_null_singletons_with_the_required_range() {
        let mut module = vec![0_u8; 0x2A_CA_24 + std::mem::size_of::<usize>()];
        let profile = NativeClientProfile::select(
            module.as_mut_ptr() as usize,
            SampVersion::Dl,
            SampVersion::Dl.entry_point(),
        )
        .unwrap();
        let mut dialog = vec![0_u8; profile.spec.ui.dialog.active_offset.get() + 4];
        unsafe {
            (module
                .as_mut_ptr()
                .add(profile.spec.ui.dialog.singleton_rva.get())
                .cast::<usize>())
            .write_unaligned(dialog.as_mut_ptr() as usize);
        }
        assert_eq!(profile.dialog(), Some(dialog.as_mut_ptr().cast()));
        unsafe {
            (module
                .as_mut_ptr()
                .add(profile.spec.ui.dialog.singleton_rva.get())
                .cast::<usize>())
            .write_unaligned(0);
        }
        assert_eq!(profile.dialog(), None);
    }

    #[test]
    fn r3_death_window_requires_a_readable_singleton() {
        let mut module = vec![0_u8; 0x2A_CA_24 + std::mem::size_of::<usize>()];
        let profile = NativeClientProfile::select(
            module.as_mut_ptr() as usize,
            SampVersion::R3_1,
            SampVersion::R3_1.entry_point(),
        )
        .unwrap();
        let unreadable =
            unsafe { VirtualAlloc(ptr::null(), 0x1000, MEM_COMMIT | MEM_RESERVE, PAGE_NOACCESS) };
        assert!(!unreadable.is_null(), "VirtualAlloc failed");
        unsafe {
            module
                .as_mut_ptr()
                .add(profile.spec.ui.death_window.singleton_rva.unwrap().get())
                .cast::<usize>()
                .write_unaligned(unreadable as usize);
        }

        assert_eq!(profile.death_window(), None);
        assert_ne!(unsafe { VirtualFree(unreadable, 0, MEM_RELEASE) }, 0);
    }

    #[test]
    fn readiness_uses_the_profile_singletons() {
        for version in [
            SampVersion::R1,
            SampVersion::R3_1,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            let mut module = vec![0_u8; 0x2A_CA_24 + std::mem::size_of::<usize>()];
            let profile = NativeClientProfile::select(
                module.as_mut_ptr() as usize,
                version,
                version.entry_point(),
            )
            .unwrap();
            let mut dialog = vec![0_u8; 0x1600];
            let mut chat = vec![0_u8; 0x1600];
            let mut input = vec![0_u8; 0x1600];
            let mut scoreboard = vec![0_u8; 0x1600];
            let mut game = vec![0_u8; 0x1600];
            let mut death_window = vec![0_u8; 0x1600];

            unsafe {
                (module
                    .as_mut_ptr()
                    .add(profile.spec.ui.dialog.singleton_rva.get())
                    .cast::<usize>())
                .write_unaligned(dialog.as_mut_ptr() as usize);
                (module
                    .as_mut_ptr()
                    .add(profile.spec.ui.chat.singleton_rva.get())
                    .cast::<usize>())
                .write_unaligned(chat.as_mut_ptr() as usize);
                (module
                    .as_mut_ptr()
                    .add(profile.spec.ui.input.singleton_rva.get())
                    .cast::<usize>())
                .write_unaligned(input.as_mut_ptr() as usize);
                (module
                    .as_mut_ptr()
                    .add(profile.spec.ui.scoreboard.singleton_rva.get())
                    .cast::<usize>())
                .write_unaligned(scoreboard.as_mut_ptr() as usize);
                (module
                    .as_mut_ptr()
                    .add(profile.spec.ui.game.singleton_rva.get())
                    .cast::<usize>())
                .write_unaligned(game.as_mut_ptr() as usize);
                if let Some(rva) = profile.spec.ui.death_window.singleton_rva {
                    (module.as_mut_ptr().add(rva.get()).cast::<usize>())
                        .write_unaligned(death_window.as_mut_ptr() as usize);
                }
            }

            assert!(profile.dialog_is_ready());
            assert!(profile.chat_is_ready());
            assert!(profile.input_is_ready());
            assert!(profile.scoreboard_is_ready());
            assert!(profile.game_is_ready());
            assert_eq!(
                profile.death_window_is_ready(),
                profile.spec.ui.death_window.singleton_rva.is_some(),
            );
        }
    }

    #[test]
    fn rejects_ranges_that_cannot_be_read() {
        let mut module = vec![0_u8; 0x2A_CA_24 + std::mem::size_of::<usize>()];
        let profile = NativeClientProfile::select(
            module.as_mut_ptr() as usize,
            SampVersion::Dl,
            SampVersion::Dl.entry_point(),
        )
        .unwrap();
        let mut dialog = vec![0_u8; 1];
        unsafe {
            (module
                .as_mut_ptr()
                .add(profile.spec.ui.dialog.singleton_rva.get())
                .cast::<usize>())
            .write_unaligned(dialog.as_mut_ptr() as usize);
        }
        assert_eq!(
            profile.singleton(profile.spec.ui.dialog.singleton_rva, usize::MAX),
            None
        );
    }
}
