//! Cached local animation-catalog `HostApi` wrappers.

use crate::{
    HostApi, LocalAnimation, SampClientSdkAnimationV1, SampClientSdkResult,
    local_animation_from_abi, valid_bounded_bytes,
};

impl HostApi {
    /// Returns an owned entry from the cached R1 animation table.
    pub fn local_animation(self, id: u16) -> Result<LocalAnimation, SampClientSdkResult> {
        let mut raw = SampClientSdkAnimationV1::default();
        match unsafe { (self.raw.local_animation)(id, &mut raw) } {
            SampClientSdkResult::Ok => {
                local_animation_from_abi(raw).ok_or(SampClientSdkResult::NativeCallFailed)
            }
            result => Err(result),
        }
    }

    /// Finds a cached R1 animation-table entry by its name and file bytes.
    ///
    /// Returns `Ok(None)` when no entry matches either byte string.
    pub fn local_animation_id(
        self,
        name: &[u8],
        file: &[u8],
    ) -> Result<Option<u16>, SampClientSdkResult> {
        if !valid_bounded_bytes(name, 35) || !valid_bounded_bytes(file, 35) {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut id = -1;
        match unsafe {
            (self.raw.local_animation_id)(
                name.as_ptr(),
                name.len(),
                file.as_ptr(),
                file.len(),
                &mut id,
            )
        } {
            SampClientSdkResult::Ok => match id {
                -1 => Ok(None),
                0..=65_535 => Ok(Some(id as u16)),
                _ => Err(SampClientSdkResult::NativeCallFailed),
            },
            result => Err(result),
        }
    }
}
