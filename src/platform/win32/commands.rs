//! Game-thread command submission and execution helpers.

use super::*;

impl BackendState {
    pub(super) fn queue_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::ShowDialog(request))
    }

    pub(super) fn queue_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::AddChatMessage(request))
    }

    pub(super) fn queue_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::AddDeathMessage(request))
    }

    pub(super) fn queue_game_command(
        &self,
        command: GameCommand,
    ) -> Result<CommandId, DirectClientError> {
        self.submit_game_command(command)
            .map_err(|error| match error {
                CommandError::QueueFull => DirectClientError::QueueFull,
                CommandError::ShuttingDown
                | CommandError::NativeFailure
                | CommandError::UnknownReceipt
                | CommandError::TimedOut
                | CommandError::WaitRejected => DirectClientError::NotReady,
            })
    }

    pub(super) fn submit_game_command(
        &self,
        command: GameCommand,
    ) -> Result<CommandId, CommandError> {
        self.game_commands.submit(command)
    }

    pub(super) fn queue_network_command(&self, command: GameCommand) -> Result<bool, SendError> {
        let id = self.submit_network_command(command)?;
        self.game_commands.detach(id).map_err(command_send_error)?;
        Ok(true)
    }

    pub(super) fn submit_network_command(
        &self,
        command: GameCommand,
    ) -> Result<CommandId, SendError> {
        self.submit_game_command(command)
            .map_err(command_send_error)
    }

    pub(super) fn send_packet(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::SendPacket {
            id: packet_id,
            payload: payload.clone(),
            options,
        })
    }

    pub(super) fn send_rpc(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::SendRpc {
            id: rpc_id,
            payload: payload.clone(),
            options,
        })
    }

    pub(super) fn submit_packet(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::SendPacket {
            id: packet_id,
            payload: payload.clone(),
            options,
        })
    }

    pub(super) fn submit_rpc(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::SendRpc {
            id: rpc_id,
            payload: payload.clone(),
            options,
        })
    }

    pub(super) fn emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::EmulateIncomingPacket {
            id: packet_id,
            payload,
        })
    }

    pub(super) fn emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.queue_network_command(GameCommand::EmulateIncomingRpc {
            id: rpc_id,
            payload,
        })
    }

    pub(super) fn submit_emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::EmulateIncomingPacket {
            id: packet_id,
            payload,
        })
    }

    pub(super) fn submit_emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.submit_network_command(GameCommand::EmulateIncomingRpc {
            id: rpc_id,
            payload,
        })
    }

    pub(super) fn show_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<(), DirectClientError> {
        let id = self.submit_local_dialog(request)?;
        self.game_commands
            .detach(id)
            .map_err(|_| DirectClientError::NotReady)
    }

    pub(super) fn show_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        let id = self.submit_local_chat_message(request)?;
        self.game_commands
            .detach(id)
            .map_err(|_| DirectClientError::NotReady)
    }

    pub(super) fn show_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        let id = self.submit_local_death_message(request)?;
        self.game_commands
            .detach(id)
            .map_err(|_| DirectClientError::NotReady)
    }

    pub(super) fn submit_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_local_dialog(request)
    }

    pub(super) fn submit_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_local_chat_message(request)
    }

    pub(super) fn submit_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_local_death_message(request)
    }

    pub(super) fn submit_local_cursor_mode(
        &self,
        mode: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !matches!(mode, 0..=4) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetCursorMode(mode))
    }

    pub(super) fn submit_local_chat_display_mode(
        &self,
        mode: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !matches!(mode, 0..=2) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetChatDisplayMode(mode))
    }

    pub(super) fn submit_local_chat_entry(
        &self,
        id: u16,
        text: Vec<u8>,
        prefix: Vec<u8>,
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || id >= 100
            || text.len() >= 144
            || prefix.len() >= 28
            || text.contains(&0)
            || prefix.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetChatEntry {
            id,
            text,
            prefix,
            text_colour,
            prefix_colour,
        })
    }

    pub(super) fn submit_local_dialog_close(
        &self,
        button: u8,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || button > 1 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::CloseDialog(button))
    }

    pub(super) fn submit_local_chat_input_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetChatInputText(text))
    }

    pub(super) fn submit_local_chat_input_enabled(
        &self,
        enabled: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetChatInputEnabled(enabled))
    }

    pub(super) fn submit_local_chat_input_process(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ProcessChatInput(text))
    }

    pub(super) fn submit_register_chat_command(
        &self,
        subscription: u64,
        slot: u8,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || subscription == 0
            || usize::from(slot) >= 144
            || name.is_empty()
            || name.len() > 32
            || name.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::RegisterChatCommand {
            subscription,
            slot,
            name,
        })
    }

    pub(super) fn submit_unregister_chat_command(
        &self,
        subscription: u64,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || subscription == 0
            || name.is_empty()
            || name.len() > 32
            || name.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::UnregisterChatCommand { subscription, name })
    }

    pub(super) fn submit_local_cursor_toggle(
        &self,
        show: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ToggleCursor(show))
    }

    pub(super) fn submit_local_scoreboard_open(
        &self,
        open: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetScoreboardOpen(open))
    }

    pub(super) fn submit_local_dialog_client_side(
        &self,
        client_side: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetDialogClientSide(client_side))
    }

    pub(super) fn submit_local_dialog_selected_item(
        &self,
        selected: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetDialogSelectedItem(selected))
    }

    pub(super) fn submit_local_dialog_editbox_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetDialogEditboxText(text))
    }

    pub(super) fn submit_samp_game_state(
        &self,
        state: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !matches!(state, 0 | 9 | 13 | 14 | 15 | 18)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetGameState(state))
    }

    pub(super) fn submit_connect_to_server(
        &self,
        address: Vec<u8>,
        port: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || address.is_empty()
            || address.len() > 256
            || address.contains(&0)
            || port == 0
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ConnectToServer { address, port })
    }

    pub(super) fn submit_disconnect_with_reason(
        &self,
        block_duration: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::DisconnectWithReason(block_duration))
    }

    pub(super) fn submit_delete_textdraw(&self, id: u16) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::DeleteTextdraw(id))
    }

    pub(super) fn submit_delete_text_label(&self, id: u16) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::DeleteTextLabel(id))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn submit_create_text_label(
        &self,
        id: u16,
        text: Vec<u8>,
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none()
            || self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXT_LABELS
            || text.len() > MAX_SAMP_TEXT_LABEL_TEXT_BYTES
            || text.contains(&0)
            || !position.x.is_finite()
            || !position.y.is_finite()
            || !position.z.is_finite()
            || !draw_distance.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::CreateTextLabel {
            id,
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id,
            attached_vehicle_id,
        })
    }

    pub(super) fn submit_set_textdraw_position(
        &self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !x.is_finite()
            || !y.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawPosition { id, x, y })
    }

    pub(super) fn submit_set_textdraw_letter_style(
        &self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !width.is_finite()
            || !height.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawLetterStyle {
            id,
            width,
            height,
            colour,
        })
    }

    pub(super) fn submit_set_textdraw_proportional(
        &self,
        id: u16,
        proportional: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawProportional { id, proportional })
    }

    pub(super) fn submit_set_textdraw_shadow(
        &self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawShadow { id, shadow, colour })
    }

    pub(super) fn submit_set_textdraw_outline(
        &self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawOutline {
            id,
            outline,
            colour,
        })
    }

    pub(super) fn submit_set_textdraw_box(
        &self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !width.is_finite()
            || !height.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawBox {
            id,
            enabled,
            colour,
            width,
            height,
        })
    }

    pub(super) fn submit_set_textdraw_alignment(
        &self,
        id: u16,
        alignment: u8,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !(1..=3).contains(&alignment)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawAlignment { id, alignment })
    }

    pub(super) fn submit_set_textdraw_string(
        &self,
        id: u16,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || text.len() > 1_601
            || text.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawString { id, text })
    }

    pub(super) fn submit_set_textdraw_model_style(
        &self,
        id: u16,
        rotation: crate::runtime::Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !rotation.x.is_finite()
            || !rotation.y.is_finite()
            || !rotation.z.is_finite()
            || !zoom.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawModelStyle {
            id,
            rotation,
            zoom,
            colour1,
            colour2,
        })
    }

    pub(super) fn submit_local_player_spawn(&self) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SpawnLocalPlayer)
    }

    pub(super) fn submit_local_player_special_action(
        &self,
        action: u8,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !matches!(action, 0..=12 | 20..=25 | 68)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetLocalPlayerSpecialAction(action))
    }

    pub(super) fn submit_local_player_name(
        &self,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || name.len() > 255 || name.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetLocalPlayerName(name))
    }

    pub(super) fn submit_force_unoccupied_sync(
        &self,
        vehicle: u16,
        seat: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(vehicle) >= MAX_SAMP_VEHICLES
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ForceUnoccupiedSync { vehicle, seat })
    }

    pub(super) fn submit_send_rate(
        &self,
        kind: u8,
        milliseconds: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !matches!(kind, 0..=2)
            || i32::try_from(milliseconds).is_err()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetSendRate { kind, milliseconds })
    }

    pub(super) fn submit_player_colour(
        &self,
        id: u16,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetPlayerColour { id, colour })
    }

    pub(super) fn execute_game_commands(&self, commands: Vec<QueuedCommand<GameCommand>>) {
        for queued in commands {
            let result = match queued.command {
                GameCommand::ShowDialog(request) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .dialog_is_ready()
                            .then_some(())
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|()| {
                                profile
                                    .show_dialog(request)
                                    .map_err(|_| CommandError::NativeFailure)
                            })
                    }),
                GameCommand::CloseDialog(button) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .close_dialog(button)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetChatInputText(text) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_input_text(&text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetChatInputEnabled(enabled) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_input_enabled(enabled)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ProcessChatInput(text) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .process_chat_input(&text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::RegisterChatCommand {
                    subscription,
                    slot,
                    name,
                } => {
                    let result =
                        self.r1_client
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|profile| {
                                profile
                                    .register_chat_command(
                                        &name,
                                        crate::host_api::chat_commands::trampoline(slot),
                                    )
                                    .map_err(|_| CommandError::NativeFailure)
                            });
                    crate::host_api::chat_commands::finish_registration(
                        subscription,
                        result.is_ok(),
                    );
                    result
                }
                GameCommand::UnregisterChatCommand { subscription, name } => {
                    let result =
                        self.r1_client
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|profile| {
                                profile
                                    .unregister_chat_command(&name)
                                    .map_err(|_| CommandError::NativeFailure)
                            });
                    crate::host_api::chat_commands::finish_unregistration(
                        subscription,
                        result.is_ok(),
                    );
                    result
                }
                GameCommand::SetChatDisplayMode(mode) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_display_mode(mode)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetChatEntry {
                    id,
                    text,
                    prefix,
                    text_colour,
                    prefix_colour,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_entry(id, &text, &prefix, text_colour, prefix_colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::AddChatMessage(request) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .chat_is_ready()
                            .then_some(())
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|()| {
                                profile
                                    .show_chat_message(request)
                                    .map_err(|_| CommandError::NativeFailure)
                            })
                    }),
                GameCommand::AddDeathMessage(request) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .death_window_is_ready()
                            .then_some(())
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|()| {
                                profile
                                    .show_death_message(request)
                                    .map_err(|_| CommandError::NativeFailure)
                            })
                    }),
                GameCommand::SetCursorMode(mode) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_cursor_mode(mode)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ToggleCursor(show) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .toggle_cursor(show)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetScoreboardOpen(open) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_scoreboard_open(open)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetDialogClientSide(client_side) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_dialog_client_side(client_side)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetDialogSelectedItem(selected) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_dialog_selected_item(selected)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetDialogEditboxText(text) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_dialog_editbox_text(&text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetGameState(state) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_game_state(state)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ConnectToServer { address, port } => {
                    let result =
                        self.r1_client
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|profile| {
                                profile
                                    .connect_to_server(&address, port)
                                    .map_err(|_| CommandError::NativeFailure)
                            });
                    if result.is_ok() {
                        self.invalidate_connection_state();
                    }
                    result
                }
                GameCommand::DisconnectWithReason(block_duration) => {
                    let rak_client = self.rak_client.load(Ordering::Acquire) as *mut c_void;
                    let result =
                        self.r1_client
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|profile| {
                                profile
                                    .disconnect_with_reason(rak_client, block_duration)
                                    .map_err(|_| CommandError::NativeFailure)
                            });
                    if result.is_ok() {
                        self.rak_client.store(0, Ordering::Release);
                        self.rpc_receiver.store(0, Ordering::Release);
                        self.player_address.store(0, Ordering::Release);
                        self.player_port.store(0, Ordering::Release);
                        self.invalidate_connection_state();
                    }
                    result
                }
                GameCommand::DeleteTextdraw(id) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .delete_textdraw(id)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::DeleteTextLabel(id) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .delete_text_label(id)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::CreateTextLabel {
                    id,
                    text,
                    colour,
                    position,
                    draw_distance,
                    behind_walls,
                    attached_player_id,
                    attached_vehicle_id,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .create_text_label(
                                id,
                                &text,
                                colour,
                                position,
                                draw_distance,
                                behind_walls,
                                attached_player_id,
                                attached_vehicle_id,
                            )
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawPosition { id, x, y } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_position(id, x, y)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawLetterStyle {
                    id,
                    width,
                    height,
                    colour,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_letter_style(id, width, height, colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawProportional { id, proportional } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_proportional(id, proportional)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawShadow { id, shadow, colour } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_shadow(id, shadow, colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawOutline {
                    id,
                    outline,
                    colour,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_outline(id, outline, colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawBox {
                    id,
                    enabled,
                    colour,
                    width,
                    height,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_box(id, enabled, colour, width, height)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawAlignment { id, alignment } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_alignment(id, alignment)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawString { id, text } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_string(id, &text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawModelStyle {
                    id,
                    rotation,
                    zoom,
                    colour1,
                    colour2,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_model_style(id, rotation, zoom, colour1, colour2)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SpawnLocalPlayer => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .spawn_local_player()
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetLocalPlayerSpecialAction(action) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_local_player_special_action(action)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetLocalPlayerName(name) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_local_player_name(&name)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ForceUnoccupiedSync { vehicle, seat } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .force_unoccupied_sync(vehicle, seat)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetPlayerColour { id, colour } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_player_colour(id, colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetSendRate { kind, milliseconds } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_send_rate(kind, milliseconds)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SendPacket {
                    id,
                    payload,
                    options,
                } => self
                    .send_packet_native(id, &payload, options)
                    .and_then(sent_game_command_result)
                    .map_err(|_| CommandError::NativeFailure),
                GameCommand::SendRpc {
                    id,
                    payload,
                    options,
                } => self
                    .send_rpc_native(id, &payload, options)
                    .and_then(sent_game_command_result)
                    .map_err(|_| CommandError::NativeFailure),
                GameCommand::EmulateIncomingPacket { id, payload } => self
                    .emulate_incoming_packet_native(id, payload)
                    .map(|_| ())
                    .map_err(|_| CommandError::NativeFailure),
                GameCommand::EmulateIncomingRpc { id, payload } => self
                    .emulate_incoming_rpc_native(id, payload)
                    .map(|_| ())
                    .map_err(|_| CommandError::NativeFailure),
            };
            match result {
                Ok(()) => self.game_commands.complete(queued.id, Ok(())),
                Err(error) => {
                    // Every command owns its plugin-provided payload. Keep logs
                    // free of dialog text, chat text, and death-window names.
                    log::debug!("game command failed: {error:?}");
                    self.game_commands
                        .complete(queued.id, Err(CommandError::NativeFailure));
                }
            }
        }
    }
}
