//! Deferred plugin-context reclamation outside callback and game threads.

use super::host;
use modkit_abi::SampReleaseCallbackV1;
use modkit_runtime::{DeferredReclamation, Reclaimable};

pub(super) struct PluginRelease {
    user_data: usize,
    callback: SampReleaseCallbackV1,
}

impl PluginRelease {
    pub(super) const fn new(user_data: usize, callback: SampReleaseCallbackV1) -> Self {
        Self {
            user_data,
            callback,
        }
    }

    pub(super) fn release(self) {
        unsafe { (self.callback)(self.user_data as *mut core::ffi::c_void) };
    }
}

pub(super) fn defer(release: impl FnOnce() + Send + 'static) {
    host()
        .deferred_reclamation
        .enqueue(Reclaimable::new(release));
    let _ = std::thread::Builder::new()
        .name("samp-client-sdk-reclamation".into())
        .spawn(|| host().deferred_reclamation.run_pending());
}

pub(super) fn new_queue() -> DeferredReclamation {
    DeferredReclamation::new()
}
