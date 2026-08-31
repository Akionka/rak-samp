use crate::CommandReceipt;
use modkit_abi::{
    MOD_INVALID_ARGUMENT, MOD_NATIVE_CALL_FAILED, ModResult, SAMP_MAX_CHAT_INPUT_TEXT_BYTES,
    SAMP_MAX_DIALOG_EDITBOX_TEXT_BYTES, SAMP_MAX_DIALOG_LIST_ITEM_BYTES,
    SAMP_MAX_DIALOG_LIST_ITEMS, SAMP_MAX_DIALOG_TEXT_BYTES, SAMP_UI_MAX_CHAT_COMMAND_NAME_BYTES,
    SampDialogRequestV1, SampDialogResponseV1, SampDialogSnapshotV1,
};
use modkit_sdk::{Core, SampUiService};

#[derive(Clone, Copy)]
pub struct Ui {
    core: Core,
    service: SampUiService,
}

#[derive(Clone, Copy)]
pub struct Dialogs {
    core: Core,
    service: SampUiService,
}

#[derive(Clone, Copy)]
pub struct ChatInput {
    core: Core,
    service: SampUiService,
}

#[derive(Clone, Copy)]
pub struct Cursor {
    core: Core,
    service: SampUiService,
}

#[derive(Clone, Copy)]
pub struct Scoreboard {
    core: Core,
    service: SampUiService,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialogStyle {
    MessageBox,
    Input,
    List,
    Password,
    TabList,
    HeadersList,
}

impl DialogStyle {
    const fn raw(self) -> u32 {
        match self {
            Self::MessageBox => 0,
            Self::Input => 1,
            Self::List => 2,
            Self::Password => 3,
            Self::TabList => 4,
            Self::HeadersList => 5,
        }
    }

    const fn from_raw(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::MessageBox),
            1 => Some(Self::Input),
            2 => Some(Self::List),
            3 => Some(Self::Password),
            4 => Some(Self::TabList),
            5 => Some(Self::HeadersList),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DialogRequest<'a> {
    pub id: u16,
    pub style: DialogStyle,
    pub title: &'a [u8],
    pub text: &'a [u8],
    pub button1: &'a [u8],
    pub button2: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogState {
    pub id: i32,
    pub style: DialogStyle,
    pub title: Vec<u8>,
    pub server_side: bool,
    pub text: Vec<u8>,
    pub editbox_text: Option<Vec<u8>>,
    pub items: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogResponse {
    pub dialog_id: u16,
    pub button: u8,
    pub list_item: i32,
    pub input: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChatDisplayMode {
    Off,
    NoShadow,
    Normal,
}

impl ChatDisplayMode {
    pub(crate) const fn raw(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::NoShadow => 1,
            Self::Normal => 2,
        }
    }

    pub(crate) const fn from_raw(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            1 => Some(Self::NoShadow),
            2 => Some(Self::Normal),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CursorMode {
    None,
    LockKeysNoCursor,
    LockCameraAndControl,
    LockCamera,
    LockCameraNoCursor,
}

impl CursorMode {
    const fn raw(self) -> i32 {
        match self {
            Self::None => 0,
            Self::LockKeysNoCursor => 1,
            Self::LockCameraAndControl => 2,
            Self::LockCamera => 3,
            Self::LockCameraNoCursor => 4,
        }
    }

    const fn from_raw(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::LockKeysNoCursor),
            2 => Some(Self::LockCameraAndControl),
            3 => Some(Self::LockCamera),
            4 => Some(Self::LockCameraNoCursor),
            _ => None,
        }
    }
}

impl Ui {
    pub(crate) const fn new(core: Core, service: SampUiService) -> Self {
        Self { core, service }
    }

    #[must_use]
    pub const fn dialogs(self) -> Dialogs {
        Dialogs {
            core: self.core,
            service: self.service,
        }
    }

    #[must_use]
    pub const fn chat_input(self) -> ChatInput {
        ChatInput {
            core: self.core,
            service: self.service,
        }
    }

    #[must_use]
    pub const fn cursor(self) -> Cursor {
        Cursor {
            core: self.core,
            service: self.service,
        }
    }

    #[must_use]
    pub const fn scoreboard(self) -> Scoreboard {
        Scoreboard {
            core: self.core,
            service: self.service,
        }
    }
}

impl Dialogs {
    pub fn is_active(self) -> Result<bool, ModResult> {
        self.service.dialog_active()
    }

    pub fn active(self) -> Result<Option<DialogState>, ModResult> {
        dialog_state(self.service.dialog_snapshot()?)
    }

    pub fn last_response(self) -> Result<Option<DialogResponse>, ModResult> {
        dialog_response(self.service.take_dialog_response()?)
    }

    pub fn selected_item(self) -> Result<i32, ModResult> {
        self.service.dialog_selected_item()
    }

    pub fn list_item_count(self) -> Result<i32, ModResult> {
        self.service.dialog_list_item_count()
    }

    pub fn show(self, request: DialogRequest<'_>) -> Result<CommandReceipt, ModResult> {
        if !valid_dialog_request(request) {
            return Err(MOD_INVALID_ARGUMENT);
        }
        let raw = SampDialogRequestV1 {
            id: request.id,
            reserved: 0,
            style: request.style.raw(),
            title: request.title.as_ptr(),
            title_len: checked_len(request.title.len())?,
            text: request.text.as_ptr(),
            text_len: checked_len(request.text.len())?,
            button1: request.button1.as_ptr(),
            button1_len: checked_len(request.button1.len())?,
            button2: request.button2.as_ptr(),
            button2_len: checked_len(request.button2.len())?,
        };
        CommandReceipt::new(self.core, self.service.submit_dialog(&raw)?)
    }

    pub fn set_client_side(self, client_side: bool) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service.submit_dialog_client_side(client_side)?,
        )
    }

    pub fn set_selected_item(self, selected: i32) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service.submit_dialog_selected_item(selected)?,
        )
    }

    pub fn set_editbox_text(self, text: &[u8]) -> Result<CommandReceipt, ModResult> {
        if text.len() > SAMP_MAX_DIALOG_EDITBOX_TEXT_BYTES || text.contains(&0) {
            return Err(MOD_INVALID_ARGUMENT);
        }
        CommandReceipt::new(self.core, self.service.submit_dialog_editbox_text(text)?)
    }

    pub fn close_with_button(self, button: u8) -> Result<CommandReceipt, ModResult> {
        if button > 1 {
            return Err(MOD_INVALID_ARGUMENT);
        }
        CommandReceipt::new(self.core, self.service.submit_dialog_close(button)?)
    }
}

impl ChatInput {
    pub fn is_active(self) -> Result<bool, ModResult> {
        self.service.chat_input_active()
    }

    pub fn text(self) -> Result<Vec<u8>, ModResult> {
        let raw = self.service.chat_input_text()?;
        let len = usize::from(raw.len);
        raw.bytes
            .get(..len)
            .map(<[u8]>::to_vec)
            .ok_or(MOD_NATIVE_CALL_FAILED)
    }

    pub fn is_command_defined(self, name: &[u8]) -> Result<bool, ModResult> {
        if name.is_empty() || name.len() > SAMP_UI_MAX_CHAT_COMMAND_NAME_BYTES || name.contains(&0)
        {
            return Err(MOD_INVALID_ARGUMENT);
        }
        self.service.chat_command_defined(name)
    }

    pub fn set_text(self, text: &[u8]) -> Result<CommandReceipt, ModResult> {
        validate_chat_input(text)?;
        CommandReceipt::new(self.core, self.service.submit_chat_input_text(text)?)
    }

    pub fn set_enabled(self, enabled: bool) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_chat_input_enabled(enabled)?)
    }

    pub fn process(self, text: &[u8]) -> Result<CommandReceipt, ModResult> {
        validate_chat_input(text)?;
        CommandReceipt::new(self.core, self.service.submit_chat_input_process(text)?)
    }
}

impl Cursor {
    pub fn mode(self) -> Result<CursorMode, ModResult> {
        CursorMode::from_raw(self.service.cursor_mode()?).ok_or(MOD_NATIVE_CALL_FAILED)
    }

    pub fn is_active(self) -> Result<bool, ModResult> {
        Ok(self.mode()? != CursorMode::None)
    }

    pub fn set_mode(self, mode: CursorMode) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_cursor_mode(mode.raw())?)
    }

    pub fn toggle(self, show: bool) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_cursor_toggle(show)?)
    }
}

impl Scoreboard {
    pub fn is_open(self) -> Result<bool, ModResult> {
        self.service.scoreboard_open()
    }

    pub fn toggle(self, open: bool) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_scoreboard_open(open)?)
    }
}

fn valid_dialog_request(request: DialogRequest<'_>) -> bool {
    [
        request.title,
        request.text,
        request.button1,
        request.button2,
    ]
    .iter()
    .all(|value| !value.contains(&0))
        && request.title.len() <= 255
        && request.button1.len() <= 255
        && request.button2.len() <= 255
        && request.text.len() < SAMP_MAX_DIALOG_TEXT_BYTES
}

fn validate_chat_input(text: &[u8]) -> Result<(), ModResult> {
    if text.len() > SAMP_MAX_CHAT_INPUT_TEXT_BYTES || text.contains(&0) {
        Err(MOD_INVALID_ARGUMENT)
    } else {
        Ok(())
    }
}

fn dialog_state(raw: SampDialogSnapshotV1) -> Result<Option<DialogState>, ModResult> {
    if raw.active == 0 {
        return Ok(None);
    }
    let title = fixed_bytes(&raw.title, usize::from(raw.title_len))?;
    let text = fixed_bytes(&raw.text, usize::from(raw.text_len))?;
    let editbox_text = if raw.has_editbox == 0 {
        None
    } else {
        Some(fixed_bytes(
            &raw.editbox_text,
            usize::from(raw.editbox_text_len),
        )?)
    };
    let count = usize::from(raw.listbox_item_count);
    if count > SAMP_MAX_DIALOG_LIST_ITEMS {
        return Err(MOD_NATIVE_CALL_FAILED);
    }
    let mut items = Vec::with_capacity(count);
    for item in &raw.listbox_items[..count] {
        let len = usize::from(item.len);
        if len > SAMP_MAX_DIALOG_LIST_ITEM_BYTES {
            return Err(MOD_NATIVE_CALL_FAILED);
        }
        items.push(item.bytes[..len].to_vec());
    }
    Ok(Some(DialogState {
        id: raw.id,
        style: DialogStyle::from_raw(raw.style).ok_or(MOD_NATIVE_CALL_FAILED)?,
        title,
        server_side: raw.server_side != 0,
        text,
        editbox_text,
        items,
    }))
}

fn dialog_response(raw: SampDialogResponseV1) -> Result<Option<DialogResponse>, ModResult> {
    if raw.available == 0 {
        return Ok(None);
    }
    Ok(Some(DialogResponse {
        dialog_id: raw.dialog_id,
        button: raw.button,
        list_item: raw.list_item,
        input: fixed_bytes(&raw.input, usize::from(raw.input_len))?,
    }))
}

fn fixed_bytes<const N: usize>(value: &[u8; N], len: usize) -> Result<Vec<u8>, ModResult> {
    value
        .get(..len)
        .map(<[u8]>::to_vec)
        .ok_or(MOD_NATIVE_CALL_FAILED)
}

fn checked_len(len: usize) -> Result<u32, ModResult> {
    u32::try_from(len).map_err(|_| MOD_INVALID_ARGUMENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_dialog_does_not_require_valid_style() {
        let raw = SampDialogSnapshotV1 {
            style: u8::MAX,
            ..SampDialogSnapshotV1::default()
        };
        assert_eq!(dialog_state(raw), Ok(None));
    }

    #[test]
    fn active_dialog_rejects_invalid_lengths() {
        let raw = SampDialogSnapshotV1 {
            active: 1,
            title_len: u8::MAX,
            ..SampDialogSnapshotV1::default()
        };
        assert_eq!(dialog_state(raw), Err(MOD_NATIVE_CALL_FAILED));
    }

    #[test]
    fn dialog_request_rejects_nul_and_oversized_text() {
        let valid = DialogRequest {
            id: 1,
            style: DialogStyle::MessageBox,
            title: b"title",
            text: b"text",
            button1: b"ok",
            button2: b"",
        };
        assert!(valid_dialog_request(valid));
        assert!(!valid_dialog_request(DialogRequest {
            title: b"bad\0title",
            ..valid
        }));
        assert!(!valid_dialog_request(DialogRequest {
            text: &[b'x'; SAMP_MAX_DIALOG_TEXT_BYTES],
            ..valid
        }));
    }
}
