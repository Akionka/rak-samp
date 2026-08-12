use crate::{CommandReceipt, HostApi, SampClientSdkCommandReceipt, SampClientSdkResult, Vector3};

impl HostApi {
    /// Queues an R1 textdraw creation in one caller-selected free pool slot.
    pub fn submit_create_textdraw(
        self,
        id: u16,
        text: &[u8],
        x: f32,
        y: f32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_create_textdraw)(id, text.as_ptr(), text.len(), x, y, &mut receipt)
        };
        self.command_receipt(result, receipt)
    }

    /// Queues a documented R1 textdraw-pool deletion.
    pub fn submit_delete_textdraw(
        self,
        id: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_delete_textdraw)(id, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues a finite R1 textdraw screen-position update.
    pub fn submit_set_textdraw_position(
        self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_set_textdraw_position)(id, x, y, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues a documented R1 textdraw font/style update.
    pub fn submit_set_textdraw_style(
        self,
        id: u16,
        style: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe { (self.raw.submit_set_textdraw_style)(id, style, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues finite R1 textdraw letter dimensions and a native colour value.
    pub fn submit_set_textdraw_letter_style(
        self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_set_textdraw_letter_style)(id, width, height, colour, &mut receipt)
        };
        self.command_receipt(result, receipt)
    }

    /// Queues an R1 textdraw proportional-flag update.
    pub fn submit_set_textdraw_proportional(
        self,
        id: u16,
        proportional: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_set_textdraw_proportional)(id, u8::from(proportional), &mut receipt)
        };
        self.command_receipt(result, receipt)
    }

    /// Queues an R1 textdraw shadow and background-colour update.
    pub fn submit_set_textdraw_shadow(
        self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_set_textdraw_shadow)(id, shadow, colour, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues an R1 textdraw outline and background-colour update.
    pub fn submit_set_textdraw_outline(
        self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_set_textdraw_outline)(id, outline, colour, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues a finite R1 textdraw box update.
    pub fn submit_set_textdraw_box(
        self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_set_textdraw_box)(
                id,
                u8::from(enabled),
                colour,
                width,
                height,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }

    /// Queues a validated R1 textdraw alignment update.
    pub fn submit_set_textdraw_alignment(
        self,
        id: u16,
        alignment: u8,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result =
            unsafe { (self.raw.submit_set_textdraw_alignment)(id, alignment, &mut receipt) };
        self.command_receipt(result, receipt)
    }

    /// Queues a bounded R1 textdraw display-string update.
    pub fn submit_set_textdraw_string(
        self,
        id: u16,
        text: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_set_textdraw_string)(id, text.as_ptr(), text.len(), &mut receipt)
        };
        self.command_receipt(result, receipt)
    }

    /// Queues a finite R1 textdraw model rotation, zoom, and vehicle-colour update.
    pub fn submit_set_textdraw_model_style(
        self,
        id: u16,
        rotation: Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_set_textdraw_model_style)(
                id,
                rotation.x,
                rotation.y,
                rotation.z,
                zoom,
                colour1,
                colour2,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }
}
