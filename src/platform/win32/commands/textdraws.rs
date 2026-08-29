use super::*;

#[derive(Debug)]
pub(in crate::platform::win32) enum TextdrawCommand {
    CreateTextdraw {
        id: u16,
        text: Vec<u8>,
        x: f32,
        y: f32,
    },
    DeleteTextdraw(u16),
    SetTextdrawPosition {
        id: u16,
        x: f32,
        y: f32,
    },
    SetTextdrawStyle {
        id: u16,
        style: i32,
    },
    SetTextdrawLetterStyle {
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    },
    SetTextdrawProportional {
        id: u16,
        proportional: bool,
    },
    SetTextdrawShadow {
        id: u16,
        shadow: u8,
        colour: u32,
    },
    SetTextdrawOutline {
        id: u16,
        outline: u8,
        colour: u32,
    },
    SetTextdrawBox {
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    },
    SetTextdrawAlignment {
        id: u16,
        alignment: u8,
    },
    SetTextdrawString {
        id: u16,
        text: Vec<u8>,
    },
    SetTextdrawModelStyle {
        id: u16,
        rotation: crate::runtime::Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    },
}

impl BackendState {
    fn queue_textdraw_command(
        &self,
        command: TextdrawCommand,
    ) -> Result<CommandId, DirectClientError> {
        self.queue_game_command(GameCommand::Textdraw(command))
    }
    pub(in crate::platform::win32) fn submit_delete_textdraw(
        &self,
        id: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::DeleteTextdraw(id))
    }

    pub(in crate::platform::win32) fn submit_create_textdraw(
        &self,
        id: u16,
        text: Vec<u8>,
        x: f32,
        y: f32,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || text.len() > MAX_TEXTDRAW_CREATE_TEXT_BYTES
            || text.contains(&0)
            || !x.is_finite()
            || !y.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::CreateTextdraw { id, text, x, y })
    }

    pub(in crate::platform::win32) fn submit_set_textdraw_position(
        &self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !x.is_finite()
            || !y.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::SetTextdrawPosition { id, x, y })
    }

    pub(in crate::platform::win32) fn submit_set_textdraw_style(
        &self,
        id: u16,
        style: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !(0..=5).contains(&style)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::SetTextdrawStyle { id, style })
    }

    pub(in crate::platform::win32) fn submit_set_textdraw_letter_style(
        &self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !width.is_finite()
            || !height.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::SetTextdrawLetterStyle {
            id,
            width,
            height,
            colour,
        })
    }

    pub(in crate::platform::win32) fn submit_set_textdraw_proportional(
        &self,
        id: u16,
        proportional: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::SetTextdrawProportional { id, proportional })
    }

    pub(in crate::platform::win32) fn submit_set_textdraw_shadow(
        &self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::SetTextdrawShadow { id, shadow, colour })
    }

    pub(in crate::platform::win32) fn submit_set_textdraw_outline(
        &self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::SetTextdrawOutline {
            id,
            outline,
            colour,
        })
    }

    pub(in crate::platform::win32) fn submit_set_textdraw_box(
        &self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !width.is_finite()
            || !height.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::SetTextdrawBox {
            id,
            enabled,
            colour,
            width,
            height,
        })
    }

    pub(in crate::platform::win32) fn submit_set_textdraw_alignment(
        &self,
        id: u16,
        alignment: u8,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !(1..=3).contains(&alignment)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::SetTextdrawAlignment { id, alignment })
    }

    pub(in crate::platform::win32) fn submit_set_textdraw_string(
        &self,
        id: u16,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || text.len() >= MAX_TEXTDRAW_CREATE_TEXT_BYTES
            || text.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_textdraw_command(TextdrawCommand::SetTextdrawString { id, text })
    }

    pub(in crate::platform::win32) fn submit_set_textdraw_model_style(
        &self,
        id: u16,
        rotation: crate::runtime::Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.connection_profile().is_none() {
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
        self.queue_textdraw_command(TextdrawCommand::SetTextdrawModelStyle {
            id,
            rotation,
            zoom,
            colour1,
            colour2,
        })
    }

    pub(super) fn execute_textdraw_command(
        &self,
        command: TextdrawCommand,
    ) -> Result<(), CommandError> {
        match command {
            TextdrawCommand::DeleteTextdraw(id) => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .delete_textdraw(id)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.publish_deleted_textdraw(id);
                    Ok(())
                }),
            TextdrawCommand::CreateTextdraw { id, text, x, y } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .create_textdraw(id, &text, x, y)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.publish_created_textdraw(id);
                    Ok(())
                }),
            TextdrawCommand::SetTextdrawPosition { id, x, y } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_textdraw_position(id, x, y)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.invalidate_textdraw_snapshot(id);
                    Ok(())
                }),
            TextdrawCommand::SetTextdrawStyle { id, style } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_textdraw_style(id, style)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.invalidate_textdraw_snapshot(id);
                    Ok(())
                }),
            TextdrawCommand::SetTextdrawLetterStyle {
                id,
                width,
                height,
                colour,
            } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_textdraw_letter_style(id, width, height, colour)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.invalidate_textdraw_snapshot(id);
                    Ok(())
                }),
            TextdrawCommand::SetTextdrawProportional { id, proportional } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_textdraw_proportional(id, proportional)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.invalidate_textdraw_snapshot(id);
                    Ok(())
                }),
            TextdrawCommand::SetTextdrawShadow { id, shadow, colour } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_textdraw_shadow(id, shadow, colour)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.invalidate_textdraw_snapshot(id);
                    Ok(())
                }),
            TextdrawCommand::SetTextdrawOutline {
                id,
                outline,
                colour,
            } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_textdraw_outline(id, outline, colour)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.invalidate_textdraw_snapshot(id);
                    Ok(())
                }),
            TextdrawCommand::SetTextdrawBox {
                id,
                enabled,
                colour,
                width,
                height,
            } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_textdraw_box(id, enabled, colour, width, height)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.invalidate_textdraw_snapshot(id);
                    Ok(())
                }),
            TextdrawCommand::SetTextdrawAlignment { id, alignment } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_textdraw_alignment(id, alignment)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.invalidate_textdraw_snapshot(id);
                    Ok(())
                }),
            TextdrawCommand::SetTextdrawString { id, text } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_textdraw_string(id, &text)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.invalidate_textdraw_snapshot(id);
                    Ok(())
                }),
            TextdrawCommand::SetTextdrawModelStyle {
                id,
                rotation,
                zoom,
                colour1,
                colour2,
            } => self
                .connection_profile()
                .ok_or(CommandError::NativeFailure)
                .and_then(|profile| {
                    profile
                        .set_textdraw_model_style(id, rotation, zoom, colour1, colour2)
                        .map_err(|_| CommandError::NativeFailure)?;
                    self.invalidate_textdraw_snapshot(id);
                    Ok(())
                }),
        }
    }
}
