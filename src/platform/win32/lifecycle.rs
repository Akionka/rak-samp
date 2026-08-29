//! Backend attachment, hook installation, and shutdown.

use super::*;

pub(super) static ACTIVE_BACKEND: OnceLock<Mutex<Option<Weak<BackendState>>>> = OnceLock::new();

pub(crate) fn attach(registry: Arc<Registry>) -> Result<Backend, AttachError> {
    let module_base = loaded_samp_module()?;
    let entry_point = unsafe { modkit_win32::pe_entry_point(module_base) }
        .ok_or(AttachError::UnsupportedClient { entry_point: 0 })?;
    let version = SampVersion::from_entry_point(entry_point)
        .ok_or(AttachError::UnsupportedClient { entry_point })?;
    let addresses = AddressSet::for_version(version);
    let selected_profile = NativeClientProfile::select(module_base, version, entry_point);
    if let Some(profile) = selected_profile {
        log::info!(
            "{} direct client helpers are enabled",
            profile.spec.identity.name
        );
    }

    let active = ACTIVE_BACKEND.get_or_init(|| Mutex::new(None));
    let mut active = active.lock().unwrap_or_else(|error| error.into_inner());
    if active.as_ref().and_then(Weak::upgrade).is_some() {
        return Err(AttachError::AlreadyAttached);
    }

    let state = Arc::new(BackendState {
        context: BackendContext {
            registry,
            module_base,
            version,
            addresses,
            native_client_profile: selected_profile,
        },
        game_tick: GameTickRuntime::new(GtaProfile::gta_sa_10_us()),
        rak_client: AtomicUsize::new(0),
        raw_player_pool: AtomicUsize::new(0),
        raw_vehicle_pool: AtomicUsize::new(0),
        raw_local_player: AtomicUsize::new(0),
        rpc_receiver: AtomicUsize::new(0),
        player_address: AtomicU32::new(0),
        player_port: AtomicU16::new(0),
        constructor_trampoline: AtomicUsize::new(0),
        incoming_rpc_trampoline: AtomicUsize::new(0),
        dialog_close_trampoline: AtomicUsize::new(0),
        outgoing_packet_original: AtomicUsize::new(0),
        incoming_packet_original: AtomicUsize::new(0),
        deallocate_packet_original: AtomicUsize::new(0),
        outgoing_rpc_original: AtomicUsize::new(0),
        client_hook_status: AtomicU32::new(ClientHookInstallState::Pending.as_raw()),
        incoming_packet_diagnostic_logged: AtomicBool::new(false),
        game_command_snapshot_diagnostic_logged: AtomicBool::new(false),
        game_command_completion_diagnostic_logged: AtomicBool::new(false),
        string_codec: Mutex::new(()),
        pending_game_tick: Mutex::new(None),
        game_commands: CommandQueue::new(),
        auto_text_label_creates: Mutex::new(HashMap::new()),
        local_player_snapshot: Mutex::new(None),
        local_player_candidate: Mutex::new(None),
        player_info_cache: Mutex::new(vec![PlayerInfoCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        player_info_requests: Mutex::new(VecDeque::with_capacity(
            PLAYER_INFO_REQUEST_QUEUE_CAPACITY,
        )),
        remote_player_state_cache: Mutex::new(vec![
            RemotePlayerStateCacheEntry::Unknown;
            MAX_SAMP_PLAYERS
        ]),
        remote_player_state_requests: Mutex::new(VecDeque::with_capacity(
            REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY,
        )),
        marker_sync_positions: Mutex::new(vec![None; MAX_SAMP_PLAYERS]),
        streamed_out_player_position_cache: Mutex::new(vec![
            StreamedOutPlayerPositionCacheEntry::Unknown;
            MAX_SAMP_PLAYERS
        ]),
        streamed_out_player_position_requests: Mutex::new(VecDeque::with_capacity(
            STREAMED_OUT_PLAYER_POSITION_REQUEST_QUEUE_CAPACITY,
        )),
        onfoot_sync_cache: Mutex::new(vec![OnFootSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        onfoot_sync_requests: Mutex::new(VecDeque::with_capacity(
            ONFOOT_SYNC_REQUEST_QUEUE_CAPACITY,
        )),
        incar_sync_cache: Mutex::new(vec![InCarSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        incar_sync_requests: Mutex::new(VecDeque::with_capacity(INCAR_SYNC_REQUEST_QUEUE_CAPACITY)),
        passenger_sync_cache: Mutex::new(vec![PassengerSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        passenger_sync_requests: Mutex::new(VecDeque::with_capacity(
            PASSENGER_SYNC_REQUEST_QUEUE_CAPACITY,
        )),
        trailer_sync_cache: Mutex::new(vec![TrailerSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        trailer_sync_requests: Mutex::new(VecDeque::with_capacity(
            TRAILER_SYNC_REQUEST_QUEUE_CAPACITY,
        )),
        aim_sync_cache: Mutex::new(vec![AimSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        aim_sync_requests: Mutex::new(VecDeque::with_capacity(AIM_SYNC_REQUEST_QUEUE_CAPACITY)),
        vehicle_exists_cache: Mutex::new(vec![VehicleExistsCacheEntry::Unknown; MAX_SAMP_VEHICLES]),
        vehicle_exists_requests: Mutex::new(VecDeque::with_capacity(
            VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY,
        )),
        text_label_exists_cache: Mutex::new(vec![
            TextLabelExistsCacheEntry::Unknown;
            MAX_SAMP_TEXT_LABELS
        ]),
        text_label_exists_requests: Mutex::new(VecDeque::with_capacity(
            TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY,
        )),
        text_label_cache: Mutex::new(vec![TextLabelCacheEntry::Unknown; MAX_SAMP_TEXT_LABELS]),
        text_label_requests: Mutex::new(VecDeque::with_capacity(TEXT_LABEL_REQUEST_QUEUE_CAPACITY)),
        textdraw_exists_cache: Mutex::new(vec![
            TextdrawExistsCacheEntry::Unknown;
            MAX_SAMP_TEXTDRAWS
        ]),
        textdraw_exists_requests: Mutex::new(VecDeque::with_capacity(
            TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY,
        )),
        textdraw_cache: Mutex::new(vec![TextdrawCacheEntry::Unknown; MAX_SAMP_TEXTDRAWS]),
        textdraw_requests: Mutex::new(VecDeque::with_capacity(TEXTDRAW_REQUEST_QUEUE_CAPACITY)),
        chat_entry_cache: Mutex::new(vec![ChatEntryCacheEntry::Unknown; MAX_CHAT_ENTRIES]),
        chat_entry_requests: Mutex::new(VecDeque::with_capacity(CHAT_ENTRY_REQUEST_QUEUE_CAPACITY)),
        object_exists_cache: Mutex::new(vec![ObjectExistsCacheEntry::Unknown; MAX_SAMP_OBJECTS]),
        object_exists_requests: Mutex::new(VecDeque::with_capacity(
            OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY,
        )),
        gangzone_cache: Mutex::new(vec![GangzoneCacheEntry::Unknown; MAX_SAMP_GANGZONES]),
        gangzone_requests: Mutex::new(VecDeque::with_capacity(GANGZONE_REQUEST_QUEUE_CAPACITY)),
        object_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_OBJECTS]),
        object_handle_requests: Mutex::new(VecDeque::with_capacity(
            OBJECT_HANDLE_REQUEST_QUEUE_CAPACITY,
        )),
        object_handle_reverse_cache: Mutex::new(HashMap::new()),
        object_handle_reverse_requests: Mutex::new(VecDeque::with_capacity(
            OBJECT_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
        )),
        pickup_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_PICKUPS]),
        pickup_handle_requests: Mutex::new(VecDeque::with_capacity(
            PICKUP_HANDLE_REQUEST_QUEUE_CAPACITY,
        )),
        pickup_handle_reverse_cache: Mutex::new(HashMap::new()),
        pickup_handle_reverse_requests: Mutex::new(VecDeque::with_capacity(
            PICKUP_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
        )),
        vehicle_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_VEHICLES]),
        vehicle_handle_requests: Mutex::new(VecDeque::with_capacity(
            VEHICLE_HANDLE_REQUEST_QUEUE_CAPACITY,
        )),
        vehicle_handle_reverse_cache: Mutex::new(HashMap::new()),
        vehicle_handle_reverse_requests: Mutex::new(VecDeque::with_capacity(
            VEHICLE_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
        )),
        player_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        player_handle_requests: Mutex::new(VecDeque::with_capacity(
            PLAYER_HANDLE_REQUEST_QUEUE_CAPACITY,
        )),
        player_handle_reverse_cache: Mutex::new(HashMap::new()),
        player_handle_reverse_requests: Mutex::new(VecDeque::with_capacity(
            PLAYER_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
        )),
        player_count_including_npcs: AtomicI32::new(0),
        player_count_excluding_npcs: AtomicI32::new(0),
        player_count_ready: AtomicBool::new(false),
        player_max_id: AtomicI32::new(0),
        player_max_id_ready: AtomicBool::new(false),
        server_info_snapshot: Mutex::new(None),
        samp_game_state: AtomicI32::new(0),
        samp_game_state_ready: AtomicBool::new(false),
        local_chat_display_mode: AtomicI32::new(0),
        local_chat_display_mode_ready: AtomicBool::new(false),
        local_cursor_mode: AtomicI32::new(0),
        local_cursor_mode_ready: AtomicBool::new(false),
        local_scoreboard_open: AtomicBool::new(false),
        local_scoreboard_open_ready: AtomicBool::new(false),
        local_dialog_active: AtomicBool::new(false),
        local_dialog_active_ready: AtomicBool::new(false),
        local_dialog_snapshot: Mutex::new(None),
        local_dialog_snapshot_ready: AtomicBool::new(false),
        local_dialog_response: Mutex::new(None),
        local_chat_input_active: AtomicBool::new(false),
        local_chat_input_active_ready: AtomicBool::new(false),
        local_chat_input_text: Mutex::new(None),
        local_chat_input_text_ready: AtomicBool::new(false),
        local_chat_input_commands: Mutex::new(None),
        local_chat_input_commands_ready: AtomicBool::new(false),
        animation_catalog: Mutex::new(None),
        cache_generation: AtomicU64::new(0),
        hooks: Mutex::new(HookStorage::default()),
    });
    *active = Some(Arc::downgrade(&state));
    drop(active);

    let participant: Arc<dyn GameTickParticipant> = Arc::clone(&state) as Arc<_>;
    state
        .game_tick
        .register_participant(Arc::downgrade(&participant));

    if let Err(error) = state.install_game_process_hook() {
        clear_active_backend(&state);
        return Err(error);
    }
    if let Err(error) = state.install_dialog_close_hook() {
        state.shutdown();
        return Err(error);
    }
    if let Err(error) = state.install_constructor_hook() {
        state.shutdown();
        return Err(error);
    }
    Ok(Backend { state })
}

impl BackendState {
    pub(super) fn install_game_process_hook(&self) -> Result<(), AttachError> {
        self.game_tick.install().map_err(|error| match error {
            gta_sa_native::GameTickInstallError::CreateHook => {
                AttachError::HookInstallFailed("CGame::Process detour")
            }
            gta_sa_native::GameTickInstallError::EnableHook => {
                AttachError::HookInstallFailed("enabling CGame::Process detour")
            }
        })
    }

    pub(super) fn install_dialog_close_hook(&self) -> Result<(), AttachError> {
        let Some(profile) = self.connection_profile() else {
            return Ok(());
        };
        let target = profile
            .dialog_close_target()
            .ok_or(AttachError::HookInstallFailed("CDialog::Close target"))?;
        let (mut detour, trampoline) = unsafe {
            InlineHook::create(
                "CDialog::Close",
                target,
                hooks::dialog_close_detour as *const () as usize,
            )
        }
        .map_err(|_| AttachError::HookInstallFailed("CDialog::Close detour"))?;
        self.dialog_close_trampoline
            .store(trampoline, Ordering::Release);
        if detour.enable().is_err() {
            self.dialog_close_trampoline.store(0, Ordering::Release);
            return Err(AttachError::HookInstallFailed(
                "enabling CDialog::Close detour",
            ));
        }
        self.hooks
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .dialog_close = Some(detour);
        Ok(())
    }

    pub(super) fn install_constructor_hook(self: &Arc<Self>) -> Result<(), AttachError> {
        let target = self.module_base + self.addresses.rak_client_constructor as usize;
        let (mut detour, trampoline) = unsafe {
            InlineHook::create(
                "RakClient constructor",
                target,
                hooks::rak_client_constructor_detour as *const () as usize,
            )
        }
        .map_err(|_| AttachError::HookInstallFailed("RakClient constructor detour"))?;
        self.constructor_trampoline
            .store(trampoline, Ordering::Release);
        if detour.enable().is_err() {
            self.constructor_trampoline.store(0, Ordering::Release);
            return Err(AttachError::HookInstallFailed(
                "enabling RakClient constructor detour",
            ));
        }
        self.hooks
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .constructor = Some(detour);
        Ok(())
    }

    pub(super) fn install_client_hooks(&self, client: *mut c_void) -> Result<(), AttachError> {
        if client.is_null() {
            return Err(AttachError::ClientNotReady);
        }
        if self
            .rak_client
            .compare_exchange(0, client as usize, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(());
        }

        let incoming_target = self.module_base + self.addresses.incoming_rpc_handler as usize;
        let (mut incoming_rpc, trampoline) = unsafe {
            InlineHook::create(
                "RakClient::HandleRPCPacket",
                incoming_target,
                hooks::incoming_rpc_detour as *const () as usize,
            )
        }
        .map_err(|_| {
            self.rak_client.store(0, Ordering::Release);
            AttachError::HookInstallFailed("incoming RPC detour")
        })?;
        self.incoming_rpc_trampoline
            .store(trampoline, Ordering::Release);
        if incoming_rpc.enable().is_err() {
            self.incoming_rpc_trampoline.store(0, Ordering::Release);
            self.rak_client.store(0, Ordering::Release);
            return Err(AttachError::HookInstallFailed(
                "enabling incoming RPC detour",
            ));
        }

        let vtable = match unsafe { VtableHook::install(client, self) } {
            Ok(vtable) => vtable,
            Err(error) => {
                incoming_rpc.disable();
                self.incoming_rpc_trampoline.store(0, Ordering::Release);
                self.rak_client.store(0, Ordering::Release);
                self.outgoing_packet_original.store(0, Ordering::Release);
                self.incoming_packet_original.store(0, Ordering::Release);
                self.deallocate_packet_original.store(0, Ordering::Release);
                self.outgoing_rpc_original.store(0, Ordering::Release);
                return Err(error);
            }
        };
        let mut hooks = self.hooks.lock().unwrap_or_else(|error| error.into_inner());
        hooks.incoming_rpc = Some(incoming_rpc);
        hooks.vtable = Some(vtable);
        self.client_hook_status
            .store(ClientHookInstallState::Ready.as_raw(), Ordering::Release);
        Ok(())
    }
}

impl BackendState {
    pub(super) fn shutdown(&self) {
        let mut hooks = self.hooks.lock().unwrap_or_else(|error| error.into_inner());
        hooks.vtable.take();
        self.game_tick.shutdown();
        if let Some(detour) = hooks.dialog_close.take() {
            detour.disable();
        }
        if let Some(detour) = hooks.incoming_rpc.take() {
            detour.disable();
        }
        if let Some(detour) = hooks.constructor.take() {
            detour.disable();
        }
        drop(hooks);

        // No new native calls can enter after the GTA runtime, vtable, and
        // SA-MP inline hooks have been removed. Existing detours retain their
        // runtime/backend state until their captured originals return.
        clear_active_backend(self);
        self.dialog_close_trampoline.store(0, Ordering::Release);
        self.rak_client.store(0, Ordering::Release);
        self.raw_player_pool.store(0, Ordering::Release);
        self.raw_vehicle_pool.store(0, Ordering::Release);
        self.raw_local_player.store(0, Ordering::Release);
        self.game_commands.shutdown();
        if let Ok(mut snapshot) = self.local_player_snapshot.try_lock() {
            *snapshot = None;
        }
        if let Ok(mut candidate) = self.local_player_candidate.try_lock() {
            *candidate = None;
        }
        self.clear_player_info_cache();
        if let Ok(mut requests) = self.player_info_requests.try_lock() {
            requests.clear();
        }
        self.clear_remote_player_state_cache();
        if let Ok(mut requests) = self.remote_player_state_requests.try_lock() {
            requests.clear();
        }
        self.clear_streamed_out_player_position_cache();
        self.clear_marker_sync_positions();
        if let Ok(mut requests) = self.streamed_out_player_position_requests.try_lock() {
            requests.clear();
        }
        self.clear_onfoot_sync_cache();
        if let Ok(mut requests) = self.onfoot_sync_requests.try_lock() {
            requests.clear();
        }
        self.clear_incar_sync_cache();
        if let Ok(mut requests) = self.incar_sync_requests.try_lock() {
            requests.clear();
        }
        self.clear_passenger_sync_cache();
        if let Ok(mut requests) = self.passenger_sync_requests.try_lock() {
            requests.clear();
        }
        self.clear_trailer_sync_cache();
        if let Ok(mut requests) = self.trailer_sync_requests.try_lock() {
            requests.clear();
        }
        self.clear_aim_sync_cache();
        if let Ok(mut requests) = self.aim_sync_requests.try_lock() {
            requests.clear();
        }
        self.clear_vehicle_exists_cache();
        if let Ok(mut requests) = self.vehicle_exists_requests.try_lock() {
            requests.clear();
        }
        self.clear_text_label_exists_cache();
        if let Ok(mut requests) = self.text_label_exists_requests.try_lock() {
            requests.clear();
        }
        self.clear_text_label_cache();
        if let Ok(mut requests) = self.text_label_requests.try_lock() {
            requests.clear();
        }
        self.clear_textdraw_exists_cache();
        if let Ok(mut requests) = self.textdraw_exists_requests.try_lock() {
            requests.clear();
        }
        self.clear_textdraw_cache();
        if let Ok(mut requests) = self.textdraw_requests.try_lock() {
            requests.clear();
        }
        self.clear_chat_entry_cache();
        if let Ok(mut requests) = self.chat_entry_requests.try_lock() {
            requests.clear();
        }
        self.clear_object_exists_cache();
        if let Ok(mut requests) = self.object_exists_requests.try_lock() {
            requests.clear();
        }
        self.clear_gangzone_cache();
        if let Ok(mut requests) = self.gangzone_requests.try_lock() {
            requests.clear();
        }
        if let Ok(mut snapshot) = self.server_info_snapshot.try_lock() {
            *snapshot = None;
        }
        self.samp_game_state_ready.store(false, Ordering::Release);
        self.local_chat_display_mode_ready
            .store(false, Ordering::Release);
        self.local_cursor_mode_ready.store(false, Ordering::Release);
        self.local_scoreboard_open_ready
            .store(false, Ordering::Release);
        self.local_dialog_active_ready
            .store(false, Ordering::Release);
        if let Ok(mut snapshot) = self.local_dialog_snapshot.try_lock() {
            *snapshot = None;
        }
        if let Ok(mut response) = self.local_dialog_response.try_lock() {
            *response = None;
        }
        self.local_dialog_snapshot_ready
            .store(false, Ordering::Release);
        self.local_chat_input_active_ready
            .store(false, Ordering::Release);
        self.local_chat_input_text_ready
            .store(false, Ordering::Release);
        if let Ok(mut snapshot) = self.local_chat_input_text.try_lock() {
            *snapshot = None;
        }
        self.local_chat_input_commands_ready
            .store(false, Ordering::Release);
        if let Ok(mut snapshot) = self.local_chat_input_commands.try_lock() {
            *snapshot = None;
        }
        self.player_count_ready.store(false, Ordering::Release);
        self.player_max_id_ready.store(false, Ordering::Release);
        if let Ok(mut catalog) = self.animation_catalog.try_lock() {
            *catalog = None;
        }
    }
}

pub(super) fn active_state() -> Option<Arc<BackendState>> {
    ACTIVE_BACKEND.get().and_then(|slot| {
        slot.lock()
            .ok()
            .and_then(|state| state.as_ref().and_then(Weak::upgrade))
    })
}

pub(super) fn clear_active_backend(target: &BackendState) {
    let Some(slot) = ACTIVE_BACKEND.get() else {
        return;
    };
    let mut active = slot.lock().unwrap_or_else(|error| error.into_inner());
    if active
        .as_ref()
        .and_then(Weak::upgrade)
        .is_some_and(|state| ptr::eq(Arc::as_ptr(&state), target))
    {
        *active = None;
    }
}

fn loaded_samp_module() -> Result<usize, AttachError> {
    modkit_win32::loaded_module("samp.dll").ok_or(AttachError::SampNotLoaded)
}
