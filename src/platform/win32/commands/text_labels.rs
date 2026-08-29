use super::*;

#[derive(Debug)]
pub(in crate::platform::win32) enum TextLabelCommand {
    DeleteTextLabel(u16),
    CreateTextLabel {
        id: u16,
        text: Vec<u8>,
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    },
    CreateTextLabelAuto {
        text: Vec<u8>,
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    },
    SetTextLabelText {
        id: u16,
        text: Vec<u8>,
    },
}

impl BackendState {
    fn queue_text_label_command(
        &self,
        command: TextLabelCommand,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::TextLabel(command))
    }
    pub(in crate::platform::win32) fn submit_delete_text_label(
        &self,
        id: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_text_label_command(TextLabelCommand::DeleteTextLabel(id))
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::platform::win32) fn submit_create_text_label(
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
        if self.scalar_profile().is_none()
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
        self.queue_text_label_command(TextLabelCommand::CreateTextLabel {
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

    #[allow(clippy::too_many_arguments)]
    pub(in crate::platform::win32) fn submit_create_text_label_auto(
        &self,
        text: Vec<u8>,
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none()
            || self.rak_client.load(Ordering::Acquire) == 0
            || text.len() > MAX_SAMP_TEXT_LABEL_TEXT_BYTES
            || text.contains(&0)
            || !position.x.is_finite()
            || !position.y.is_finite()
            || !position.z.is_finite()
            || !draw_distance.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let mut completions = self
            .auto_text_label_creates
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let id = self.queue_text_label_command(TextLabelCommand::CreateTextLabelAuto {
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id,
            attached_vehicle_id,
        })?;
        completions.insert(id, None);
        Ok(id)
    }

    pub(in crate::platform::win32) fn submit_set_text_label_text(
        &self,
        id: u16,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXT_LABELS
            || text.is_empty()
            || text.len() > MAX_SAMP_TEXT_LABEL_TEXT_BYTES
            || text.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_text_label_command(TextLabelCommand::SetTextLabelText { id, text })
    }

    pub(in crate::platform::win32) fn try_take_created_text_label(
        &self,
        id: CommandId,
    ) -> Result<Option<Result<u16, CommandError>>, CommandError> {
        match self.game_commands.try_take(id)? {
            Some(Ok(())) => Ok(Some(Ok(self.take_created_text_label_id(id)?))),
            Some(Err(error)) => {
                self.forget_created_text_label(id);
                Ok(Some(Err(error)))
            }
            None => Ok(None),
        }
    }

    pub(in crate::platform::win32) fn wait_for_created_text_label(
        &self,
        id: CommandId,
        timeout: std::time::Duration,
    ) -> Result<Result<u16, CommandError>, CommandError> {
        match self.game_commands.wait(
            id,
            timeout,
            !self.is_game_thread()
                && !self.registry.is_dispatching_on_current_thread()
                && !crate::host_api::chat_commands::is_dispatching_on_current_thread(),
        ) {
            Ok(Ok(())) => Ok(Ok(self.take_created_text_label_id(id)?)),
            Ok(Err(error)) => {
                self.forget_created_text_label(id);
                Ok(Err(error))
            }
            Err(error) => Err(error),
        }
    }

    fn take_created_text_label_id(&self, command: CommandId) -> Result<u16, CommandError> {
        self.auto_text_label_creates
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&command)
            .flatten()
            .ok_or(CommandError::NativeFailure)
    }

    pub(super) fn complete_created_text_label(&self, command: CommandId, label: u16) {
        if let Some(result) = self
            .auto_text_label_creates
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get_mut(&command)
        {
            *result = Some(label);
        }
    }

    pub(in crate::platform::win32) fn forget_created_text_label(&self, command: CommandId) {
        let _ = self
            .auto_text_label_creates
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&command);
    }

    pub(super) fn execute_text_label_command(
        &self,
        command_id: CommandId,
        command: TextLabelCommand,
    ) -> Result<(), CommandError> {
        match command {
            TextLabelCommand::DeleteTextLabel(id) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .delete_text_label(id)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.publish_deleted_text_label(id);
                    Ok(())
                }),
            TextLabelCommand::CreateTextLabel {
                id,
                text,
                colour,
                position,
                draw_distance,
                behind_walls,
                attached_player_id,
                attached_vehicle_id,
            } => self
                .connection_profile()
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
            TextLabelCommand::CreateTextLabelAuto {
                text,
                colour,
                position,
                draw_distance,
                behind_walls,
                attached_player_id,
                attached_vehicle_id,
            } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    let id = profile
                        .first_free_text_label_id()
                        .map_err(|_| CommandError::NativeFailure)?;
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
                        .map_err(|_| CommandError::NativeFailure)?;
                    let snapshot = profile
                        .text_label(id)
                        .map_err(|_| CommandError::NativeFailure)?
                        .ok_or(CommandError::NativeFailure)?;
                    self.publish_created_text_label(id, snapshot);
                    self.complete_created_text_label(command_id, id);
                    Ok(())
                }),
            TextLabelCommand::SetTextLabelText { id, text } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    let label = profile
                        .text_label(id)
                        .map_err(|_| CommandError::NativeFailure)?
                        .ok_or(CommandError::NativeFailure)?;
                    profile
                        .create_text_label(
                            id,
                            &text,
                            label.colour,
                            label.position,
                            label.draw_distance,
                            label.behind_walls,
                            label.attached_player_id.unwrap_or(u16::MAX),
                            label.attached_vehicle_id.unwrap_or(u16::MAX),
                        )
                        .map_err(|_| CommandError::NativeFailure)?;
                    let snapshot = profile
                        .text_label(id)
                        .map_err(|_| CommandError::NativeFailure)?
                        .ok_or(CommandError::NativeFailure)?;
                    self.publish_created_text_label(id, snapshot);
                    Ok(())
                }),
        }
    }
}
