use super::{
    GangzoneId, ObjectHandle, ObjectId, PickupHandle, PlayerId, Samp, TextLabelId, TextdrawId,
    VehicleHandle, VehicleId,
};
use crate::{
    CommandReceipt, Gangzone, HostApi, SampClientSdkResult, TextDraw, TextLabel,
    TextLabelCreateReceipt,
};

#[derive(Clone, Copy)]
pub struct Textdraws {
    api: HostApi,
}

impl Textdraws {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn exists(self, id: TextdrawId) -> Result<bool, SampClientSdkResult> {
        self.api.is_textdraw_defined(id.get())
    }

    pub fn get(self, id: TextdrawId) -> Result<Option<TextDraw>, SampClientSdkResult> {
        self.api.textdraw(id.get())
    }

    /// Queues an R1 textdraw creation in one caller-selected free pool slot.
    /// Text is NUL-free and limited to 800 bytes; coordinates must be finite.
    pub fn create(
        self,
        id: TextdrawId,
        text: &[u8],
        x: f32,
        y: f32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_create_textdraw(id.get(), text, x, y)
    }

    /// Queues the documented R1 textdraw-pool deletion.
    pub fn delete(self, id: TextdrawId) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_delete_textdraw(id.get())
    }

    /// Queues a finite R1 textdraw screen-position update.
    pub fn set_position(
        self,
        id: TextdrawId,
        x: f32,
        y: f32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_set_textdraw_position(id.get(), x, y)
    }

    /// Queues an R1 textdraw font/style update. Valid styles are `0..=5`.
    pub fn set_style(
        self,
        id: TextdrawId,
        style: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_set_textdraw_style(id.get(), style)
    }

    /// Queues finite R1 textdraw letter dimensions and a native colour value.
    pub fn set_letter_style(
        self,
        id: TextdrawId,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_letter_style(id.get(), width, height, colour)
    }

    /// Queues an R1 textdraw proportional-flag update.
    pub fn set_proportional(
        self,
        id: TextdrawId,
        proportional: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_proportional(id.get(), proportional)
    }

    /// Queues an R1 textdraw shadow and background-colour update.
    pub fn set_shadow(
        self,
        id: TextdrawId,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_shadow(id.get(), shadow, colour)
    }

    /// Queues an R1 textdraw outline and background-colour update.
    pub fn set_outline(
        self,
        id: TextdrawId,
        outline: u8,
        colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_outline(id.get(), outline, colour)
    }

    /// Queues a finite R1 textdraw box update.
    pub fn set_box(
        self,
        id: TextdrawId,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_box(id.get(), enabled, colour, width, height)
    }

    /// Queues a validated R1 textdraw alignment update (1 left, 2 centre, 3 right).
    pub fn set_alignment(
        self,
        id: TextdrawId,
        alignment: u8,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_set_textdraw_alignment(id.get(), alignment)
    }

    /// Queues a bounded R1 textdraw display-string update.
    pub fn set_text(
        self,
        id: TextdrawId,
        text: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_set_textdraw_string(id.get(), text)
    }

    /// Queues a finite R1 textdraw model rotation, zoom, and vehicle-colour update.
    pub fn set_model_style(
        self,
        id: TextdrawId,
        rotation: crate::Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_model_style(id.get(), rotation, zoom, colour1, colour2)
    }
}

#[derive(Clone, Copy)]
pub struct Labels {
    api: HostApi,
}

impl Labels {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn exists(self, id: TextLabelId) -> Result<bool, SampClientSdkResult> {
        self.api.is_text_label_defined(id.get())
    }

    pub fn get(self, id: TextLabelId) -> Result<Option<TextLabel>, SampClientSdkResult> {
        self.api.text_label(id.get())
    }

    /// Queues deletion of this documented R1 3D text-label-pool entry.
    pub fn delete(self, id: TextLabelId) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_delete_text_label(id.get())
    }

    /// Queues replacement text for an existing R1 3D text label.
    ///
    /// The replacement must be non-empty, bounded, and contain no NUL byte.
    pub fn set_text(
        self,
        id: TextLabelId,
        text: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_set_text_label_text(id.get(), text)
    }

    /// Queues R1 3D text-label creation at the first native free pool ID.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        self,
        text: &[u8],
        colour: u32,
        position: crate::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: Option<PlayerId>,
        attached_vehicle_id: Option<VehicleId>,
    ) -> Result<TextLabelCreateReceipt, SampClientSdkResult> {
        self.api.submit_create_text_label_auto(
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id.map(PlayerId::get),
            attached_vehicle_id.map(VehicleId::get),
        )
    }

    /// Queues creation of one R1 3D text label at a caller-selected pool ID.
    #[allow(clippy::too_many_arguments)]
    pub fn create_at(
        self,
        id: TextLabelId,
        text: &[u8],
        colour: u32,
        position: crate::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: Option<PlayerId>,
        attached_vehicle_id: Option<VehicleId>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_create_text_label(
            id.get(),
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id.map(PlayerId::get),
            attached_vehicle_id.map(VehicleId::get),
        )
    }
}

#[derive(Clone, Copy)]
pub struct Objects {
    api: HostApi,
}

impl Objects {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn exists(self, id: ObjectId) -> Result<bool, SampClientSdkResult> {
        self.api.is_object_defined(id.get())
    }

    /// Returns the cached GTA SA object handle for a checked object ID.
    pub fn handle(self, id: ObjectId) -> Result<Option<ObjectHandle>, SampClientSdkResult> {
        self.api
            .object_handle(id.get())
            .map(|handle| handle.and_then(|handle| ObjectHandle::new(handle as u32)))
    }
}

impl ObjectHandle {
    /// Resolves this GTA SA object handle back to a checked object-pool ID.
    pub fn to_id(self, samp: Samp) -> Result<Option<ObjectId>, SampClientSdkResult> {
        samp.api()
            .object_id_by_handle(self.get() as i32)
            .map(|id| id.and_then(ObjectId::new))
    }
}

/// Placeholder for the pickup facade. No pickup read or mutation has crossed
/// the fixed R1 native boundary yet.
#[derive(Clone, Copy)]
pub struct Pickups {
    api: HostApi,
}

impl Pickups {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    /// Returns the cached GTA SA pickup handle for a raw pickup-pool index.
    pub fn handle(self, id: u16) -> Result<Option<PickupHandle>, SampClientSdkResult> {
        self.api
            .pickup_handle(id)
            .map(|handle| handle.and_then(|handle| PickupHandle::new(handle as u32)))
    }
}

impl PickupHandle {
    /// Resolves this GTA SA pickup handle back to a pickup-pool index.
    pub fn to_id(self, samp: Samp) -> Result<Option<u16>, SampClientSdkResult> {
        samp.api().pickup_id_by_handle(self.get() as i32)
    }
}

#[derive(Clone, Copy)]
pub struct Vehicles {
    api: HostApi,
}

impl Vehicles {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn exists(self, id: VehicleId) -> Result<bool, SampClientSdkResult> {
        self.api.is_vehicle_defined(id.get())
    }

    /// Returns the cached GTA SA vehicle handle for a checked vehicle ID.
    pub fn handle(self, id: VehicleId) -> Result<Option<VehicleHandle>, SampClientSdkResult> {
        self.api
            .vehicle_handle(id.get())
            .map(|handle| handle.and_then(|handle| VehicleHandle::new(handle as u32)))
    }
}

impl VehicleHandle {
    /// Resolves this GTA SA vehicle handle back to a checked vehicle-pool ID.
    pub fn to_id(self, samp: Samp) -> Result<Option<VehicleId>, SampClientSdkResult> {
        samp.api()
            .vehicle_id_by_handle(self.get() as i32)
            .map(|id| id.and_then(VehicleId::new))
    }
}

#[derive(Clone, Copy)]
pub struct Gangzones {
    api: HostApi,
}

impl Gangzones {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn get(self, id: GangzoneId) -> Result<Option<Gangzone>, SampClientSdkResult> {
        self.api.gangzone(id.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samp() -> Samp {
        Samp::from_api(crate::events::test_support::test_api())
    }

    #[test]
    fn textdraw_delete_returns_an_owned_completion_receipt() {
        let mut receipt = samp()
            .textdraws()
            .delete(TextdrawId::new(7).unwrap())
            .unwrap();
        assert_eq!(receipt.id(), 26);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn textdraw_create_returns_an_owned_completion_receipt() {
        let mut receipt = samp()
            .textdraws()
            .create(TextdrawId::new(7).unwrap(), b"fixture", 12.5, 34.0)
            .unwrap();
        assert_eq!(receipt.id(), 50);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn textdraw_position_returns_an_owned_completion_receipt() {
        let mut receipt = samp()
            .textdraws()
            .set_position(TextdrawId::new(7).unwrap(), 12.5, 34.0)
            .unwrap();
        assert_eq!(receipt.id(), 27);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn textdraw_style_returns_an_owned_completion_receipt() {
        let mut receipt = samp()
            .textdraws()
            .set_style(TextdrawId::new(7).unwrap(), 4)
            .unwrap();
        assert_eq!(receipt.id(), 51);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn textdraw_letter_style_returns_an_owned_completion_receipt() {
        let mut receipt = samp()
            .textdraws()
            .set_letter_style(TextdrawId::new(7).unwrap(), 1.25, 2.5, 0xFF11_2233)
            .unwrap();
        assert_eq!(receipt.id(), 28);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }
}
