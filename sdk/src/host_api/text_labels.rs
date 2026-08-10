use crate::{CommandReceipt, HostApi, SampClientSdkCommandReceipt, SampClientSdkResult, Vector3};

impl HostApi {
    /// Queues a documented R1 3D text-label-pool deletion.
    pub fn submit_delete_text_label(
        self,
        id: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_delete_text_label)(id, &mut receipt) };
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
