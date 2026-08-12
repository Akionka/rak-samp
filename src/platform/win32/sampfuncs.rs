//! Optional bridge to the SAMPFUNCS 5.7.1 console logger.
//!
//! SAMPFUNCS exposes `SAMPFUNCS::LogConsole` as a decorated C++ export rather
//! than a C ABI. We only borrow that logger when its ASI is already loaded; we
//! never load SAMPFUNCS or register this host as one of its plugins.

use std::{
    ffi::{c_char, c_void},
    mem, ptr,
    sync::{Mutex, OnceLock},
};
use windows_sys::Win32::{
    Foundation::HMODULE,
    System::LibraryLoader::{GetModuleHandleA, GetModuleHandleExA, GetProcAddress},
};

const SAMPFUNCS_MODULE: &core::ffi::CStr = c"SAMPFUNCS.asi";
const LOG_CONSOLE_EXPORT: &core::ffi::CStr = c"?LogConsole@SAMPFUNCS@@QAAXPBDZZ";

/// The supplied SAMPFUNCS 5.7.1 header contains only this plugin pointer.
/// A valid local instance avoids passing a null C++ `this` argument.
#[repr(C)]
struct SampfuncsInstance {
    plugin: *mut c_void,
}

// `QAAX` in the x86 MSVC export decoration denotes a cdecl member function.
// The hidden `this` pointer is therefore the first stack argument.
type LogConsoleFn = unsafe extern "C" fn(*mut SampfuncsInstance, *const c_char);

struct SampfuncsBinding {
    // Retains a module reference for the cached function pointer lifetime.
    _module: usize,
    log_console: LogConsoleFn,
}

enum BindingState {
    Unresolved,
    Resolved(SampfuncsBinding),
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SampfuncsLogError {
    NotLoaded,
    Unsupported,
    Failed,
}

static BINDING: OnceLock<Mutex<BindingState>> = OnceLock::new();

/// Returns whether the SAMPFUNCS ASI is already loaded in this process.
pub(crate) fn sampfuncs_loaded() -> bool {
    !unsafe { GetModuleHandleA(SAMPFUNCS_MODULE.as_ptr().cast()) }.is_null()
}

/// Writes a bounded NUL-free byte string through SAMPFUNCS's own console
/// logger. Percent signs are escaped because its exported method is variadic.
pub(crate) fn sampfuncs_log_console(text: &[u8]) -> Result<(), SampfuncsLogError> {
    if text.contains(&0) {
        return Err(SampfuncsLogError::Failed);
    }

    let mut binding = binding_state()
        .lock()
        .map_err(|_| SampfuncsLogError::Failed)?;
    if matches!(*binding, BindingState::Unresolved) {
        *binding = match resolve_binding() {
            Ok(resolved) => BindingState::Resolved(resolved),
            Err(SampfuncsLogError::Unsupported) => BindingState::Unsupported,
            Err(error) => return Err(error),
        };
    }
    let BindingState::Resolved(binding) = &*binding else {
        return Err(SampfuncsLogError::Unsupported);
    };

    let mut instance = SampfuncsInstance {
        plugin: ptr::null_mut(),
    };
    let format = escaped_format(text);
    // `format` supplies the complete variadic format string. It contains no
    // conversion directives, so no variadic arguments are required.
    unsafe { (binding.log_console)(&mut instance, format.as_ptr().cast()) };
    Ok(())
}

fn binding_state() -> &'static Mutex<BindingState> {
    BINDING.get_or_init(|| Mutex::new(BindingState::Unresolved))
}

fn resolve_binding() -> Result<SampfuncsBinding, SampfuncsLogError> {
    // Resolve the symbol before retaining the module so unsupported modules do
    // not gain a reference on every failed logging attempt.
    let module = unsafe { GetModuleHandleA(SAMPFUNCS_MODULE.as_ptr().cast()) };
    if module.is_null() {
        return Err(SampfuncsLogError::NotLoaded);
    }
    if unsafe { GetProcAddress(module, LOG_CONSOLE_EXPORT.as_ptr().cast()) }.is_none() {
        return Err(SampfuncsLogError::Unsupported);
    }

    let mut retained: HMODULE = ptr::null_mut();
    if unsafe { GetModuleHandleExA(0, SAMPFUNCS_MODULE.as_ptr().cast(), &raw mut retained) } == 0 {
        return Err(SampfuncsLogError::NotLoaded);
    }
    let Some(symbol) = (unsafe { GetProcAddress(retained, LOG_CONSOLE_EXPORT.as_ptr().cast()) })
    else {
        return Err(SampfuncsLogError::Unsupported);
    };
    // The SAMPFUNCS SDK declares `LogConsole(const char*, ...)`; this resolved
    // fixed-argument view is ABI-compatible when the format has no directives.
    let log_console =
        unsafe { mem::transmute::<unsafe extern "system" fn() -> isize, LogConsoleFn>(symbol) };
    Ok(SampfuncsBinding {
        _module: retained as usize,
        log_console,
    })
}

fn escaped_format(text: &[u8]) -> Vec<u8> {
    let mut format = Vec::with_capacity(text.len().saturating_mul(2).saturating_add(1));
    for &byte in text {
        format.push(byte);
        if byte == b'%' {
            format.push(b'%');
        }
    }
    format.push(0);
    format
}

#[cfg(test)]
mod tests {
    use super::escaped_format;

    #[test]
    fn console_format_escapes_percent_signs() {
        assert_eq!(escaped_format(b"100% ready"), b"100%% ready\0");
    }
}
