//! Safe host-table wrapper, command receipts, and ABI conversions.

use super::*;

/// A validated reference to the host API table.
#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct HostApi {
    pub(crate) raw: &'static SampClientSdkApiV1,
}

/// One owned completion receipt for a game-thread command.
///
/// Polling and waiting consume the receipt once the command completes. Dropping
/// a pending receipt releases only the waiter; the host still owns and executes
/// the copied command on a later game tick.
pub struct CommandReceipt<T> {
    api: HostApi,
    raw: SampClientSdkCommandReceipt,
    decode: fn(SampClientSdkCommandResultV1) -> Result<T, SampClientSdkResult>,
    active: bool,
}

impl<T> CommandReceipt<T> {
    pub(crate) fn new(
        api: HostApi,
        raw: SampClientSdkCommandReceipt,
        decode: fn(SampClientSdkCommandResultV1) -> Result<T, SampClientSdkResult>,
    ) -> Self {
        Self {
            api,
            raw,
            decode,
            active: true,
        }
    }

    /// Returns the host-owned opaque command identity.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.raw.id
    }

    /// Consumes and returns a ready completion, or returns `Ok(None)` while it
    /// remains pending. Completion failures are returned as SDK result codes.
    pub fn try_take(&mut self) -> Result<Option<T>, SampClientSdkResult> {
        if !self.active {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut output = SampClientSdkCommandResultV1::default();
        match unsafe { (self.api.raw.command_try_take)(self.raw, &mut output) } {
            SampClientSdkResult::Ok => {
                self.active = false;
                (self.decode)(output).map(Some)
            }
            SampClientSdkResult::CommandPending => Ok(None),
            error => Err(error),
        }
    }

    /// Waits for and consumes the completion.
    ///
    /// `TimedOut` leaves this receipt usable for another poll or wait. The host
    /// rejects waits from a listener callback and, once enabled, the game thread.
    pub fn wait(&mut self, timeout: Duration) -> Result<T, SampClientSdkResult> {
        if !self.active {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let timeout_ms = timeout.as_millis().min(u128::from(u32::MAX)) as u32;
        let mut output = SampClientSdkCommandResultV1::default();
        match unsafe { (self.api.raw.command_wait)(self.raw, timeout_ms, &mut output) } {
            SampClientSdkResult::Ok => {
                self.active = false;
                (self.decode)(output)
            }
            error => Err(error),
        }
    }

    /// Detaches this waiter without cancelling the copied native command.
    pub fn release(mut self) -> Result<(), SampClientSdkResult> {
        if !self.active {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        match unsafe { (self.api.raw.command_release)(self.raw) } {
            SampClientSdkResult::Ok => {
                self.active = false;
                Ok(())
            }
            error => Err(error),
        }
    }
}

impl<T> Drop for CommandReceipt<T> {
    fn drop(&mut self) {
        if self.active {
            let _ = unsafe { (self.api.raw.command_release)(self.raw) };
            self.active = false;
        }
    }
}

/// An owned typed completion receipt for queued R1 3D text-label creation.
///
/// Polling and waiting consume the receipt once creation either yields a
/// checked [`crate::TextLabelId`] or reports a native failure. Dropping a
/// pending receipt detaches only the caller; it does not cancel creation.
#[must_use = "a text-label creation receipt owns one pending completion"]
pub struct TextLabelCreateReceipt {
    api: HostApi,
    raw: SampClientSdkCommandReceipt,
    active: bool,
}

impl TextLabelCreateReceipt {
    pub(crate) fn new(api: HostApi, raw: SampClientSdkCommandReceipt) -> Self {
        Self {
            api,
            raw,
            active: true,
        }
    }

    /// Returns the host-owned opaque command identity.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.raw.id
    }

    /// Consumes and returns the created label ID, or reports `Ok(None)` while
    /// creation remains pending.
    pub fn try_take(&mut self) -> Result<Option<crate::TextLabelId>, SampClientSdkResult> {
        if !self.active {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut output = SampClientSdkTextLabelCreateResultV1::default();
        match unsafe { (self.api.raw.text_label_create_try_take)(self.raw, &mut output) } {
            SampClientSdkResult::Ok => {
                self.active = false;
                text_label_create_result(output).map(Some)
            }
            SampClientSdkResult::CommandPending => Ok(None),
            error => Err(error),
        }
    }

    /// Waits for and consumes the created label ID.
    ///
    /// `TimedOut` leaves this receipt usable for another poll or wait. The
    /// host rejects waits from callbacks and from the game thread.
    pub fn wait(&mut self, timeout: Duration) -> Result<crate::TextLabelId, SampClientSdkResult> {
        if !self.active {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let timeout_ms = timeout.as_millis().min(u128::from(u32::MAX)) as u32;
        let mut output = SampClientSdkTextLabelCreateResultV1::default();
        match unsafe { (self.api.raw.text_label_create_wait)(self.raw, timeout_ms, &mut output) } {
            SampClientSdkResult::Ok => {
                self.active = false;
                text_label_create_result(output)
            }
            error => Err(error),
        }
    }

    /// Detaches this waiter without cancelling the copied native creation.
    pub fn release(mut self) -> Result<(), SampClientSdkResult> {
        if !self.active {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        match unsafe { (self.api.raw.command_release)(self.raw) } {
            SampClientSdkResult::Ok => {
                self.active = false;
                Ok(())
            }
            error => Err(error),
        }
    }
}

impl Drop for TextLabelCreateReceipt {
    fn drop(&mut self) {
        if self.active {
            let _ = unsafe { (self.api.raw.command_release)(self.raw) };
            self.active = false;
        }
    }
}

fn text_label_create_result(
    result: SampClientSdkTextLabelCreateResultV1,
) -> Result<crate::TextLabelId, SampClientSdkResult> {
    match result.status {
        SampClientSdkResult::Ok => {
            crate::TextLabelId::new(result.id).ok_or(SampClientSdkResult::NativeCallFailed)
        }
        error => Err(error),
    }
}

impl HostApi {
    pub(crate) fn command_receipt(
        self,
        result: SampClientSdkResult,
        receipt: SampClientSdkCommandReceipt,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        match result {
            SampClientSdkResult::Ok if receipt.id != 0 => {
                Ok(CommandReceipt::new(self, receipt, unit_command_result))
            }
            SampClientSdkResult::Ok => Err(SampClientSdkResult::NativeCallFailed),
            error => Err(error),
        }
    }
}

pub(crate) fn unit_command_result(
    result: SampClientSdkCommandResultV1,
) -> Result<(), SampClientSdkResult> {
    match result.status {
        SampClientSdkResult::Ok => Ok(()),
        error => Err(error),
    }
}

impl HostApi {
    /// # Safety
    ///
    /// `raw` must point to a live API table exported by a compatible host.
    pub(crate) unsafe fn from_raw(raw: *const SampClientSdkApiV1) -> Result<Self, ResolveError> {
        let raw = NonNull::new(raw.cast_mut()).ok_or(ResolveError::MissingApi)?;
        let raw = unsafe { raw.as_ref() };
        if raw.abi_version != ABI_VERSION_V1
            || raw.size < mem::size_of::<SampClientSdkApiV1>() as u32
        {
            return Err(ResolveError::UnsupportedAbi);
        }
        Ok(Self { raw })
    }

    #[must_use]
    pub(crate) fn raw(self) -> &'static SampClientSdkApiV1 {
        self.raw
    }

    /// Queues one validated R1 CNetGame-state write and returns its completion receipt.
    pub fn submit_samp_game_state(
        self,
        state: SampGameState,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_samp_game_state)(state.raw(), &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues one R1 replication send-rate write in milliseconds.
    pub fn submit_send_rate(
        self,
        kind: SendRateKind,
        milliseconds: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_send_rate)(kind.raw(), milliseconds, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues one documented R1 unoccupied-vehicle synchronization send.
    pub fn submit_force_unoccupied_sync(
        self,
        vehicle: u16,
        seat: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_force_unoccupied_sync)(vehicle, seat, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues one bounded R1 chat-history entry replacement.
    pub fn submit_local_chat_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_local_chat_entry)(
                id,
                text.as_ptr(),
                text.len(),
                prefix.as_ptr(),
                prefix.len(),
                text_colour,
                prefix_colour,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }
}

pub(crate) fn valid_bounded_bytes(value: &[u8], maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && !value.contains(&0)
}

pub(crate) fn local_dialog_state_from_abi(
    raw: SampClientSdkDialogSnapshotV1,
) -> Result<Option<LocalDialogState>, SampClientSdkResult> {
    match raw.active {
        0 if raw == SampClientSdkDialogSnapshotV1::default() => Ok(None),
        1 => {
            let Some(style) = LocalDialogStyle::from_raw(raw.style) else {
                return Err(SampClientSdkResult::NativeCallFailed);
            };
            let server_side = match raw.server_side {
                0 => false,
                1 => true,
                _ => return Err(SampClientSdkResult::NativeCallFailed),
            };
            let title_len = usize::from(raw.title_len);
            let text_len = usize::from(raw.text_len);
            let editbox_text_len = usize::from(raw.editbox_text_len);
            let listbox_item_count = usize::from(raw.listbox_item_count);
            if title_len > raw.title.len()
                || text_len > raw.text.len()
                || editbox_text_len > raw.editbox_text.len()
                || listbox_item_count > raw.listbox_items.len()
            {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            let editbox_text = match raw.has_editbox {
                0 if editbox_text_len == 0 => None,
                1 => Some(raw.editbox_text[..editbox_text_len].to_vec()),
                _ => return Err(SampClientSdkResult::NativeCallFailed),
            };
            let items = raw.listbox_items[..listbox_item_count]
                .iter()
                .map(|item| {
                    let len = usize::from(item.len);
                    (len <= item.bytes.len())
                        .then(|| item.bytes[..len].to_vec())
                        .ok_or(SampClientSdkResult::NativeCallFailed)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(LocalDialogState {
                id: raw.id,
                style,
                title: raw.title[..title_len].to_vec(),
                server_side,
                text: raw.text[..text_len].to_vec(),
                editbox_text,
                items,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

pub(crate) fn local_animation_from_abi(raw: SampClientSdkAnimationV1) -> Option<LocalAnimation> {
    let name_len = usize::from(raw.name_len);
    let file_len = usize::from(raw.file_len);
    if name_len == 0
        || file_len == 0
        || name_len > raw.name.len()
        || file_len > raw.file.len()
        || raw.name[..name_len].contains(&0)
        || raw.file[..file_len].contains(&0)
    {
        return None;
    }
    Some(LocalAnimation {
        name: raw.name[..name_len].to_vec(),
        file: raw.file[..file_len].to_vec(),
    })
}

pub(crate) fn player_info_from_abi(
    raw: SampClientSdkPlayerInfoV1,
) -> Result<Option<PlayerInfo>, SampClientSdkResult> {
    match raw.exists {
        0 => {
            if raw != SampClientSdkPlayerInfoV1::default() {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(None)
        }
        1 => {
            let nickname_len = usize::from(raw.nickname_len);
            if nickname_len == 0
                || nickname_len > raw.nickname.len()
                || !matches!(raw.is_local, 0 | 1)
                || !matches!(raw.is_npc, 0 | 1)
                || raw._reserved != 0
                || (raw.is_local != 0 && raw.is_npc != 0)
            {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(Some(PlayerInfo {
                id: raw.id,
                nickname: raw.nickname[..nickname_len].to_vec(),
                is_local: raw.is_local != 0,
                is_npc: raw.is_npc != 0,
                colour: raw.colour,
                score: raw.score,
                ping: raw.ping,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

pub(crate) fn remote_player_state_from_abi(
    raw: SampClientSdkRemotePlayerStateV1,
) -> Result<Option<RemotePlayerState>, SampClientSdkResult> {
    match raw.exists {
        0 if raw == SampClientSdkRemotePlayerStateV1::default() => Ok(None),
        1 if raw._reserved == 0 && raw.health.is_finite() && raw.armour.is_finite() => {
            Ok(Some(RemotePlayerState {
                id: raw.id,
                health: raw.health,
                armour: raw.armour,
                special_action: raw.special_action,
                animation_id: raw.animation_id,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

pub(crate) fn onfoot_sync_from_abi(
    raw: SampClientSdkOnFootSyncV1,
) -> Result<Option<OnFootSync>, SampClientSdkResult> {
    match raw.exists {
        0 if raw == SampClientSdkOnFootSyncV1::default() => Ok(None),
        1 if raw._reserved == [0; 3]
            && raw._reserved2 == 0
            && raw._reserved3 == 0
            && raw.position.x.is_finite()
            && raw.position.y.is_finite()
            && raw.position.z.is_finite()
            && raw.quaternion.iter().all(|value| value.is_finite())
            && raw.speed.x.is_finite()
            && raw.speed.y.is_finite()
            && raw.speed.z.is_finite()
            && raw.surfing_offset.x.is_finite()
            && raw.surfing_offset.y.is_finite()
            && raw.surfing_offset.z.is_finite() =>
        {
            Ok(Some(OnFootSync {
                id: raw.id,
                controller_left_stick_x: raw.controller_left_stick_x,
                controller_left_stick_y: raw.controller_left_stick_y,
                controller_buttons: raw.controller_buttons,
                position: raw.position,
                quaternion: raw.quaternion,
                health: raw.health,
                armour: raw.armour,
                weapon: raw.weapon,
                special_action: raw.special_action,
                speed: raw.speed,
                surfing_offset: raw.surfing_offset,
                surfing_vehicle_id: raw.surfing_vehicle_id,
                animation: raw.animation,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

pub(crate) fn vehicle_sync_from_abi(
    raw: SampClientSdkInCarSyncV1,
) -> Result<Option<InCarSync>, SampClientSdkResult> {
    match raw.exists {
        0 if raw == SampClientSdkInCarSyncV1::default() => Ok(None),
        1 if matches!(raw.siren, 0 | 1)
            && matches!(raw.landing_gear, 0 | 1)
            && raw._reserved == [0; 2]
            && raw._reserved2 == 0
            && raw.quaternion.iter().all(|value| value.is_finite())
            && raw.position.x.is_finite()
            && raw.position.y.is_finite()
            && raw.position.z.is_finite()
            && raw.speed.x.is_finite()
            && raw.speed.y.is_finite()
            && raw.speed.z.is_finite()
            && raw.vehicle_health.is_finite() =>
        {
            Ok(Some(InCarSync {
                id: raw.id,
                vehicle_id: raw.vehicle_id,
                controller_left_stick_x: raw.controller_left_stick_x,
                controller_left_stick_y: raw.controller_left_stick_y,
                controller_buttons: raw.controller_buttons,
                quaternion: raw.quaternion,
                position: raw.position,
                speed: raw.speed,
                vehicle_health: raw.vehicle_health,
                driver_health: raw.driver_health,
                driver_armour: raw.driver_armour,
                weapon: raw.weapon,
                siren: raw.siren != 0,
                landing_gear: raw.landing_gear != 0,
                trailer_id: raw.trailer_id,
                vehicle_specific: raw.vehicle_specific,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

pub(crate) fn passenger_sync_from_abi(
    raw: SampClientSdkPassengerSyncV1,
) -> Result<Option<PassengerSync>, SampClientSdkResult> {
    match raw.exists {
        0 if raw == SampClientSdkPassengerSyncV1::default() => Ok(None),
        1 if raw._reserved == [0; 3]
            && raw._reserved2 == 0
            && raw.position.x.is_finite()
            && raw.position.y.is_finite()
            && raw.position.z.is_finite() =>
        {
            Ok(Some(PassengerSync {
                id: raw.id,
                vehicle_id: raw.vehicle_id,
                seat_id: raw.seat_id,
                weapon: raw.weapon,
                health: raw.health,
                armour: raw.armour,
                controller_left_stick_x: raw.controller_left_stick_x,
                controller_left_stick_y: raw.controller_left_stick_y,
                controller_buttons: raw.controller_buttons,
                position: raw.position,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

pub(crate) fn gangzone_from_abi(
    raw: SampClientSdkGangzoneV1,
) -> Result<Option<Gangzone>, SampClientSdkResult> {
    match raw.exists {
        0 => {
            if raw != SampClientSdkGangzoneV1::default() {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(None)
        }
        1 if raw._reserved == [0; 3]
            && raw._reserved2 == 0
            && raw.left.is_finite()
            && raw.bottom.is_finite()
            && raw.right.is_finite()
            && raw.top.is_finite() =>
        {
            Ok(Some(Gangzone {
                id: raw.id,
                left: raw.left,
                bottom: raw.bottom,
                right: raw.right,
                top: raw.top,
                colour: raw.colour,
                alternate_colour: raw.alternate_colour,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

pub(crate) fn chat_entry_from_abi(
    raw: SampClientSdkChatEntryV1,
) -> Result<ChatEntry, SampClientSdkResult> {
    if raw.id >= MAX_SAMP_CHAT_ENTRIES
        || usize::from(raw.text_len) > MAX_SAMP_CHAT_ENTRY_TEXT_BYTES
        || usize::from(raw.prefix_len) > MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES
    {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    Ok(ChatEntry {
        id: raw.id,
        text: raw.text[..usize::from(raw.text_len)].to_vec(),
        prefix: raw.prefix[..usize::from(raw.prefix_len)].to_vec(),
        text_colour: raw.text_colour,
        prefix_colour: raw.prefix_colour,
    })
}

pub(crate) fn text_label_from_abi(
    raw: SampClientSdkTextLabelV1,
) -> Result<Option<TextLabel>, SampClientSdkResult> {
    match raw.exists {
        0 => {
            if raw != SampClientSdkTextLabelV1::default() {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(None)
        }
        1 if matches!(raw.behind_walls, 0 | 1)
            && raw._reserved == [0; 2]
            && raw._reserved2 == 0
            && raw._reserved3 == [0; 2]
            && usize::from(raw.text_len) <= raw.text.len()
            && !raw.text[..usize::from(raw.text_len)].contains(&0)
            && raw._reserved3 == [0; 2]
            && raw.position.x.is_finite()
            && raw.position.y.is_finite()
            && raw.position.z.is_finite()
            && raw.draw_distance.is_finite() =>
        {
            let text_len = usize::from(raw.text_len);
            if text_len > raw.text.len() || raw.text[..text_len].contains(&0) {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(Some(TextLabel {
                id: raw.id,
                text: raw.text[..text_len].to_vec(),
                colour: raw.colour,
                position: raw.position,
                draw_distance: raw.draw_distance,
                behind_walls: raw.behind_walls != 0,
                attached_player_id: (raw.attached_player_id != u16::MAX)
                    .then_some(raw.attached_player_id),
                attached_vehicle_id: (raw.attached_vehicle_id != u16::MAX)
                    .then_some(raw.attached_vehicle_id),
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

pub(crate) fn textdraw_from_abi(
    raw: SampClientSdkTextDrawV1,
) -> Result<Option<TextDraw>, SampClientSdkResult> {
    match raw.exists {
        0 => {
            if raw != SampClientSdkTextDrawV1::default() {
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            Ok(None)
        }
        1 if matches!(raw.proportional, 0 | 1)
            && matches!(raw.align_left, 0 | 1)
            && matches!(raw.align_center, 0 | 1)
            && matches!(raw.align_right, 0 | 1)
            && matches!(raw.box_enabled, 0 | 1)
            && raw._reserved == [0; 2]
            && raw._reserved2 == 0
            && usize::from(raw.text_len) <= MAX_SAMP_TEXTDRAW_STRING_BYTES
            && raw.letter_width.is_finite()
            && raw.letter_height.is_finite()
            && raw.x.is_finite()
            && raw.y.is_finite()
            && raw.box_width.is_finite()
            && raw.box_height.is_finite()
            && raw.rotation.x.is_finite()
            && raw.rotation.y.is_finite()
            && raw.rotation.z.is_finite()
            && raw.zoom.is_finite() =>
        {
            Ok(Some(TextDraw {
                pool_index: raw.pool_index,
                text: raw.text[..usize::from(raw.text_len)].to_vec(),
                letter_width: raw.letter_width,
                letter_height: raw.letter_height,
                letter_colour: raw.letter_colour,
                x: raw.x,
                y: raw.y,
                shadow: raw.shadow,
                outline: raw.outline,
                background_colour: raw.background_colour,
                style: raw.style,
                proportional: raw.proportional != 0,
                align_left: raw.align_left != 0,
                align_center: raw.align_center != 0,
                align_right: raw.align_right != 0,
                box_enabled: raw.box_enabled != 0,
                box_width: raw.box_width,
                box_height: raw.box_height,
                box_colour: raw.box_colour,
                model_id: raw.model_id,
                rotation: raw.rotation,
                zoom: raw.zoom,
                model_colour1: raw.model_colour1,
                model_colour2: raw.model_colour2,
            }))
        }
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}
