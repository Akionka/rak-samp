//! Incoming SA-MP RPC codecs.
//!
//! The `common` module owns profile-neutral descriptors. The `r1` module owns
//! player and session descriptors specific to the R1 profile. `SHOW_DIALOG`
//! remains SDK-owned because it crosses the Native encoded-string boundary.

pub mod common;
pub mod r1;
