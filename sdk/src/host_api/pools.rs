use crate::{
    ChatEntry, Gangzone, HostApi, MAX_SAMP_CHAT_ENTRIES, MAX_SAMP_GANGZONES, MAX_SAMP_OBJECTS,
    MAX_SAMP_TEXT_LABELS, MAX_SAMP_TEXTDRAWS, MAX_SAMP_VEHICLES, SampClientSdkChatEntryV1,
    SampClientSdkGangzoneV1, SampClientSdkResult, SampClientSdkTextDrawV1,
    SampClientSdkTextLabelV1, TextDraw, TextLabel, chat_entry_from_abi, gangzone_from_abi,
    text_label_from_abi, textdraw_from_abi,
};

impl HostApi {
    /// Returns whether the latest cached R1 vehicle-pool result has `id` defined.
    pub fn is_vehicle_defined(self, id: u16) -> Result<bool, SampClientSdkResult> {
        if id >= MAX_SAMP_VEHICLES {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut exists = 0;
        match unsafe { (self.raw.vehicle_exists)(id, &mut exists) } {
            SampClientSdkResult::Ok => match exists {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(SampClientSdkResult::NativeCallFailed),
            },
            result => Err(result),
        }
    }
    /// Returns whether the latest cached R1 3D text-label-pool result has `id` defined.
    pub fn is_text_label_defined(self, id: u16) -> Result<bool, SampClientSdkResult> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut exists = 0;
        match unsafe { (self.raw.text_label_exists)(id, &mut exists) } {
            SampClientSdkResult::Ok => match exists {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(SampClientSdkResult::NativeCallFailed),
            },
            result => Err(result),
        }
    }
    /// Returns whether the latest cached R1 textdraw-pool result has `pool_index` defined.
    pub fn is_textdraw_defined(self, pool_index: u16) -> Result<bool, SampClientSdkResult> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut exists = 0;
        match unsafe { (self.raw.textdraw_exists)(pool_index, &mut exists) } {
            SampClientSdkResult::Ok => match exists {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(SampClientSdkResult::NativeCallFailed),
            },
            result => Err(result),
        }
    }
    /// Returns whether the latest cached R1 object-pool result has `id` defined.
    pub fn is_object_defined(self, id: u16) -> Result<bool, SampClientSdkResult> {
        if id >= MAX_SAMP_OBJECTS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut exists = 0;
        match unsafe { (self.raw.object_exists)(id, &mut exists) } {
            SampClientSdkResult::Ok => match exists {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(SampClientSdkResult::NativeCallFailed),
            },
            result => Err(result),
        }
    }
    /// Returns the latest cached R1 gangzone record for `id`.
    pub fn gangzone(self, id: u16) -> Result<Option<Gangzone>, SampClientSdkResult> {
        if id >= MAX_SAMP_GANGZONES {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkGangzoneV1::default();
        match unsafe { (self.raw.gangzone_info)(id, &mut raw) } {
            SampClientSdkResult::Ok => gangzone_from_abi(raw),
            result => Err(result),
        }
    }
    /// Returns the latest cached R1 3D text-label record for `id`.
    pub fn text_label(self, id: u16) -> Result<Option<TextLabel>, SampClientSdkResult> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkTextLabelV1::default();
        match unsafe { (self.raw.text_label_info)(id, &mut raw) } {
            SampClientSdkResult::Ok => text_label_from_abi(raw),
            result => Err(result),
        }
    }
    /// Returns the latest cached R1 numeric textdraw record for `pool_index`.
    pub fn textdraw(self, pool_index: u16) -> Result<Option<TextDraw>, SampClientSdkResult> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkTextDrawV1::default();
        match unsafe { (self.raw.textdraw_info)(pool_index, &mut raw) } {
            SampClientSdkResult::Ok => textdraw_from_abi(raw),
            result => Err(result),
        }
    }
    /// Returns one latest cached fixed R1 chat-history entry.
    pub fn chat_entry(self, id: u16) -> Result<ChatEntry, SampClientSdkResult> {
        if id >= MAX_SAMP_CHAT_ENTRIES {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkChatEntryV1::default();
        match unsafe { (self.raw.chat_entry_info)(id, &mut raw) } {
            SampClientSdkResult::Ok => chat_entry_from_abi(raw),
            result => Err(result),
        }
    }
}
