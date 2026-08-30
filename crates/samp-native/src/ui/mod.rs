//! Guarded UI cache reads shared by immutable client profiles.

mod chat;
mod dialog;
mod display;
mod input;

use super::{
    memory::{
        bounded_c_string, read_i32_bool, read_pointer, read_unaligned, readable_range,
        writable_range, write_unaligned,
    },
    profile::{ListItemTextLayout, NativeProfile, PoolGetterAbi},
};
use crate::{
    ChatEntrySnapshot, DirectClientError, LocalChatMessageRequest, LocalDeathMessageRequest,
    LocalDialogRequest, LocalDialogResponseSnapshot, LocalDialogSnapshot, LocalDialogStyle,
};
use std::{ffi::c_void, mem};

type R1DxutEditBoxGetTextFn = unsafe extern "thiscall" fn(*mut c_void) -> *const u8;
type ClassicDxutEditBoxGetTextFn = unsafe extern "thiscall" fn(*mut c_void) -> *const u8;
type R1DxutEditBoxSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const i8, bool);
type ClassicDxutEditBoxSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const i8, bool);

impl NativeProfile {
    pub(super) fn set_editbox_text(
        self,
        editbox: *mut u8,
        rva: Option<super::profile::NativeRva>,
        text: &[u8],
        maximum: usize,
    ) -> Result<(), DirectClientError> {
        if text.len() > maximum
            || text.contains(&0)
            || editbox.is_null()
            || !readable_range(editbox, 1)
        {
            return Err(DirectClientError::NotReady);
        }
        let target = self.ui_target(rva.ok_or(DirectClientError::NotReady)?)?;
        let mut text = text.to_vec();
        text.push(0);
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let set_text: R1DxutEditBoxSetTextFn = mem::transmute(target);
                    set_text(editbox.cast(), text.as_ptr().cast(), false);
                }
                PoolGetterAbi::Classic => {
                    let set_text: ClassicDxutEditBoxSetTextFn = mem::transmute(target);
                    set_text(editbox.cast(), text.as_ptr().cast(), false);
                }
            }
        }
        Ok(())
    }

    pub(super) fn ui_target(
        self,
        rva: super::profile::NativeRva,
    ) -> Result<usize, DirectClientError> {
        self.module_base
            .checked_add(rva.get())
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;

    #[test]
    fn ui_scalars_reject_invalid_values_for_every_profile() {
        for version in [
            SampVersion::R1,
            SampVersion::R3_1,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            let profile = NativeProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            assert_eq!(
                profile.chat_entry(u16::MAX),
                Err(DirectClientError::NotReady)
            );
            assert_eq!(
                profile.set_chat_entry(0, b"text\0", b"prefix", 0, 0),
                Err(DirectClientError::NotReady)
            );
            assert_eq!(
                profile.set_chat_display_mode(3),
                Err(DirectClientError::NotReady)
            );
        }
    }

    #[test]
    fn ui_specs_keep_verified_profile_behavior_boundaries() {
        let r1 = NativeProfile::select(0x10000, SampVersion::R1, SampVersion::R1.entry_point())
            .expect("the R1 identity must select");
        let r3 = NativeProfile::select(0x10000, SampVersion::R3_1, SampVersion::R3_1.entry_point())
            .expect("the R3 identity must select");
        let r5 = NativeProfile::select(0x10000, SampVersion::R5_1, SampVersion::R5_1.entry_point())
            .expect("the R5 identity must select");
        let dl = NativeProfile::select(0x10000, SampVersion::Dl, SampVersion::Dl.entry_point())
            .expect("the DL identity must select");

        assert_eq!(
            r1.spec.strategies.list_item_text_layout,
            ListItemTextLayout::DxutComboBoxItem
        );
        for profile in [r3, r5, dl] {
            assert_eq!(
                profile.spec.strategies.list_item_text_layout,
                ListItemTextLayout::DirectPointer
            );
        }
        assert_eq!(
            r3.spec
                .ui
                .input
                .edit_box_set_text_rva
                .map(|value| value.get()),
            Some(0x84E70)
        );
        assert_eq!(
            r3.spec
                .ui
                .input
                .edit_box_get_text_rva
                .map(|value| value.get()),
            Some(0x84F40)
        );
        assert_eq!(dl.spec.ui.dialog.show_rva.get(), 0x6FA50);
    }
}
