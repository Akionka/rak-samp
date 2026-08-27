//! Incoming SA-MP RPC codecs.
//!
//! The fixed module owns three completed batches. The final batch contains the
//! 26 descriptors from `CLIENT_CHECK` through `SET_CAMERA_BEHIND`, inclusive.

pub mod fixed;

pub use fixed::*;
