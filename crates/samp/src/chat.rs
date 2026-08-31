use crate::{ChatDisplayMode, CommandReceipt, Subscription};
use modkit_abi::{
    MOD_NATIVE_CALL_FAILED, ModResult, SAMP_MAX_CHAT_COMMAND_ARGUMENT_BYTES,
    SAMP_MAX_CHAT_ENTRY_PREFIX_BYTES, SAMP_MAX_CHAT_ENTRY_TEXT_BYTES, SampChatCommandCallbackV1,
};
use modkit_sdk::{Core, SampService, SampUiService};
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};

type ChatCommandHandler = dyn Fn(&[u8]) + Send + Sync + 'static;

struct ChatCommandState {
    handler: Box<ChatCommandHandler>,
}

#[derive(Clone, Copy)]
pub struct Chat {
    core: Core,
    service: SampService,
    ui: SampUiService,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChatStyle {
    Chat,
    Info,
    Debug,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatEntry {
    pub id: u16,
    pub text: Vec<u8>,
    pub prefix: Vec<u8>,
    pub text_colour: u32,
    pub prefix_colour: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeathMessage<'a> {
    pub killer: &'a [u8],
    pub victim: &'a [u8],
    pub killer_colour: u32,
    pub victim_colour: u32,
    pub weapon: u8,
}

impl ChatStyle {
    const fn raw(self) -> u32 {
        match self {
            Self::Chat => modkit_abi::SAMP_CHAT_STYLE_CHAT,
            Self::Info => modkit_abi::SAMP_CHAT_STYLE_INFO,
            Self::Debug => modkit_abi::SAMP_CHAT_STYLE_DEBUG,
        }
    }
}

pub struct ChatCommandRegistration {
    pub subscription: Subscription,
    pub installation: CommandReceipt,
}

impl Chat {
    pub(crate) const fn new(core: Core, service: SampService, ui: SampUiService) -> Self {
        Self { core, service, ui }
    }

    pub fn add(
        self,
        style: ChatStyle,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandReceipt, ModResult> {
        let id =
            self.service
                .submit_chat_add(style.raw(), text, prefix, text_colour, prefix_colour)?;
        CommandReceipt::new(self.core, id)
    }

    pub fn display_mode(self) -> Result<ChatDisplayMode, ModResult> {
        ChatDisplayMode::from_raw(self.ui.chat_display_mode()?).ok_or(MOD_NATIVE_CALL_FAILED)
    }

    pub fn set_display_mode(self, mode: ChatDisplayMode) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.ui.submit_chat_display_mode(mode.raw())?)
    }

    pub fn entry(self, id: u16) -> Result<ChatEntry, ModResult> {
        let raw = self.ui.chat_entry(id)?;
        let text_len = usize::from(raw.text_len);
        let prefix_len = usize::from(raw.prefix_len);
        if text_len > SAMP_MAX_CHAT_ENTRY_TEXT_BYTES
            || prefix_len > SAMP_MAX_CHAT_ENTRY_PREFIX_BYTES
        {
            return Err(MOD_NATIVE_CALL_FAILED);
        }
        Ok(ChatEntry {
            id: raw.id,
            text: raw.text[..text_len].to_vec(),
            prefix: raw.prefix[..prefix_len].to_vec(),
            text_colour: raw.text_colour,
            prefix_colour: raw.prefix_colour,
        })
    }

    pub fn set_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandReceipt, ModResult> {
        let receipt = self
            .ui
            .submit_chat_entry(id, text, prefix, text_colour, prefix_colour)?;
        CommandReceipt::new(self.core, receipt)
    }

    pub fn add_death(self, message: DeathMessage<'_>) -> Result<CommandReceipt, ModResult> {
        let receipt = self.ui.submit_death_message(
            message.killer,
            message.victim,
            message.killer_colour,
            message.victim_colour,
            message.weapon,
        )?;
        CommandReceipt::new(self.core, receipt)
    }

    pub fn register_command(
        self,
        name: &[u8],
        handler: impl Fn(&[u8]) + Send + Sync + 'static,
    ) -> Result<ChatCommandRegistration, ModResult> {
        let state = Box::new(ChatCommandState {
            handler: Box::new(handler),
        });
        let raw = Box::into_raw(state);
        let result = unsafe {
            self.service.submit_register_chat_command(
                name,
                dispatch_chat_command as SampChatCommandCallbackV1,
                raw.cast::<c_void>(),
                release_chat_command,
            )
        };
        let (subscription, receipt) = match result {
            Ok(value) => value,
            Err(error) => {
                drop(unsafe { Box::from_raw(raw) });
                return Err(error);
            }
        };
        Ok(ChatCommandRegistration {
            subscription: Subscription::new(self.core, subscription)?,
            installation: CommandReceipt::new(self.core, receipt)?,
        })
    }
}

unsafe extern "system" fn dispatch_chat_command(
    user_data: *mut c_void,
    arguments: *const u8,
    arguments_len: u32,
) {
    if user_data.is_null()
        || arguments_len > SAMP_MAX_CHAT_COMMAND_ARGUMENT_BYTES
        || (arguments.is_null() && arguments_len != 0)
    {
        return;
    }
    let Some(state) = (unsafe { user_data.cast::<ChatCommandState>().as_ref() }) else {
        return;
    };
    let arguments = if arguments_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(arguments, arguments_len as usize) }
    };
    let _ = catch_unwind(AssertUnwindSafe(|| (state.handler)(arguments)));
}

unsafe extern "system" fn release_chat_command(user_data: *mut c_void) {
    if !user_data.is_null() {
        drop(unsafe { Box::from_raw(user_data.cast::<ChatCommandState>()) });
    }
}
