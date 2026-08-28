//! Incoming SA-MP RPC codecs.
//!
//! The `common` module owns profile-neutral descriptors. The `r1` module owns
//! player, session, world, UI, vehicle, and actor descriptors specific to the
//! R1 profile. Descriptors that require Native encoded strings remain SDK-owned.

pub mod common;
pub mod r1;
