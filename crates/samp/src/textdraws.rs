use crate::{CommandReceipt, Vector3};
use modkit_abi::{MOD_NATIVE_CALL_FAILED, ModResult, SAMP_MAX_TEXTDRAW_TEXT_BYTES, SampTextdrawV1};
use modkit_sdk::{Core, SampTextdrawService};

#[derive(Clone, Copy)]
pub struct Textdraws {
    core: Core,
    service: SampTextdrawService,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TextdrawId(u16);

impl TextdrawId {
    pub const fn new(raw: u16) -> Option<Self> {
        if raw < modkit_abi::SAMP_MAX_TEXTDRAWS {
            Some(Self(raw))
        } else {
            None
        }
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Textdraw {
    pub id: TextdrawId,
    pub text: Vec<u8>,
    pub letter_width: f32,
    pub letter_height: f32,
    pub letter_colour: u32,
    pub x: f32,
    pub y: f32,
    pub shadow: u8,
    pub outline: u8,
    pub background_colour: u32,
    pub style: i32,
    pub proportional: bool,
    pub align_left: bool,
    pub align_center: bool,
    pub align_right: bool,
    pub box_enabled: bool,
    pub box_width: f32,
    pub box_height: f32,
    pub box_colour: u32,
    pub model_id: u16,
    pub rotation: Vector3,
    pub zoom: f32,
    pub model_colour1: u16,
    pub model_colour2: u16,
}

impl Textdraws {
    pub(crate) const fn new(core: Core, service: SampTextdrawService) -> Self {
        Self { core, service }
    }

    pub fn exists(self, id: TextdrawId) -> Result<bool, ModResult> {
        self.service.exists(id.get())
    }

    pub fn get(self, id: TextdrawId) -> Result<Option<Textdraw>, ModResult> {
        textdraw(self.service.snapshot(id.get())?)
    }

    pub fn create(
        self,
        id: TextdrawId,
        text: &[u8],
        x: f32,
        y: f32,
    ) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_create(id.get(), text, x, y)?)
    }

    pub fn delete(self, id: TextdrawId) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_delete(id.get())?)
    }

    pub fn set_position(self, id: TextdrawId, x: f32, y: f32) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_set_position(id.get(), x, y)?)
    }

    pub fn set_style(self, id: TextdrawId, style: i32) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_set_style(id.get(), style)?)
    }

    pub fn set_letter_style(
        self,
        id: TextdrawId,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service
                .submit_set_letter_style(id.get(), width, height, colour)?,
        )
    }

    pub fn set_proportional(
        self,
        id: TextdrawId,
        proportional: bool,
    ) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service
                .submit_set_proportional(id.get(), proportional)?,
        )
    }

    pub fn set_shadow(
        self,
        id: TextdrawId,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service.submit_set_shadow(id.get(), shadow, colour)?,
        )
    }

    pub fn set_outline(
        self,
        id: TextdrawId,
        outline: u8,
        colour: u32,
    ) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service.submit_set_outline(id.get(), outline, colour)?,
        )
    }

    pub fn set_box(
        self,
        id: TextdrawId,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service
                .submit_set_box(id.get(), enabled, colour, width, height)?,
        )
    }

    pub fn set_alignment(self, id: TextdrawId, alignment: u8) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service.submit_set_alignment(id.get(), alignment)?,
        )
    }

    pub fn set_text(self, id: TextdrawId, text: &[u8]) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_set_text(id.get(), text)?)
    }

    pub fn set_model_style(
        self,
        id: TextdrawId,
        rotation: Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.service.submit_set_model_style(
                id.get(),
                [rotation.x, rotation.y, rotation.z],
                zoom,
                colour1,
                colour2,
            )?,
        )
    }
}

fn textdraw(raw: SampTextdrawV1) -> Result<Option<Textdraw>, ModResult> {
    if raw.exists == 0 {
        return Ok(None);
    }
    if [
        raw.proportional,
        raw.align_left,
        raw.align_center,
        raw.align_right,
        raw.box_enabled,
    ]
    .iter()
    .any(|value| *value > 1)
        || usize::from(raw.text_len) > SAMP_MAX_TEXTDRAW_TEXT_BYTES
    {
        return Err(MOD_NATIVE_CALL_FAILED);
    }
    Ok(Some(Textdraw {
        id: TextdrawId::new(raw.pool_index).ok_or(MOD_NATIVE_CALL_FAILED)?,
        text: raw.text[..usize::from(raw.text_len)].to_vec(),
        letter_width: raw.letter_width,
        letter_height: raw.letter_height,
        letter_colour: raw.letter_colour,
        x: raw.x,
        y: raw.y,
        shadow: raw.shadow,
        outline: raw.outline,
        background_colour: raw.background_colour,
        style: raw.style,
        proportional: raw.proportional != 0,
        align_left: raw.align_left != 0,
        align_center: raw.align_center != 0,
        align_right: raw.align_right != 0,
        box_enabled: raw.box_enabled != 0,
        box_width: raw.box_width,
        box_height: raw.box_height,
        box_colour: raw.box_colour,
        model_id: raw.model_id,
        rotation: Vector3 {
            x: raw.rotation.x,
            y: raw.rotation.y,
            z: raw.rotation.z,
        },
        zoom: raw.zoom,
        model_colour1: raw.model_colour1,
        model_colour2: raw.model_colour2,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_textdraw_ignores_unused_fields() {
        let raw = SampTextdrawV1 {
            proportional: u8::MAX,
            ..SampTextdrawV1::default()
        };
        assert_eq!(textdraw(raw), Ok(None));
    }

    #[test]
    fn present_textdraw_rejects_invalid_flags() {
        let raw = SampTextdrawV1 {
            exists: 1,
            proportional: 2,
            ..SampTextdrawV1::default()
        };
        assert_eq!(textdraw(raw), Err(MOD_NATIVE_CALL_FAILED));
    }
}
