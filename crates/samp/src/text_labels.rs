use crate::{
    CommandReceipt, PlayerId, TextLabel, TextLabelCreateReceipt, TextLabelId, Vector3, VehicleId,
};
use modkit_abi::{ModResult, SampTextLabelCreateV1, SampVector3V1};
use modkit_sdk::{Core, SampTextLabelService};

#[derive(Clone, Copy)]
pub struct Labels {
    core: Core,
    service: SampTextLabelService,
}

impl Labels {
    pub(crate) const fn new(core: Core, service: SampTextLabelService) -> Self {
        Self { core, service }
    }

    pub fn exists(self, id: TextLabelId) -> Result<bool, ModResult> {
        match self.service.snapshot(id.get()) {
            Ok(_) => Ok(true),
            Err(error) if error == modkit_abi::MOD_NOT_FOUND => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub fn get(self, id: TextLabelId) -> Result<Option<TextLabel>, ModResult> {
        match self.service.snapshot(id.get()) {
            Ok(value) => TextLabel::from_abi(value).map(Some),
            Err(error) if error == modkit_abi::MOD_NOT_FOUND => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn delete(self, id: TextLabelId) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_delete(id.get())?)
    }

    pub fn set_text(self, id: TextLabelId, text: &[u8]) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.service.submit_set_text(id.get(), text)?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create(
        self,
        text: &[u8],
        colour: u32,
        position: Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: Option<PlayerId>,
        attached_vehicle_id: Option<VehicleId>,
    ) -> Result<TextLabelCreateReceipt, ModResult> {
        let request = create_request(
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id,
            attached_vehicle_id,
        )?;
        TextLabelCreateReceipt::new(self.core, self.service.submit_create(&request)?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_at(
        self,
        id: TextLabelId,
        text: &[u8],
        colour: u32,
        position: Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: Option<PlayerId>,
        attached_vehicle_id: Option<VehicleId>,
    ) -> Result<CommandReceipt, ModResult> {
        let request = create_request(
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id,
            attached_vehicle_id,
        )?;
        CommandReceipt::new(
            self.core,
            self.service.submit_create_at(id.get(), &request)?,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn create_request(
    text: &[u8],
    colour: u32,
    position: Vector3,
    draw_distance: f32,
    behind_walls: bool,
    attached_player_id: Option<PlayerId>,
    attached_vehicle_id: Option<VehicleId>,
) -> Result<SampTextLabelCreateV1, ModResult> {
    if text.len() > modkit_abi::SAMP_MAX_TEXT_LABEL_TEXT_BYTES {
        return Err(modkit_abi::MOD_INVALID_ARGUMENT);
    }
    let text_len = u32::try_from(text.len()).map_err(|_| modkit_abi::MOD_INVALID_ARGUMENT)?;
    Ok(SampTextLabelCreateV1 {
        text: text.as_ptr(),
        text_len,
        colour,
        position: SampVector3V1 {
            x: position.x,
            y: position.y,
            z: position.z,
        },
        draw_distance,
        attached_player_id: attached_player_id
            .map_or(modkit_abi::SAMP_TEXT_LABEL_NO_ATTACHMENT, PlayerId::get),
        attached_vehicle_id: attached_vehicle_id
            .map_or(modkit_abi::SAMP_TEXT_LABEL_NO_ATTACHMENT, VehicleId::get),
        behind_walls: u8::from(behind_walls),
        reserved: [0; 3],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_request_preserves_absent_attachments() {
        let request = create_request(
            b"label",
            0xFFFF_FFFF,
            Vector3::default(),
            25.0,
            true,
            None,
            None,
        )
        .unwrap();
        assert_eq!(request.text_len, 5);
        assert_eq!(request.attached_player_id, u16::MAX);
        assert_eq!(request.attached_vehicle_id, u16::MAX);
        assert_eq!(request.behind_walls, 1);
    }

    #[test]
    fn create_request_uses_checked_vehicle_id() {
        let vehicle = VehicleId::new(modkit_abi::SAMP_MAX_VEHICLES - 1).unwrap();
        let request = create_request(
            b"label",
            0,
            Vector3::default(),
            25.0,
            false,
            None,
            Some(vehicle),
        )
        .unwrap();
        assert_eq!(
            request.attached_vehicle_id,
            modkit_abi::SAMP_MAX_VEHICLES - 1
        );
        assert_eq!(VehicleId::new(modkit_abi::SAMP_MAX_VEHICLES), None);
    }

    #[test]
    fn create_request_rejects_oversized_text() {
        let text = vec![0; modkit_abi::SAMP_MAX_TEXT_LABEL_TEXT_BYTES + 1];
        assert_eq!(
            create_request(&text, 0, Vector3::default(), 1.0, false, None, None).unwrap_err(),
            modkit_abi::MOD_INVALID_ARGUMENT
        );
    }
}
