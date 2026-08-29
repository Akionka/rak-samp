use super::{
    AimSyncSnapshot, AnimationSnapshot, ChatEntrySnapshot, DirectClientError, GangzoneSnapshot,
    InCarSyncSnapshot, LocalDialogResponseSnapshot, LocalDialogSnapshot, LocalPlayerSnapshot,
    OnFootSyncSnapshot, PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot,
    Runtime, ServerInfoSnapshot, TextLabelSnapshot, TextdrawSnapshot, TrailerSyncSnapshot, Vector3,
};
impl Runtime {
    pub(crate) fn local_player(&self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        self.backend.local_player()
    }

    pub(crate) fn player_info(
        &self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        self.backend.player_info(id)
    }

    pub(crate) fn remote_player_state(
        &self,
        id: u16,
    ) -> Result<Option<RemotePlayerStateSnapshot>, DirectClientError> {
        self.backend.remote_player_state(id)
    }

    pub(crate) fn streamed_out_player_position(
        &self,
        id: u16,
    ) -> Result<Option<Vector3>, DirectClientError> {
        self.backend.streamed_out_player_position(id)
    }

    pub(crate) fn onfoot_sync(
        &self,
        id: u16,
    ) -> Result<Option<OnFootSyncSnapshot>, DirectClientError> {
        self.backend.onfoot_sync(id)
    }

    pub(crate) fn vehicle_sync(
        &self,
        id: u16,
    ) -> Result<Option<InCarSyncSnapshot>, DirectClientError> {
        self.backend.vehicle_sync(id)
    }

    pub(crate) fn passenger_sync(
        &self,
        id: u16,
    ) -> Result<Option<PassengerSyncSnapshot>, DirectClientError> {
        self.backend.passenger_sync(id)
    }

    pub(crate) fn trailer_sync(
        &self,
        id: u16,
    ) -> Result<Option<TrailerSyncSnapshot>, DirectClientError> {
        self.backend.trailer_sync(id)
    }
    pub(crate) fn aim_sync(&self, id: u16) -> Result<Option<AimSyncSnapshot>, DirectClientError> {
        self.backend.aim_sync(id)
    }

    pub(crate) fn player_defined(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.player_defined(id)
    }

    pub(crate) fn player_paused(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.player_paused(id)
    }

    pub(crate) fn player_count(&self, include_npcs: bool) -> Result<u16, DirectClientError> {
        self.backend.player_count(include_npcs)
    }

    pub(crate) fn player_max_id(&self) -> Result<u16, DirectClientError> {
        self.backend.player_max_id()
    }

    pub(crate) fn vehicle_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.vehicle_exists(id)
    }

    pub(crate) fn text_label_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.text_label_exists(id)
    }

    pub(crate) fn textdraw_exists(&self, pool_index: u16) -> Result<bool, DirectClientError> {
        self.backend.textdraw_exists(pool_index)
    }

    pub(crate) fn object_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.object_exists(id)
    }

    pub(crate) fn gangzone(&self, id: u16) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        self.backend.gangzone(id)
    }

    pub(crate) fn text_label(
        &self,
        id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        self.backend.text_label(id)
    }

    pub(crate) fn textdraw(
        &self,
        pool_index: u16,
    ) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        self.backend.textdraw(pool_index)
    }

    pub(crate) fn chat_entry(&self, id: u16) -> Result<ChatEntrySnapshot, DirectClientError> {
        self.backend.chat_entry(id)
    }

    pub(crate) fn server_info(&self) -> Result<ServerInfoSnapshot, DirectClientError> {
        self.backend.server_info()
    }

    pub(crate) fn samp_game_state(&self) -> Result<i32, DirectClientError> {
        self.backend.samp_game_state()
    }

    pub(crate) fn local_chat_display_mode(&self) -> Result<i32, DirectClientError> {
        self.backend.local_chat_display_mode()
    }

    pub(crate) fn local_cursor_mode(&self) -> Result<i32, DirectClientError> {
        self.backend.local_cursor_mode()
    }

    pub(crate) fn local_scoreboard_open(&self) -> Result<bool, DirectClientError> {
        self.backend.local_scoreboard_open()
    }

    pub(crate) fn local_dialog_active(&self) -> Result<bool, DirectClientError> {
        self.backend.local_dialog_active()
    }

    pub(crate) fn local_dialog_state(
        &self,
    ) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        self.backend.local_dialog_state()
    }

    pub(crate) fn take_local_dialog_response(
        &self,
    ) -> Result<Option<LocalDialogResponseSnapshot>, DirectClientError> {
        self.backend.take_local_dialog_response()
    }

    pub(crate) fn object_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.backend.object_handle(id)
    }

    pub(crate) fn object_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.object_id_by_handle(handle)
    }

    pub(crate) fn pickup_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.backend.pickup_handle(id)
    }

    pub(crate) fn pickup_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.pickup_id_by_handle(handle)
    }

    pub(crate) fn vehicle_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.backend.vehicle_handle(id)
    }

    pub(crate) fn vehicle_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.vehicle_id_by_handle(handle)
    }

    pub(crate) fn player_ped_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.backend.player_ped_handle(id)
    }

    pub(crate) fn player_id_by_ped_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.player_id_by_ped_handle(handle)
    }

    pub(crate) fn local_dialog_selected_item(&self) -> Result<i32, DirectClientError> {
        self.backend.local_dialog_selected_item()
    }

    pub(crate) fn local_dialog_list_item_count(&self) -> Result<i32, DirectClientError> {
        self.backend.local_dialog_list_item_count()
    }

    pub(crate) fn local_chat_input_active(&self) -> Result<bool, DirectClientError> {
        self.backend.local_chat_input_active()
    }

    pub(crate) fn local_chat_input_text(&self) -> Result<Vec<u8>, DirectClientError> {
        self.backend.local_chat_input_text()
    }

    pub(crate) fn local_chat_command_defined(
        &self,
        name: &[u8],
    ) -> Result<bool, DirectClientError> {
        self.backend.local_chat_command_defined(name)
    }

    pub(crate) fn local_animation(&self, id: u16) -> Result<AnimationSnapshot, DirectClientError> {
        self.backend.local_animation(id)
    }

    pub(crate) fn local_animation_id(
        &self,
        name: &[u8],
        file: &[u8],
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.local_animation_id(name, file)
    }
}
