//! Owned local chat-command registrations and their game-thread lifecycle.

use super::{clone_initialized, copied_nul_free_string, direct_client_result, host};
use crate::{command::CommandError, platform::bounded_c_string};
use sdk_abi::{
    SampClientSdkChatCommandCallbackV1, SampClientSdkCommandReceipt, SampClientSdkResult,
    SampClientSdkSubscription,
};
use std::{
    collections::HashMap,
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

const MAX_CHAT_COMMANDS: usize = 144;
const MAX_CHAT_COMMAND_NAME_BYTES: usize = 32;
const MAX_CHAT_COMMAND_ARGUMENT_BYTES: usize = 128;

thread_local! {
    static CHAT_COMMAND_CALLBACK_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Returns whether the current thread is executing one local chat-command callback.
pub(crate) fn is_dispatching_on_current_thread() -> bool {
    CHAT_COMMAND_CALLBACK_DEPTH.with(|depth| depth.get() != 0)
}

pub(super) struct ChatCommandRegistry {
    state: Mutex<ChatCommandRegistryState>,
}

struct ChatCommandRegistryState {
    entries: HashMap<u64, Arc<ChatCommandEntry>>,
    slots: Vec<Option<u64>>,
}

struct ChatCommandEntry {
    name: Vec<u8>,
    slot: u8,
    callback: SampClientSdkChatCommandCallbackV1,
    user_data: usize,
    registered: AtomicBool,
    active: AtomicBool,
    dispatches: AtomicUsize,
    no_dispatches: Condvar,
    dispatch_lock: Mutex<()>,
}

impl ChatCommandRegistry {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(ChatCommandRegistryState {
                entries: HashMap::new(),
                slots: vec![None; MAX_CHAT_COMMANDS],
            }),
        }
    }

    fn reserve(
        &self,
        id: u64,
        name: Vec<u8>,
        callback: SampClientSdkChatCommandCallbackV1,
        user_data: usize,
    ) -> Result<Arc<ChatCommandEntry>, SampClientSdkResult> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.entries.values().any(|entry| entry.name == name) {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let Some(slot) = state.slots.iter().position(Option::is_none) else {
            return Err(SampClientSdkResult::QueueFull);
        };
        let entry = Arc::new(ChatCommandEntry {
            name,
            slot: slot as u8,
            callback,
            user_data,
            registered: AtomicBool::new(false),
            active: AtomicBool::new(true),
            dispatches: AtomicUsize::new(0),
            no_dispatches: Condvar::new(),
            dispatch_lock: Mutex::new(()),
        });
        state.slots[slot] = Some(id);
        state.entries.insert(id, Arc::clone(&entry));
        Ok(entry)
    }

    fn entry(&self, id: u64) -> Option<Arc<ChatCommandEntry>> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .entries
            .get(&id)
            .cloned()
    }

    fn entry_for_slot(&self, slot: usize) -> Option<Arc<ChatCommandEntry>> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let id = (*state.slots.get(slot)?)?;
        state.entries.get(&id).cloned()
    }

    fn remove(&self, id: u64) -> Option<Arc<ChatCommandEntry>> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let entry = state.entries.remove(&id)?;
        state.slots[usize::from(entry.slot)] = None;
        Some(entry)
    }

    fn finish_registration(&self, id: u64, succeeded: bool) {
        let Some(entry) = self.entry(id) else {
            return;
        };
        if succeeded {
            entry.registered.store(true, Ordering::Release);
        } else {
            entry.active.store(false, Ordering::Release);
            let _ = self.remove(id);
        }
    }

    fn finish_unregistration(&self, id: u64, succeeded: bool) {
        let Some(entry) = self.entry(id) else {
            return;
        };
        if succeeded {
            entry.active.store(false, Ordering::Release);
            let _ = self.remove(id);
        } else {
            entry.active.store(true, Ordering::Release);
        }
    }
}

impl ChatCommandEntry {
    fn enter_dispatch(&self) -> Option<ChatCommandDispatch<'_>> {
        if !self.registered.load(Ordering::Acquire) || !self.active.load(Ordering::Acquire) {
            return None;
        }
        let guard = self
            .dispatch_lock
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if !self.registered.load(Ordering::Acquire) || !self.active.load(Ordering::Acquire) {
            return None;
        }
        self.dispatches.fetch_add(1, Ordering::AcqRel);
        drop(guard);
        CHAT_COMMAND_CALLBACK_DEPTH.with(|depth| depth.set(depth.get() + 1));
        Some(ChatCommandDispatch { entry: self })
    }

    fn deactivate(&self) {
        let _guard = self
            .dispatch_lock
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        self.active.store(false, Ordering::Release);
    }

    fn reactivate(&self) {
        self.active.store(true, Ordering::Release);
    }

    fn synchronize_dispatches(&self) {
        let mut guard = self
            .dispatch_lock
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        while self.dispatches.load(Ordering::Acquire) != 0 {
            guard = self
                .no_dispatches
                .wait(guard)
                .unwrap_or_else(|error| error.into_inner());
        }
    }
}

struct ChatCommandDispatch<'entry> {
    entry: &'entry ChatCommandEntry,
}

impl Drop for ChatCommandDispatch<'_> {
    fn drop(&mut self) {
        CHAT_COMMAND_CALLBACK_DEPTH.with(|depth| depth.set(depth.get() - 1));
        if self.entry.dispatches.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.entry.no_dispatches.notify_all();
        }
    }
}

pub(super) unsafe extern "system" fn submit_register_chat_command(
    name: *const u8,
    name_len: usize,
    callback: Option<SampClientSdkChatCommandCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut SampClientSdkSubscription,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    let Some(callback) = callback else {
        return SampClientSdkResult::InvalidArgument;
    };
    if user_data.is_null() || subscription.is_null() || receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(name) = (unsafe { copied_nul_free_string(name, name_len, MAX_CHAT_COMMAND_NAME_BYTES) })
    else {
        return SampClientSdkResult::InvalidArgument;
    };
    if name.is_empty() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    if is_dispatching_on_current_thread() || !runtime.command_wait_allowed() {
        return SampClientSdkResult::CallbackInProgress;
    }
    let id = host().next_subscription.fetch_add(1, Ordering::AcqRel);
    let entry = match host()
        .chat_commands
        .reserve(id, name.clone(), callback, user_data as usize)
    {
        Ok(entry) => entry,
        Err(error) => return error,
    };
    match runtime.submit_register_chat_command(id, entry.slot, name) {
        Ok(command_id) => {
            unsafe {
                subscription.write(SampClientSdkSubscription { id });
                receipt.write(SampClientSdkCommandReceipt { id: command_id });
            }
            SampClientSdkResult::Ok
        }
        Err(error) => {
            entry.deactivate();
            let _ = host().chat_commands.remove(id);
            direct_client_result(error)
        }
    }
}

pub(super) fn unregister(subscription: SampClientSdkSubscription) -> Option<SampClientSdkResult> {
    let entry = host().chat_commands.entry(subscription.id)?;
    entry.deactivate();
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return Some(SampClientSdkResult::NotReady);
    };
    Some(
        runtime
            .submit_unregister_chat_command(subscription.id, entry.name.clone())
            .map_or_else(direct_client_result, |_| SampClientSdkResult::Ok),
    )
}

pub(super) fn unregister_and_wait(
    subscription: SampClientSdkSubscription,
) -> Option<SampClientSdkResult> {
    let entry = host().chat_commands.entry(subscription.id)?;
    if is_dispatching_on_current_thread() {
        return Some(SampClientSdkResult::CallbackInProgress);
    }
    entry.deactivate();
    let Some(runtime) = clone_initialized(&host().runtime) else {
        entry.reactivate();
        return Some(SampClientSdkResult::NotReady);
    };
    if !runtime.command_wait_allowed() {
        entry.reactivate();
        return Some(SampClientSdkResult::CallbackInProgress);
    }
    let command_id =
        match runtime.submit_unregister_chat_command(subscription.id, entry.name.clone()) {
            Ok(command_id) => command_id,
            Err(error) => {
                entry.reactivate();
                return Some(direct_client_result(error));
            }
        };
    loop {
        match runtime.wait_for_command(command_id, Duration::MAX) {
            Err(CommandError::TimedOut) => continue,
            Ok(Ok(())) => {
                entry.synchronize_dispatches();
                return Some(SampClientSdkResult::Ok);
            }
            Ok(Err(error)) | Err(error) => {
                entry.reactivate();
                return Some(command_result(error));
            }
        }
    }
}

fn command_result(error: CommandError) -> SampClientSdkResult {
    match error {
        CommandError::QueueFull => SampClientSdkResult::QueueFull,
        CommandError::ShuttingDown => SampClientSdkResult::ShuttingDown,
        CommandError::NativeFailure => SampClientSdkResult::NativeCallFailed,
        CommandError::UnknownReceipt => SampClientSdkResult::InvalidArgument,
        CommandError::TimedOut => SampClientSdkResult::TimedOut,
        CommandError::WaitRejected => SampClientSdkResult::CallbackInProgress,
    }
}

/// Reports the fixed native callback trampoline for one bounded slot.
pub(crate) fn trampoline(slot: u8) -> unsafe extern "cdecl" fn(*const i8) {
    CHAT_COMMAND_TRAMPOLINES[usize::from(slot)]
}

/// Completes host-side registration bookkeeping before its command receipt is published.
pub(crate) fn finish_registration(subscription: u64, succeeded: bool) {
    host()
        .chat_commands
        .finish_registration(subscription, succeeded);
}

/// Completes host-side removal bookkeeping before its command receipt is published.
pub(crate) fn finish_unregistration(subscription: u64, succeeded: bool) {
    host()
        .chat_commands
        .finish_unregistration(subscription, succeeded);
}

fn dispatch_native_command(slot: usize, args: *const i8) {
    let Some(entry) = host().chat_commands.entry_for_slot(slot) else {
        return;
    };
    let Some(_dispatch) = entry.enter_dispatch() else {
        return;
    };
    let Some(args) =
        (unsafe { bounded_c_string(args.cast(), MAX_CHAT_COMMAND_ARGUMENT_BYTES + 1) })
    else {
        return;
    };
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        (entry.callback)(entry.user_data as *mut c_void, args.as_ptr(), args.len());
    }));
}

macro_rules! chat_command_trampoline {
    ($name:ident, $slot:literal) => {
        unsafe extern "cdecl" fn $name(args: *const i8) {
            dispatch_native_command($slot, args);
        }
    };
}

chat_command_trampoline!(chat_command_0, 0);
chat_command_trampoline!(chat_command_1, 1);
chat_command_trampoline!(chat_command_2, 2);
chat_command_trampoline!(chat_command_3, 3);
chat_command_trampoline!(chat_command_4, 4);
chat_command_trampoline!(chat_command_5, 5);
chat_command_trampoline!(chat_command_6, 6);
chat_command_trampoline!(chat_command_7, 7);
chat_command_trampoline!(chat_command_8, 8);
chat_command_trampoline!(chat_command_9, 9);
chat_command_trampoline!(chat_command_10, 10);
chat_command_trampoline!(chat_command_11, 11);
chat_command_trampoline!(chat_command_12, 12);
chat_command_trampoline!(chat_command_13, 13);
chat_command_trampoline!(chat_command_14, 14);
chat_command_trampoline!(chat_command_15, 15);
chat_command_trampoline!(chat_command_16, 16);
chat_command_trampoline!(chat_command_17, 17);
chat_command_trampoline!(chat_command_18, 18);
chat_command_trampoline!(chat_command_19, 19);
chat_command_trampoline!(chat_command_20, 20);
chat_command_trampoline!(chat_command_21, 21);
chat_command_trampoline!(chat_command_22, 22);
chat_command_trampoline!(chat_command_23, 23);
chat_command_trampoline!(chat_command_24, 24);
chat_command_trampoline!(chat_command_25, 25);
chat_command_trampoline!(chat_command_26, 26);
chat_command_trampoline!(chat_command_27, 27);
chat_command_trampoline!(chat_command_28, 28);
chat_command_trampoline!(chat_command_29, 29);
chat_command_trampoline!(chat_command_30, 30);
chat_command_trampoline!(chat_command_31, 31);
chat_command_trampoline!(chat_command_32, 32);
chat_command_trampoline!(chat_command_33, 33);
chat_command_trampoline!(chat_command_34, 34);
chat_command_trampoline!(chat_command_35, 35);
chat_command_trampoline!(chat_command_36, 36);
chat_command_trampoline!(chat_command_37, 37);
chat_command_trampoline!(chat_command_38, 38);
chat_command_trampoline!(chat_command_39, 39);
chat_command_trampoline!(chat_command_40, 40);
chat_command_trampoline!(chat_command_41, 41);
chat_command_trampoline!(chat_command_42, 42);
chat_command_trampoline!(chat_command_43, 43);
chat_command_trampoline!(chat_command_44, 44);
chat_command_trampoline!(chat_command_45, 45);
chat_command_trampoline!(chat_command_46, 46);
chat_command_trampoline!(chat_command_47, 47);
chat_command_trampoline!(chat_command_48, 48);
chat_command_trampoline!(chat_command_49, 49);
chat_command_trampoline!(chat_command_50, 50);
chat_command_trampoline!(chat_command_51, 51);
chat_command_trampoline!(chat_command_52, 52);
chat_command_trampoline!(chat_command_53, 53);
chat_command_trampoline!(chat_command_54, 54);
chat_command_trampoline!(chat_command_55, 55);
chat_command_trampoline!(chat_command_56, 56);
chat_command_trampoline!(chat_command_57, 57);
chat_command_trampoline!(chat_command_58, 58);
chat_command_trampoline!(chat_command_59, 59);
chat_command_trampoline!(chat_command_60, 60);
chat_command_trampoline!(chat_command_61, 61);
chat_command_trampoline!(chat_command_62, 62);
chat_command_trampoline!(chat_command_63, 63);
chat_command_trampoline!(chat_command_64, 64);
chat_command_trampoline!(chat_command_65, 65);
chat_command_trampoline!(chat_command_66, 66);
chat_command_trampoline!(chat_command_67, 67);
chat_command_trampoline!(chat_command_68, 68);
chat_command_trampoline!(chat_command_69, 69);
chat_command_trampoline!(chat_command_70, 70);
chat_command_trampoline!(chat_command_71, 71);
chat_command_trampoline!(chat_command_72, 72);
chat_command_trampoline!(chat_command_73, 73);
chat_command_trampoline!(chat_command_74, 74);
chat_command_trampoline!(chat_command_75, 75);
chat_command_trampoline!(chat_command_76, 76);
chat_command_trampoline!(chat_command_77, 77);
chat_command_trampoline!(chat_command_78, 78);
chat_command_trampoline!(chat_command_79, 79);
chat_command_trampoline!(chat_command_80, 80);
chat_command_trampoline!(chat_command_81, 81);
chat_command_trampoline!(chat_command_82, 82);
chat_command_trampoline!(chat_command_83, 83);
chat_command_trampoline!(chat_command_84, 84);
chat_command_trampoline!(chat_command_85, 85);
chat_command_trampoline!(chat_command_86, 86);
chat_command_trampoline!(chat_command_87, 87);
chat_command_trampoline!(chat_command_88, 88);
chat_command_trampoline!(chat_command_89, 89);
chat_command_trampoline!(chat_command_90, 90);
chat_command_trampoline!(chat_command_91, 91);
chat_command_trampoline!(chat_command_92, 92);
chat_command_trampoline!(chat_command_93, 93);
chat_command_trampoline!(chat_command_94, 94);
chat_command_trampoline!(chat_command_95, 95);
chat_command_trampoline!(chat_command_96, 96);
chat_command_trampoline!(chat_command_97, 97);
chat_command_trampoline!(chat_command_98, 98);
chat_command_trampoline!(chat_command_99, 99);
chat_command_trampoline!(chat_command_100, 100);
chat_command_trampoline!(chat_command_101, 101);
chat_command_trampoline!(chat_command_102, 102);
chat_command_trampoline!(chat_command_103, 103);
chat_command_trampoline!(chat_command_104, 104);
chat_command_trampoline!(chat_command_105, 105);
chat_command_trampoline!(chat_command_106, 106);
chat_command_trampoline!(chat_command_107, 107);
chat_command_trampoline!(chat_command_108, 108);
chat_command_trampoline!(chat_command_109, 109);
chat_command_trampoline!(chat_command_110, 110);
chat_command_trampoline!(chat_command_111, 111);
chat_command_trampoline!(chat_command_112, 112);
chat_command_trampoline!(chat_command_113, 113);
chat_command_trampoline!(chat_command_114, 114);
chat_command_trampoline!(chat_command_115, 115);
chat_command_trampoline!(chat_command_116, 116);
chat_command_trampoline!(chat_command_117, 117);
chat_command_trampoline!(chat_command_118, 118);
chat_command_trampoline!(chat_command_119, 119);
chat_command_trampoline!(chat_command_120, 120);
chat_command_trampoline!(chat_command_121, 121);
chat_command_trampoline!(chat_command_122, 122);
chat_command_trampoline!(chat_command_123, 123);
chat_command_trampoline!(chat_command_124, 124);
chat_command_trampoline!(chat_command_125, 125);
chat_command_trampoline!(chat_command_126, 126);
chat_command_trampoline!(chat_command_127, 127);
chat_command_trampoline!(chat_command_128, 128);
chat_command_trampoline!(chat_command_129, 129);
chat_command_trampoline!(chat_command_130, 130);
chat_command_trampoline!(chat_command_131, 131);
chat_command_trampoline!(chat_command_132, 132);
chat_command_trampoline!(chat_command_133, 133);
chat_command_trampoline!(chat_command_134, 134);
chat_command_trampoline!(chat_command_135, 135);
chat_command_trampoline!(chat_command_136, 136);
chat_command_trampoline!(chat_command_137, 137);
chat_command_trampoline!(chat_command_138, 138);
chat_command_trampoline!(chat_command_139, 139);
chat_command_trampoline!(chat_command_140, 140);
chat_command_trampoline!(chat_command_141, 141);
chat_command_trampoline!(chat_command_142, 142);
chat_command_trampoline!(chat_command_143, 143);

const CHAT_COMMAND_TRAMPOLINES: [unsafe extern "cdecl" fn(*const i8); MAX_CHAT_COMMANDS] = [
    chat_command_0,
    chat_command_1,
    chat_command_2,
    chat_command_3,
    chat_command_4,
    chat_command_5,
    chat_command_6,
    chat_command_7,
    chat_command_8,
    chat_command_9,
    chat_command_10,
    chat_command_11,
    chat_command_12,
    chat_command_13,
    chat_command_14,
    chat_command_15,
    chat_command_16,
    chat_command_17,
    chat_command_18,
    chat_command_19,
    chat_command_20,
    chat_command_21,
    chat_command_22,
    chat_command_23,
    chat_command_24,
    chat_command_25,
    chat_command_26,
    chat_command_27,
    chat_command_28,
    chat_command_29,
    chat_command_30,
    chat_command_31,
    chat_command_32,
    chat_command_33,
    chat_command_34,
    chat_command_35,
    chat_command_36,
    chat_command_37,
    chat_command_38,
    chat_command_39,
    chat_command_40,
    chat_command_41,
    chat_command_42,
    chat_command_43,
    chat_command_44,
    chat_command_45,
    chat_command_46,
    chat_command_47,
    chat_command_48,
    chat_command_49,
    chat_command_50,
    chat_command_51,
    chat_command_52,
    chat_command_53,
    chat_command_54,
    chat_command_55,
    chat_command_56,
    chat_command_57,
    chat_command_58,
    chat_command_59,
    chat_command_60,
    chat_command_61,
    chat_command_62,
    chat_command_63,
    chat_command_64,
    chat_command_65,
    chat_command_66,
    chat_command_67,
    chat_command_68,
    chat_command_69,
    chat_command_70,
    chat_command_71,
    chat_command_72,
    chat_command_73,
    chat_command_74,
    chat_command_75,
    chat_command_76,
    chat_command_77,
    chat_command_78,
    chat_command_79,
    chat_command_80,
    chat_command_81,
    chat_command_82,
    chat_command_83,
    chat_command_84,
    chat_command_85,
    chat_command_86,
    chat_command_87,
    chat_command_88,
    chat_command_89,
    chat_command_90,
    chat_command_91,
    chat_command_92,
    chat_command_93,
    chat_command_94,
    chat_command_95,
    chat_command_96,
    chat_command_97,
    chat_command_98,
    chat_command_99,
    chat_command_100,
    chat_command_101,
    chat_command_102,
    chat_command_103,
    chat_command_104,
    chat_command_105,
    chat_command_106,
    chat_command_107,
    chat_command_108,
    chat_command_109,
    chat_command_110,
    chat_command_111,
    chat_command_112,
    chat_command_113,
    chat_command_114,
    chat_command_115,
    chat_command_116,
    chat_command_117,
    chat_command_118,
    chat_command_119,
    chat_command_120,
    chat_command_121,
    chat_command_122,
    chat_command_123,
    chat_command_124,
    chat_command_125,
    chat_command_126,
    chat_command_127,
    chat_command_128,
    chat_command_129,
    chat_command_130,
    chat_command_131,
    chat_command_132,
    chat_command_133,
    chat_command_134,
    chat_command_135,
    chat_command_136,
    chat_command_137,
    chat_command_138,
    chat_command_139,
    chat_command_140,
    chat_command_141,
    chat_command_142,
    chat_command_143,
];

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "system" fn callback(
        _user_data: *mut c_void,
        _args: *const u8,
        _args_len: usize,
    ) {
    }

    #[test]
    fn registry_reserves_distinct_slots_and_releases_failed_registrations() {
        let registry = ChatCommandRegistry::new();
        let first = registry.reserve(1, b"first".to_vec(), callback, 0).unwrap();
        let second = registry
            .reserve(2, b"second".to_vec(), callback, 0)
            .unwrap();
        assert_ne!(first.slot, second.slot);
        assert!(matches!(
            registry.reserve(3, b"first".to_vec(), callback, 0),
            Err(SampClientSdkResult::InvalidArgument)
        ));

        registry.finish_registration(1, false);
        assert!(registry.entry(1).is_none());
        assert!(registry.entry_for_slot(usize::from(first.slot)).is_none());
    }

    #[test]
    fn registration_and_removal_publish_only_live_callbacks() {
        let registry = ChatCommandRegistry::new();
        let entry = registry
            .reserve(7, b"fixture".to_vec(), callback, 0)
            .unwrap();
        assert!(entry.enter_dispatch().is_none());

        registry.finish_registration(7, true);
        assert!(entry.enter_dispatch().is_some());

        entry.deactivate();
        assert!(entry.enter_dispatch().is_none());
        registry.finish_unregistration(7, false);
        assert!(entry.enter_dispatch().is_some());

        registry.finish_unregistration(7, true);
        assert!(registry.entry(7).is_none());
    }
}
