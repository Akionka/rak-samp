//! The public, owned-value facade over the host ABI.

use crate::{
    ChatEntry, CommandReceipt, Gangzone, HostApi, LocalChatDisplayMode, LocalChatMessage,
    LocalCursorMode, LocalDeathMessage, LocalDialog, LocalDialogState, ResolveError,
    SampClientSdkClientVersion, SampClientSdkHostStatus, SampClientSdkResult, SampGameState,
    TextDraw, TextLabel,
    limits::{
        MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES, MAX_SAMP_GANGZONES, MAX_SAMP_OBJECTS, MAX_SAMP_PLAYERS,
        MAX_SAMP_TEXT_LABELS, MAX_SAMP_TEXTDRAWS, MAX_SAMP_VEHICLES,
    },
};
use std::time::Duration;

mod local_player;
mod network;
pub use local_player::{Anim, Animations, Local, Player, Players};
pub use network::{Net, Server};

macro_rules! bounded_id {
    ($name:ident, $maximum:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u16);

        impl $name {
            /// Returns `None` when `raw` is outside the R1 pool range.
            #[must_use]
            pub const fn new(raw: u16) -> Option<Self> {
                if raw < $maximum {
                    Some(Self(raw))
                } else {
                    None
                }
            }

            /// Returns the bounded raw SA-MP pool index.
            #[must_use]
            pub const fn get(self) -> u16 {
                self.0
            }
        }
    };
}

bounded_id!(
    PlayerId,
    MAX_SAMP_PLAYERS,
    "A checked SA-MP player-pool ID."
);
bounded_id!(
    VehicleId,
    MAX_SAMP_VEHICLES,
    "A checked SA-MP vehicle-pool ID."
);
bounded_id!(
    TextLabelId,
    MAX_SAMP_TEXT_LABELS,
    "A checked SA-MP 3D text-label ID."
);
bounded_id!(
    TextdrawId,
    MAX_SAMP_TEXTDRAWS,
    "A checked SA-MP textdraw-pool index."
);
bounded_id!(
    ObjectId,
    MAX_SAMP_OBJECTS,
    "A checked SA-MP object-pool ID."
);
bounded_id!(
    GangzoneId,
    MAX_SAMP_GANGZONES,
    "A checked SA-MP gangzone-pool ID."
);

macro_rules! gta_handle {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u32);

        impl $name {
            /// Returns `None` for the null GTA handle.
            #[must_use]
            pub const fn new(raw: u32) -> Option<Self> {
                if raw == 0 { None } else { Some(Self(raw)) }
            }

            /// Returns the raw non-null GTA handle.
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }
        }
    };
}

gta_handle!(
    ObjectHandle,
    "A typed non-null GTA SA object handle (GTAREF)."
);
gta_handle!(
    PickupHandle,
    "A typed non-null GTA SA pickup handle (GTAREF)."
);
gta_handle!(VehicleHandle, "A typed non-null GTA SA vehicle handle.");
gta_handle!(PedHandle, "A typed non-null GTA SA ped handle.");

/// Entry point for safe, copied SA-MP client operations.
#[derive(Clone, Copy)]
pub struct Samp {
    api: HostApi,
}

impl Samp {
    /// Connects to the default `samp_client_sdk.asi` host.
    pub fn connect(timeout: Duration) -> Result<Self, ResolveError> {
        crate::wait_for_default_host(timeout).map(|api| Self { api })
    }

    /// Connects to a named host module. `module_name` must be NUL-terminated.
    pub fn connect_to(module_name: &[u8], timeout: Duration) -> Result<Self, ResolveError> {
        crate::wait_for_host(module_name, timeout).map(|api| Self { api })
    }

    /// Returns the host lifecycle state without accessing client memory.
    #[must_use]
    pub fn status(self) -> SampClientSdkHostStatus {
        self.api.status()
    }

    /// Returns lifecycle and recognized-build predicates without reading
    /// client memory. This groups SF.lua's three historical probe helpers
    /// under one explicit host-status view.
    #[must_use]
    pub const fn probe(self) -> Probe {
        Probe { api: self.api }
    }

    /// Returns the recognized SA-MP client version identity.
    pub fn version(self) -> Result<SampClientSdkClientVersion, SampClientSdkResult> {
        self.api.samp_version()
    }

    /// Returns the cached native R1 game-state scalar.
    pub fn game_state(self) -> Result<i32, SampClientSdkResult> {
        self.api.samp_game_state()
    }

    /// Queues one validated R1 CNetGame-state write on the game thread.
    pub fn set_game_state(
        self,
        state: SampGameState,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_samp_game_state(state)
    }

    #[must_use]
    pub fn net(self) -> Net {
        Net::from_api(self.api)
    }

    #[must_use]
    pub fn server(self) -> Server {
        Server::from_api(self.api)
    }

    #[must_use]
    pub fn local(self) -> Local {
        Local::from_api(self.api)
    }

    #[must_use]
    pub fn players(self) -> Players {
        Players::from_api(self.api)
    }

    #[must_use]
    pub fn textdraws(self) -> Textdraws {
        Textdraws { api: self.api }
    }

    #[must_use]
    pub fn labels(self) -> Labels {
        Labels { api: self.api }
    }

    #[must_use]
    pub fn objects(self) -> Objects {
        Objects { api: self.api }
    }

    #[must_use]
    pub fn pickups(self) -> Pickups {
        Pickups { api: self.api }
    }

    #[must_use]
    pub fn vehicles(self) -> Vehicles {
        Vehicles { api: self.api }
    }

    #[must_use]
    pub fn gangzones(self) -> Gangzones {
        Gangzones { api: self.api }
    }

    #[must_use]
    pub fn dialogs(self) -> Dialogs {
        Dialogs { api: self.api }
    }

    #[must_use]
    pub fn chat(self) -> Chat {
        Chat { api: self.api }
    }

    #[must_use]
    pub fn chat_input(self) -> ChatInput {
        ChatInput { api: self.api }
    }

    #[must_use]
    pub fn cursor(self) -> Cursor {
        Cursor { api: self.api }
    }

    #[must_use]
    pub fn scoreboard(self) -> Scoreboard {
        Scoreboard { api: self.api }
    }

    #[must_use]
    pub fn anim(self) -> Anim {
        Anim::from_api(self.api)
    }

    pub(crate) const fn api(self) -> HostApi {
        self.api
    }

    #[cfg(test)]
    pub(crate) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }
}

/// Safe host and recognized-build probes.
#[derive(Clone, Copy)]
pub struct Probe {
    api: HostApi,
}

impl Probe {
    /// Returns whether the host has attached to a recognized `samp.dll`.
    #[must_use]
    pub fn is_samp_loaded(self) -> bool {
        self.api.is_samp_loaded()
    }

    /// Returns whether the SDK recognizes the loaded SA-MP build.
    #[must_use]
    pub fn is_sampfuncs_lua_loaded(self) -> bool {
        self.api.samp_version().is_ok()
    }

    /// Returns whether the recognized client and its RakClient hooks are ready.
    #[must_use]
    pub fn is_samp_available(self) -> bool {
        self.api.is_samp_available()
    }
}

#[derive(Clone, Copy)]
pub struct Textdraws {
    api: HostApi,
}

impl Textdraws {
    pub fn exists(self, id: TextdrawId) -> Result<bool, SampClientSdkResult> {
        self.api.is_textdraw_defined(id.get())
    }

    pub fn get(self, id: TextdrawId) -> Result<Option<TextDraw>, SampClientSdkResult> {
        self.api.textdraw(id.get())
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
    pub fn get(self, id: GangzoneId) -> Result<Option<Gangzone>, SampClientSdkResult> {
        self.api.gangzone(id.get())
    }
}

#[derive(Clone, Copy)]
pub struct Dialogs {
    api: HostApi,
}

impl Dialogs {
    pub fn active(self) -> Result<Option<LocalDialogState>, SampClientSdkResult> {
        self.api.active_local_dialog()
    }

    pub fn is_active(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_dialog_active()
    }

    /// Returns the copied selected index for an active R1 list dialog.
    pub fn selected_item(self) -> Result<i32, SampClientSdkResult> {
        self.api.local_dialog_selected_item()
    }

    /// Returns the copied count of items in the active R1 dialog list.
    pub fn list_item_count(self) -> Result<i32, SampClientSdkResult> {
        self.api.local_dialog_list_item_count()
    }

    /// Queues selection of an item in the active R1 list dialog.
    pub fn set_selected_item(
        self,
        selected: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_dialog_selected_item(selected)
    }

    pub fn show(self, dialog: LocalDialog<'_>) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_dialog(dialog)
    }

    /// Queues an R1 write that marks the current dialog as client-side or
    /// server-side on the game thread.
    pub fn set_client_side(
        self,
        client_side: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_dialog_client_side(client_side)
    }

    /// Queues closure of the active R1 dialog with its first (`0`) or second
    /// (`1`) response button.
    pub fn close_with_button(self, button: u8) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        if button > 1 {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        self.api.submit_local_dialog_close(button)
    }

    /// Queues a bounded R1 dialog editbox text replacement on the game thread.
    pub fn set_editbox_text(self, text: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        if text.len() > MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES || text.contains(&0) {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        self.api.submit_local_dialog_editbox_text(text)
    }
}

#[derive(Clone, Copy)]
pub struct Chat {
    api: HostApi,
}

impl Chat {
    pub fn display_mode(self) -> Result<LocalChatDisplayMode, SampClientSdkResult> {
        self.api.local_chat_display_mode()
    }

    /// Returns one copied fixed R1 chat-history entry.
    pub fn entry(self, id: u16) -> Result<ChatEntry, SampClientSdkResult> {
        self.api.chat_entry(id)
    }

    /// Queues one R1 chat display-mode write.
    pub fn set_display_mode(
        self,
        mode: LocalChatDisplayMode,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_display_mode(mode)
    }

    /// Queues one bounded R1 chat-history entry replacement.
    pub fn set_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_local_chat_entry(id, text, prefix, text_colour, prefix_colour)
    }

    pub fn is_visible(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_chat_visible()
    }

    #[allow(clippy::should_implement_trait)] // Mirrors the documented `Chat::add` SDK verb.
    pub fn add(
        self,
        message: LocalChatMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_message(message)
    }

    /// Alias for [`Self::add`] that emphasizes the request's explicit native style.
    #[allow(clippy::should_implement_trait)] // Mirrors the documented `Chat::add_with_style` SDK verb.
    pub fn add_with_style(
        self,
        message: LocalChatMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.add(message)
    }

    pub fn death_window(self) -> DeathWindow {
        DeathWindow { api: self.api }
    }
}

#[derive(Clone, Copy)]
pub struct DeathWindow {
    api: HostApi,
}

/// Safe cached state for SA-MP's local chat-input UI.
#[derive(Clone, Copy)]
pub struct ChatInput {
    api: HostApi,
}

impl ChatInput {
    pub fn is_active(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_chat_input_active()
    }

    /// Returns the owned game-thread-cached R1 chat-input text.
    pub fn text(self) -> Result<Vec<u8>, SampClientSdkResult> {
        self.api.local_chat_input_text()
    }

    /// Queues a copied R1 chat-input text update. Text is limited to 128 bytes
    /// and cannot contain an interior NUL.
    pub fn set_text(self, text: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_input_text(text)
    }

    /// Queues R1's native chat-input open or close transition.
    pub fn set_enabled(self, enabled: bool) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_input_enabled(enabled)
    }

    /// Queues a copied R1 chat-input text update followed by native command
    /// processing. Text is limited to 128 bytes and cannot contain an interior
    /// NUL.
    pub fn process(self, text: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_input_process(text)
    }
}

impl DeathWindow {
    #[allow(clippy::should_implement_trait)] // Mirrors the documented death-window `add` verb.
    pub fn add(
        self,
        message: LocalDeathMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_death_message(message)
    }
}

#[derive(Clone, Copy)]
pub struct Cursor {
    api: HostApi,
}

impl Cursor {
    pub fn mode(self) -> Result<LocalCursorMode, SampClientSdkResult> {
        self.api.local_cursor_mode()
    }

    pub fn is_active(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_cursor_active()
    }

    /// Queues one validated R1 cursor-mode change on the game thread.
    pub fn set_mode(
        self,
        mode: LocalCursorMode,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_cursor_mode(mode)
    }

    /// Queues SF.lua-compatible R1 cursor visibility behavior, including input
    /// re-enabling when hiding the cursor.
    pub fn toggle(self, show: bool) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_cursor_toggle(show)
    }
}

#[derive(Clone, Copy)]
pub struct Scoreboard {
    api: HostApi,
}

impl Scoreboard {
    pub fn is_open(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_scoreboard_open()
    }

    /// Queues one R1 scoreboard visibility change on the game thread.
    pub fn toggle(self, open: bool) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_scoreboard_open(open)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_ids_reject_pool_bounds() {
        assert_eq!(
            PlayerId::new(MAX_SAMP_PLAYERS - 1).map(PlayerId::get),
            Some(1003)
        );
        assert_eq!(PlayerId::new(MAX_SAMP_PLAYERS), None);
        assert_eq!(VehicleId::new(MAX_SAMP_VEHICLES), None);
        assert_eq!(TextLabelId::new(MAX_SAMP_TEXT_LABELS), None);
        assert_eq!(TextdrawId::new(MAX_SAMP_TEXTDRAWS), None);
        assert_eq!(ObjectId::new(MAX_SAMP_OBJECTS), None);
        assert_eq!(GangzoneId::new(MAX_SAMP_GANGZONES), None);
    }

    #[test]
    fn gta_handles_reject_the_null_value() {
        assert_eq!(ObjectHandle::new(0), None);
        assert_eq!(PickupHandle::new(0), None);
        assert_eq!(VehicleHandle::new(0), None);
        assert_eq!(PedHandle::new(0), None);
        assert_eq!(ObjectHandle::new(42).map(ObjectHandle::get), Some(42));
        assert_eq!(PickupHandle::new(42).map(PickupHandle::get), Some(42));
        assert_eq!(VehicleHandle::new(42).map(VehicleHandle::get), Some(42));
        assert_eq!(PedHandle::new(42).map(PedHandle::get), Some(42));
    }

    #[test]
    fn handle_lookups_route_through_the_mock_abi_and_round_trip() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let object_id = ObjectId::new(7).unwrap();
        let object_handle = samp.objects().handle(object_id).unwrap().unwrap();
        assert_eq!(object_handle.get(), 0x1007);
        assert_eq!(object_handle.to_id(samp).unwrap(), Some(object_id));

        let pickup_handle = samp.pickups().handle(7).unwrap().unwrap();
        assert_eq!(pickup_handle.get(), 0x2007);
        assert_eq!(pickup_handle.to_id(samp).unwrap(), Some(7));

        let vehicle_id = VehicleId::new(7).unwrap();
        let vehicle_handle = samp.vehicles().handle(vehicle_id).unwrap().unwrap();
        assert_eq!(vehicle_handle.get(), 0x3007);
        assert_eq!(vehicle_handle.to_id(samp).unwrap(), Some(vehicle_id));

        let player_id = PlayerId::new(7).unwrap();
        let ped_handle = samp
            .players()
            .player(player_id)
            .ped_handle()
            .unwrap()
            .unwrap();
        assert_eq!(ped_handle.get(), 0x4007);
        assert_eq!(ped_handle.to_id(samp).unwrap(), Some(player_id));
    }

    #[test]
    fn facade_reads_route_to_the_mock_abi() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        assert!(samp.probe().is_samp_loaded());
        assert!(samp.probe().is_sampfuncs_lua_loaded());
        assert!(samp.probe().is_samp_available());
        assert_eq!(samp.version(), Ok(SampClientSdkClientVersion::R1));
        assert_eq!(samp.game_state(), Ok(14));
        assert_eq!(samp.server().info().map(|info| info.port), Ok(7777));
        assert_eq!(samp.local().player().map(|player| player.id()), Ok(42));
        assert_eq!(samp.players().count(true), Ok(3));
        assert_eq!(
            samp.players().player(PlayerId::new(7).unwrap()).nickname(),
            Ok(Some(b"remote".to_vec()))
        );
        assert_eq!(
            samp.textdraws().exists(TextdrawId::new(7).unwrap()),
            Ok(true)
        );
        assert_eq!(
            samp.textdraws()
                .get(TextdrawId::new(7).unwrap())
                .map(|value| value.map(|value| (value.letter_style(), value.position()))),
            Ok(Some(((1.0, 2.0, 0xFF11_2233), (3.0, 4.0))))
        );
        assert_eq!(samp.labels().exists(TextLabelId::new(7).unwrap()), Ok(true));
        assert_eq!(
            samp.labels()
                .delete(TextLabelId::new(7).unwrap())
                .map(|receipt| receipt.id()),
            Ok(36)
        );
        assert_eq!(
            samp.labels()
                .create_at(
                    TextLabelId::new(7).unwrap(),
                    b"fixture",
                    0xFF11_2233,
                    crate::Vector3 {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0
                    },
                    25.0,
                    true,
                    Some(PlayerId::new(8).unwrap()),
                    None,
                )
                .map(|receipt| receipt.id()),
            Ok(39)
        );
        assert_eq!(samp.dialogs().list_item_count(), Ok(3));
        assert_eq!(
            samp.chat().entry(7).map(|entry| (entry.text, entry.prefix)),
            Ok((b"fixture".to_vec(), b"prefix".to_vec()))
        );
        assert_eq!(samp.objects().exists(ObjectId::new(7).unwrap()), Ok(true));
        assert_eq!(samp.vehicles().exists(VehicleId::new(7).unwrap()), Ok(true));
        assert_eq!(samp.chat_input().is_active(), Ok(false));
        assert_eq!(
            samp.dialogs().active().map(|dialog| dialog.map(|dialog| (
                dialog.id(),
                dialog.style(),
                dialog.caption().to_vec(),
                dialog.is_client_side(),
                dialog.text().to_vec(),
                dialog.editbox_text().map(<[u8]>::to_vec),
                dialog.items().to_vec()
            ))),
            Ok(Some((
                7,
                crate::LocalDialogStyle::Input,
                b"fixture".to_vec(),
                true,
                b"fixture".to_vec(),
                Some(b"fixture".to_vec()),
                vec![b"fixture".to_vec(); 3]
            )))
        );
        assert_eq!(
            samp.anim().get(0).map(|animation| animation.name),
            Ok(b"AIRPORT".to_vec())
        );
        assert_eq!(samp.anim().find(b"AIRPORT", b"THRW_BARL_THRW"), Ok(Some(0)));
        assert_eq!(samp.net().rpc_name(61), Some("ShowDialog"));
        assert_eq!(samp.net().packet_name(207), Some("PLAYER_SYNC"));
        assert_eq!(
            samp.net()
                .encode_string(b"ok")
                .map(|value| value.len_bits()),
            Ok(32)
        );
        let mut stream = crate::raknet::BitStream::from_bits([0b1010_0000], 3).unwrap();
        assert_eq!(
            samp.net().decode_string(&mut stream),
            Ok(b"fixture".to_vec())
        );
        let _ = samp.pickups();
    }

    #[test]
    fn network_commands_return_owned_completion_receipts() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut chat = samp.net().send_chat(b"fixture").unwrap();
        assert_eq!(chat.id(), 4);
        assert_eq!(chat.try_take(), Ok(Some(())));

        let mut packet = samp.net().send_packet(207, &[1, 2], 16).unwrap();
        assert_eq!(packet.id(), 4);
        assert_eq!(packet.try_take(), Ok(Some(())));

        let mut rpc = samp
            .net()
            .send_rpc_with_options(61, &[3], 8, crate::SampClientSdkSendOptions::default())
            .unwrap();
        assert_eq!(rpc.id(), 4);
        assert_eq!(rpc.wait(Duration::from_millis(0)), Ok(()));

        let mut emulated = samp.net().emulate_incoming_packet(207, &[4], 8).unwrap();
        assert_eq!(emulated.id(), 5);
        assert_eq!(emulated.try_take(), Ok(Some(())));

        let mut connect = samp.net().connect(b"127.0.0.1", 7777).unwrap();
        assert_eq!(connect.id(), 24);
        assert_eq!(connect.try_take(), Ok(Some(())));

        let mut disconnect = samp.net().disconnect(0).unwrap();
        assert_eq!(disconnect.id(), 25);
        assert_eq!(disconnect.try_take(), Ok(Some(())));
    }

    #[test]
    fn local_protocol_actions_delegate_to_the_receipt_bearing_network_path() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let vehicle = VehicleId::new(7).unwrap();
        for mut receipt in [
            samp.local().request_class(3).unwrap(),
            samp.local().send_interior_change(1).unwrap(),
            samp.local().send_spawn().unwrap(),
            samp.local().send_enter_vehicle(vehicle, false).unwrap(),
            samp.local().send_exit_vehicle(vehicle).unwrap(),
        ] {
            assert_eq!(receipt.id(), 4);
            assert_eq!(receipt.try_take(), Ok(Some(())));
        }
    }

    #[test]
    fn cursor_mode_change_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.cursor().set_mode(LocalCursorMode::LockCamera).unwrap();
        assert_eq!(receipt.id(), 6);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn cursor_toggle_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.cursor().toggle(false).unwrap();
        assert_eq!(receipt.id(), 14);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn chat_display_mode_change_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp
            .chat()
            .set_display_mode(LocalChatDisplayMode::NoShadow)
            .unwrap();
        assert_eq!(receipt.id(), 15);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn chat_input_mutations_return_owned_completion_receipts() {
        let samp = Samp::from_api(crate::events::test_support::test_api());

        let mut text = samp.chat_input().set_text(b"/sdk").unwrap();
        assert_eq!(text.id(), 17);
        assert_eq!(text.try_take(), Ok(Some(())));

        let mut enabled = samp.chat_input().set_enabled(true).unwrap();
        assert_eq!(enabled.id(), 18);
        assert_eq!(enabled.try_take(), Ok(Some(())));

        let mut processed = samp.chat_input().process(b"/sdk").unwrap();
        assert_eq!(processed.id(), 19);
        assert_eq!(processed.try_take(), Ok(Some(())));
    }

    #[test]
    fn chat_input_text_is_an_owned_cached_value() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        assert_eq!(samp.chat_input().text(), Ok(b"/sdk".to_vec()));
    }

    #[test]
    fn textdraw_delete_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp
            .textdraws()
            .delete(TextdrawId::new(7).unwrap())
            .unwrap();
        assert_eq!(receipt.id(), 26);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn textdraw_position_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp
            .textdraws()
            .set_position(TextdrawId::new(7).unwrap(), 12.5, 34.0)
            .unwrap();
        assert_eq!(receipt.id(), 27);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn textdraw_letter_style_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp
            .textdraws()
            .set_letter_style(TextdrawId::new(7).unwrap(), 1.25, 2.5, 0xFF11_2233)
            .unwrap();
        assert_eq!(receipt.id(), 28);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn scoreboard_toggle_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.scoreboard().toggle(true).unwrap();
        assert_eq!(receipt.id(), 7);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn dialog_client_side_change_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.dialogs().set_client_side(true).unwrap();
        assert_eq!(receipt.id(), 8);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn dialog_close_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.dialogs().close_with_button(1).unwrap();
        assert_eq!(receipt.id(), 16);
        assert_eq!(receipt.try_take(), Ok(Some(())));
        assert!(matches!(
            samp.dialogs().close_with_button(2),
            Err(SampClientSdkResult::InvalidArgument)
        ));
    }

    #[test]
    fn dialog_editbox_mutation_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.dialogs().set_editbox_text(b"fixture").unwrap();
        assert_eq!(receipt.id(), 40);
        assert_eq!(receipt.try_take(), Ok(Some(())));
        assert!(matches!(
            samp.dialogs().set_editbox_text(&[0]),
            Err(SampClientSdkResult::InvalidArgument)
        ));
    }

    #[test]
    fn game_state_change_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.set_game_state(SampGameState::Connected).unwrap();
        assert_eq!(receipt.id(), 10);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn local_player_mutations_and_send_rate_return_owned_completion_receipts() {
        let samp = Samp::from_api(crate::events::test_support::test_api());

        let mut spawn = samp.local().spawn().unwrap();
        assert_eq!(spawn.id(), 11);
        assert_eq!(spawn.try_take(), Ok(Some(())));

        let mut special_action = samp
            .local()
            .set_special_action(crate::SpecialAction::HandsUp)
            .unwrap();
        assert_eq!(special_action.id(), 12);
        assert_eq!(special_action.try_take(), Ok(Some(())));

        let mut send_rate = samp
            .net()
            .set_send_rate(crate::SendRateKind::Aim, 25)
            .unwrap();
        assert_eq!(send_rate.id(), 13);
        assert_eq!(send_rate.try_take(), Ok(Some(())));

        let mut colour = samp
            .players()
            .player(PlayerId::new(7).unwrap())
            .set_colour(0xFF00_00FF)
            .unwrap();
        assert_eq!(colour.id(), 21);
        assert_eq!(colour.try_take(), Ok(Some(())));

        let mut nickname = samp.local().set_nickname(b"fixture").unwrap();
        assert_eq!(nickname.id(), 22);
        assert_eq!(nickname.try_take(), Ok(Some(())));

        let mut unoccupied = samp
            .local()
            .force_unoccupied_sync(VehicleId::new(7).unwrap(), 1)
            .unwrap();
        assert_eq!(unoccupied.id(), 23);
        assert_eq!(unoccupied.try_take(), Ok(Some(())));
    }
}
