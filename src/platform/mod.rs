#[cfg(not(all(windows, target_pointer_width = "32")))]
mod unsupported;
#[cfg(all(windows, target_pointer_width = "32"))]
mod win32;

pub(crate) use samp_native::memory::bounded_c_string;

#[cfg(not(all(windows, target_pointer_width = "32")))]
pub(crate) use unsupported::*;
#[cfg(all(windows, target_pointer_width = "32"))]
pub(crate) use win32::*;
