use crate::{CommandReceipt, Subscription};
use modkit_abi::{ModResult, SAMP_MAX_CHAT_COMMAND_ARGUMENT_BYTES, SampChatCommandCallbackV1};
use modkit_sdk::{Core, SampService};
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChatStyle {
    Chat,
    Info,
    Debug,
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
    pub(crate) const fn new(core: Core, service: SampService) -> Self {
        Self { core, service }
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
