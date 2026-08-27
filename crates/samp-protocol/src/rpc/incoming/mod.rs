//! Incoming SA-MP RPC codecs.
//!
//! The fixed module owns four completed batches. The latest batch contains the
//! 29 descriptors from `ATTACH_CAMERA_TO_OBJECT` through `PLAYER_EXIT_VEHICLE`,
//! inclusive. `SHOW_DIALOG` remains SDK-owned because it crosses the Native
//! encoded-string boundary.

pub mod fixed;

pub use fixed::*;
