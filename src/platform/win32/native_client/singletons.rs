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
            self.spec
                .ui
                .scoreboard
                .enabled_offset
                .get()
                .checked_add(4)?,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;

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
    fn r3_death_window_stays_unavailable() {
        let profile = NativeClientProfile::select(
            0x10000,
            SampVersion::R3_1,
            SampVersion::R3_1.entry_point(),
        )
        .unwrap();
        assert_eq!(profile.death_window(), None);
    }
}
