use super::*;

impl BackendState {
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
        self.queue_game_command(GameCommand::DeleteTextdraw(id))
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
        self.queue_game_command(GameCommand::CreateTextdraw { id, text, x, y })
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
        self.queue_game_command(GameCommand::SetTextdrawPosition { id, x, y })
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
        self.queue_game_command(GameCommand::SetTextdrawStyle { id, style })
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
        self.queue_game_command(GameCommand::SetTextdrawLetterStyle {
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
        self.queue_game_command(GameCommand::SetTextdrawProportional { id, proportional })
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
        self.queue_game_command(GameCommand::SetTextdrawShadow { id, shadow, colour })
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
        self.queue_game_command(GameCommand::SetTextdrawOutline {
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
        self.queue_game_command(GameCommand::SetTextdrawBox {
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
        self.queue_game_command(GameCommand::SetTextdrawAlignment { id, alignment })
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
        self.queue_game_command(GameCommand::SetTextdrawString { id, text })
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
        self.queue_game_command(GameCommand::SetTextdrawModelStyle {
            id,
            rotation,
            zoom,
            colour1,
            colour2,
        })
    }
}
