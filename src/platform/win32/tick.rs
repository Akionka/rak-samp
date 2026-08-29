//! Game-thread tick orchestration.

use super::*;

impl BackendState {
    pub(super) fn prepare_game_tick(&self) -> Option<Vec<QueuedCommand<GameCommand>>> {
        (self.rak_client.load(Ordering::Acquire) != 0)
            .then(|| self.game_commands.take_tick_snapshot())
    }

    /// Executes one post-process game tick. `commands` is captured before the
    /// native process call, so submissions made while that call or this drain
    /// is running remain owned by the following tick.
    pub(super) fn pump_game_tick(&self, commands: Vec<QueuedCommand<GameCommand>>) {
        self.execute_game_commands(commands);
        let Some(connection_profile) = self.connection_profile() else {
            return;
        };
        // Odd generations are in-flight. Readers only observe the next even
        // generation after every cache path below has had one tick to refresh.
        self.cache_generation.fetch_add(1, Ordering::AcqRel);
        self.refresh_samp_game_state(connection_profile);
        self.refresh_server_info_snapshot(connection_profile);
        self.refresh_player_info(connection_profile);
        self.refresh_remote_player_state(connection_profile);
        self.refresh_streamed_out_player_position(connection_profile);
        self.refresh_onfoot_sync(connection_profile);
        self.refresh_incar_sync(connection_profile);
        self.refresh_passenger_sync(connection_profile);
        self.refresh_trailer_sync(connection_profile);
        self.refresh_aim_sync(connection_profile);
        self.refresh_vehicle_exists(connection_profile);
        self.refresh_object_exists(connection_profile);
        self.refresh_gangzones(connection_profile);
        self.refresh_object_handles(connection_profile);
        self.refresh_pickup_handles(connection_profile);
        self.refresh_vehicle_handles(connection_profile);
        self.refresh_player_handles(connection_profile);
        self.refresh_object_handle_ids(connection_profile);
        self.refresh_pickup_handle_ids(connection_profile);
        self.refresh_vehicle_handle_ids(connection_profile);
        self.refresh_player_handle_ids(connection_profile);
        self.refresh_local_chat_display_mode(connection_profile);
        self.refresh_local_cursor_mode(connection_profile);
        self.refresh_local_scoreboard_open(connection_profile);
        self.refresh_local_dialog_active(connection_profile);
        self.refresh_local_dialog_state(connection_profile);
        self.refresh_local_chat_input_active(connection_profile);
        self.refresh_local_chat_input_commands(connection_profile);
        self.refresh_local_chat_input_text(connection_profile);
        self.refresh_chat_entries(connection_profile);
        self.refresh_text_label_exists(connection_profile);
        self.refresh_text_labels(connection_profile);
        self.refresh_local_player_snapshot(Some(connection_profile));
        self.refresh_player_count(connection_profile);
        self.refresh_player_max_id(connection_profile);
        self.refresh_animation_catalog(connection_profile);
        self.refresh_raw_pool_addresses(connection_profile);
        self.refresh_textdraw_exists(connection_profile);
        self.refresh_textdraws(connection_profile);
        self.cache_generation.fetch_add(1, Ordering::Release);
    }

    pub(super) fn is_game_thread(&self) -> bool {
        let game_thread = self.game_thread_id.load(Ordering::Acquire);
        game_thread != 0 && game_thread == unsafe { GetCurrentThreadId() }
    }

    pub(super) unsafe fn run_game_process_tick(&self, original: GameProcessFn) {
        // Publish this before entering GTA so a plugin reached from the native
        // process path cannot block the game thread on its own command receipt.
        self.game_thread_id
            .store(unsafe { GetCurrentThreadId() }, Ordering::Release);
        let commands = self.prepare_game_tick();
        if let Some(commands) = commands.as_ref().filter(|commands| !commands.is_empty())
            && !self
                .game_command_snapshot_diagnostic_logged
                .swap(true, Ordering::AcqRel)
        {
            let first_id = commands[0].id;
            let last_id = commands.last().map_or(first_id, |command| command.id);
            // Snapshot metadata lets a live smoke prove the command crossed
            // the game-thread boundary without exposing plugin payloads.
            log::debug!(
                "captured first game command snapshot: count={}, first_id={first_id}, last_id={last_id}",
                commands.len(),
            );
        }
        unsafe { original() };
        if let Some(commands) = commands {
            self.pump_game_tick(commands);
        }
    }
}
