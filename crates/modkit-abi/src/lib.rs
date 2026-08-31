//! Stable C ABI primitives shared by the modkit host and plugin-side crates.
//!
//! This crate owns the exact-version service discovery ABI and the scalar
//! result/ID types that cross the stable plugin boundary. It deliberately
//! contains no Windows, MinHook, GTA, or SA-MP native dependencies and no
//! allocator-owned values in its ABI declarations.

#![deny(unsafe_op_in_unsafe_fn)]

mod bootstrap;
mod context;
mod core;
mod gta_sa;
mod ids;
mod legacy;
mod result;
mod samp;
mod samp_codec;
mod samp_control;
mod samp_net;
mod samp_player;
mod samp_pool;
mod samp_text_label;
mod samp_textdraw;
mod samp_ui;
mod service;

pub use bootstrap::{GetModHostApiV1, MOD_HOST_ABI_VERSION_V1, ModHostApiV1};
pub use context::{GameContextTokenV1, NativeExecutionConstraintV1};
pub use core::{
    CommandCompletionV1, CoreServiceV1, HostStatusV1, LOG_LEVEL_DEBUG, LOG_LEVEL_ERROR,
    LOG_LEVEL_INFO, LOG_LEVEL_WARN, MAX_LOG_MESSAGE_BYTES, TIMEOUT_INFINITE,
};
pub use gta_sa::*;
pub use ids::{CommandReceiptId, ServiceId, SubscriptionId};
pub use legacy::LegacySampServiceV1;
pub use result::{
    MOD_BUFFER_TOO_SMALL, MOD_BUSY, MOD_CALLBACK_IN_PROGRESS, MOD_INVALID_ARGUMENT,
    MOD_NATIVE_CALL_FAILED, MOD_NOT_FOUND, MOD_NOT_READY, MOD_OK, MOD_OUT_OF_BOUNDS,
    MOD_PAYLOAD_TOO_LARGE, MOD_PENDING, MOD_QUEUE_FULL, MOD_SHUTTING_DOWN, MOD_TIMED_OUT,
    MOD_UNSUPPORTED, MOD_UNSUPPORTED_VERSION, MOD_WAIT_REJECTED, ModResult,
};
pub use samp::*;
pub use samp_codec::*;
pub use samp_control::*;
pub use samp_net::*;
pub use samp_player::*;
pub use samp_pool::*;
pub use samp_text_label::*;
pub use samp_textdraw::*;
pub use samp_ui::*;
pub use service::{
    SERVICE_ID_CORE, SERVICE_ID_GTA_SA, SERVICE_ID_INPUT, SERVICE_ID_LEGACY_SAMP_ABI,
    SERVICE_ID_RENDER, SERVICE_ID_SAMP, SERVICE_ID_SAMP_CODEC, SERVICE_ID_SAMP_CONTROL,
    SERVICE_ID_SAMP_NETWORK, SERVICE_ID_SAMP_PLAYER, SERVICE_ID_SAMP_POOL,
    SERVICE_ID_SAMP_TEXT_LABEL, SERVICE_ID_SAMP_TEXTDRAW, SERVICE_ID_SAMP_UI, ServiceHeader,
};
