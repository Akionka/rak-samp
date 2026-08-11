use crate::{
    CommandReceipt, HostApi, SampClientSdkCommandReceipt, SampClientSdkResult,
    TextLabelCreateReceipt, Vector3,
};

impl HostApi {
    /// Queues native R1 3D text-label creation at the first free pool ID.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn submit_create_text_label_auto(
        self,
        text: &[u8],
        colour: u32,
        position: Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: Option<u16>,
        attached_vehicle_id: Option<u16>,
    ) -> Result<TextLabelCreateReceipt, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_create_text_label_auto)(
                text.as_ptr(),
                text.len(),
                colour,
                position,
                draw_distance,
                u8::from(behind_walls),
                attached_player_id.unwrap_or(u16::MAX),
                attached_vehicle_id.unwrap_or(u16::MAX),
                &mut receipt,
            )
        };
        match result {
            SampClientSdkResult::Ok if receipt.id != 0 => {
                Ok(TextLabelCreateReceipt::new(self, receipt))
            }
            SampClientSdkResult::Ok => Err(SampClientSdkResult::NativeCallFailed),
            error => Err(error),
        }
    }

    /// Queues a documented R1 3D text-label-pool deletion.
    pub fn submit_delete_text_label(
        self,
        id: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_delete_text_label)(id, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues replacement text for an existing R1 3D text label.
    pub fn submit_set_text_label_text(
        self,
        id: u16,
        text: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_set_text_label_text)(id, text.as_ptr(), text.len(), &mut receipt)
        };
        self.command_receipt(result, receipt)
    }

    /// Queues R1 3D text-label creation at a selected pool ID.
    #[allow(clippy::too_many_arguments)]
    pub fn submit_create_text_label(
        self,
        id: u16,
        text: &[u8],
        colour: u32,
        position: Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: Option<u16>,
        attached_vehicle_id: Option<u16>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_create_text_label)(
                id,
                text.as_ptr(),
                text.len(),
                colour,
                position,
                draw_distance,
                u8::from(behind_walls),
                attached_player_id.unwrap_or(u16::MAX),
                attached_vehicle_id.unwrap_or(u16::MAX),
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }
}
