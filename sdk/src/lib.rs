//! Stable C ABI definitions and safe host-discovery helpers for `samp-client-sdk` plugins.
//!
//! Depend on this crate from an independently loaded ASI plugin. Do **not**
//! depend on the `samp_client_sdk` host crate: that would embed a second hook engine in
//! the process instead of communicating with `samp_client_sdk.asi`. Register callbacks with
//! [`Samp::net`] to register callbacks or send owned traffic. Use the ID-filtered and typed
//! variants when one handler owns one protocol message, and [`register_handlers!`] to keep a
//! group in one [`SubscriptionSet`]. Synchronize subscriptions before unloading the plugin.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp_client_sdk supports only 32-bit Windows x86 targets");

mod abi;
mod api;
pub mod events;
mod facade;
mod host_api;
pub mod limits;
pub mod raknet;
pub mod raw;
mod resolve;
mod subscriptions;
mod types;

pub use abi::*;
pub use api::*;
pub use facade::*;
use limits::{
    MAX_RAKNET_DECODED_STRING_BYTES, MAX_SAMP_CHAT_ENTRIES, MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES,
    MAX_SAMP_CHAT_ENTRY_TEXT_BYTES, MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES,
    MAX_SAMP_DIALOG_LISTBOX_ITEM_BYTES, MAX_SAMP_DIALOG_LISTBOX_ITEMS, MAX_SAMP_DIALOG_TEXT_BYTES,
    MAX_SAMP_GANGZONES, MAX_SAMP_OBJECTS, MAX_SAMP_PLAYERS, MAX_SAMP_TEXT_LABEL_TEXT_BYTES,
    MAX_SAMP_TEXT_LABELS, MAX_SAMP_TEXTDRAW_STRING_BYTES, MAX_SAMP_TEXTDRAWS, MAX_SAMP_VEHICLES,
};
pub use resolve::*;
pub use subscriptions::*;
pub use types::*;

use core::{ffi::c_void, mem, ptr::NonNull};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    time::Duration,
};

type EventHandler = dyn for<'event> Fn(&mut events::Event<'event>) -> SampClientSdkHookAction
    + Send
    + Sync
    + 'static;

type ChatCommandHandler = dyn Fn(&[u8]) + Send + Sync + 'static;

struct CallbackState {
    api: HostApi,
    handler: Box<EventHandler>,
}

pub(crate) struct ChatCommandCallbackState {
    pub(crate) handler: Box<ChatCommandHandler>,
}

type RegisterListener = unsafe extern "system" fn(
    SampClientSdkDirection,
    Option<SampClientSdkEventCallbackV1>,
    *mut c_void,
    *mut SampClientSdkSubscription,
) -> SampClientSdkResult;

unsafe extern "system" fn dispatch_callback(
    user_data: *mut c_void,
    raw: *mut SampClientSdkEventV1,
) -> SampClientSdkHookAction {
    let Some(callback) = (unsafe { user_data.cast::<CallbackState>().as_ref() }) else {
        return SampClientSdkHookAction::Continue;
    };
    let Ok(mut event) = (unsafe { events::Event::from_callback(callback.api, raw) }) else {
        return SampClientSdkHookAction::Continue;
    };
    catch_unwind(AssertUnwindSafe(|| (callback.handler)(&mut event)))
        .unwrap_or(SampClientSdkHookAction::Continue)
}

pub(crate) unsafe extern "system" fn dispatch_chat_command_callback(
    user_data: *mut c_void,
    args: *const u8,
    args_len: usize,
) {
    if args_len > limits::MAX_SAMP_CHAT_INPUT_TEXT_BYTES || (args.is_null() && args_len != 0) {
        return;
    }
    let Some(callback) = (unsafe { user_data.cast::<ChatCommandCallbackState>().as_ref() }) else {
        return;
    };
    let args = if args_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args, args_len) }
    };
    let _ = catch_unwind(AssertUnwindSafe(|| (callback.handler)(args)));
}

#[cfg(test)]
mod tests;
