//! Plugin-side safe connection to the modkit host and service discovery.
//!
//! This crate resolves the `GtaModHost_GetApiV1` export and queries exact-version
//! service tables. It never falls back to the legacy `SampClientSdk_GetApiV1`
//! export. The host API and service tables are host-owned, immutable, and valid
//! for the process lifetime; host hot-unload is not supported.

#![deny(unsafe_op_in_unsafe_fn)]

mod context;
mod host;
mod resolve;

pub use context::GameContext;
pub use host::{
    Core, Host, HostStatus, LegacySamp, SampNetService, SampService, Service, ServiceError,
};
pub use resolve::{ConnectError, DEFAULT_HOST_MODULE};
