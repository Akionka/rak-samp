use super::{
    R1ClientProfile,
    addresses::{
        CHAT_SINGLETON_RVA, DEATH_WINDOW_SINGLETON_RVA, DIALOG_SINGLETON_RVA, INPUT_SINGLETON_RVA,
        SCOREBOARD_SINGLETON_RVA,
    },
    memory::{INPUT_ENABLED_OFFSET, read_pointer, readable_range},
};
use std::ffi::c_void;

impl R1ClientProfile {
    /// Resolves a singleton pointer and verifies the caller's minimum readable range.
    fn singleton(self, rva: usize, minimum_length: usize) -> Option<*mut c_void> {
        let address = self.module_base.checked_add(rva)?;
        let singleton: *mut c_void = unsafe { read_pointer(address) }?.cast();
        (!singleton.is_null() && readable_range(singleton.cast(), minimum_length))
            .then_some(singleton)
    }

    pub(super) fn dialog(self) -> Option<*mut c_void> {
        self.singleton(DIALOG_SINGLETON_RVA, 1)
    }

    pub(super) fn chat(self) -> Option<*mut c_void> {
        self.singleton(CHAT_SINGLETON_RVA, 1)
    }

    pub(super) fn scoreboard(self) -> Option<*mut c_void> {
        self.singleton(SCOREBOARD_SINGLETON_RVA, 4)
    }

    pub(super) fn input(self) -> Option<*mut c_void> {
        self.singleton(INPUT_SINGLETON_RVA, INPUT_ENABLED_OFFSET + 4)
    }

    pub(super) fn death_window(self) -> Option<*mut c_void> {
        self.singleton(DEATH_WINDOW_SINGLETON_RVA, 1)
    }
}
