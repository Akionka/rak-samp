//! Flat v1 host API table.

use super::*;

/// The host-side ABI table exported by `samp_client_sdk.asi`.
///
/// Fields are currently appended to preserve the v1 layout; during the ALPHA
/// stage the ABI may make an explicit compatibility break. Check `size` before
/// accessing fields added by a newer ABI version. Normal plugins use
/// Use [`crate::Samp`] and its subsystem facades instead of calling this table directly.
#[repr(C)]
pub struct SampClientSdkApiV1 {
    pub abi_version: u32,
    pub size: u32,
    pub host_status: extern "system" fn() -> SampClientSdkHostStatus,
    pub register_packet: unsafe extern "system" fn(
        SampClientSdkDirection,
        Option<SampClientSdkEventCallbackV1>,
        *mut c_void,
        *mut SampClientSdkSubscription,
    ) -> SampClientSdkResult,
    pub register_rpc: unsafe extern "system" fn(
        SampClientSdkDirection,
        Option<SampClientSdkEventCallbackV1>,
        *mut c_void,
        *mut SampClientSdkSubscription,
    ) -> SampClientSdkResult,
    pub unregister: unsafe extern "system" fn(SampClientSdkSubscription) -> SampClientSdkResult,
    pub event_id: unsafe extern "system" fn(*const SampClientSdkEventV1) -> u8,
    pub event_reset_read:
        unsafe extern "system" fn(*mut SampClientSdkEventV1) -> SampClientSdkResult,
    pub event_clear: unsafe extern "system" fn(*mut SampClientSdkEventV1) -> SampClientSdkResult,
    pub event_read_u8:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut u8) -> SampClientSdkResult,
    pub event_read_u16:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut u16) -> SampClientSdkResult,
    pub event_read_u32:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut u32) -> SampClientSdkResult,
    pub event_read_f32:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut f32) -> SampClientSdkResult,
    pub event_read_bytes:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut u8, usize) -> SampClientSdkResult,
    pub event_write_u8:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, u8) -> SampClientSdkResult,
    pub event_write_u16:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, u16) -> SampClientSdkResult,
    pub event_write_u32:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, u32) -> SampClientSdkResult,
    pub event_write_f32:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, f32) -> SampClientSdkResult,
    pub event_write_bytes: unsafe extern "system" fn(
        *mut SampClientSdkEventV1,
        *const u8,
        usize,
    ) -> SampClientSdkResult,
    pub send_packet: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        SampClientSdkSendOptions,
    ) -> SampClientSdkResult,
    pub send_rpc: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        SampClientSdkSendOptions,
    ) -> SampClientSdkResult,
    /// Atomically replaces a byte-aligned callback payload. This field was appended to ABI v1.
    pub event_replace_bytes: unsafe extern "system" fn(
        *mut SampClientSdkEventV1,
        *const u8,
        usize,
    ) -> SampClientSdkResult,
    /// Removes a listener and waits for callbacks already running on other threads.
    pub unregister_and_wait:
        unsafe extern "system" fn(SampClientSdkSubscription) -> SampClientSdkResult,
    /// Queues a locally generated incoming packet. `data` excludes the packet ID.
    pub emulate_incoming_packet:
        unsafe extern "system" fn(u8, *const u8, usize, usize) -> SampClientSdkResult,
    /// Dispatches a locally generated incoming RPC. `data` excludes the RPC ID.
    pub emulate_incoming_rpc:
        unsafe extern "system" fn(u8, *const u8, usize, usize) -> SampClientSdkResult,
    /// Returns unread bits in a callback-local event. This field was appended to ABI v1.
    pub event_remaining_bits: unsafe extern "system" fn(*mut SampClientSdkEventV1) -> usize,
    /// Reads exact bits into a left-aligned byte buffer. This field was appended to ABI v1.
    pub event_read_bits:
        unsafe extern "system" fn(*mut SampClientSdkEventV1, *mut u8, usize) -> SampClientSdkResult,
    /// Atomically replaces a callback payload with an exact bit length.
    pub event_replace_bits: unsafe extern "system" fn(
        *mut SampClientSdkEventV1,
        *const u8,
        usize,
        usize,
    ) -> SampClientSdkResult,
    /// Encodes one string with SA-MP's native RakNet compressor.
    pub encode_string: unsafe extern "system" fn(
        *const u8,
        usize,
        *mut u8,
        usize,
        *mut usize,
    ) -> SampClientSdkResult,
    /// Decodes one string from a callback event and advances its read cursor.
    pub event_read_encoded_string: unsafe extern "system" fn(
        *mut SampClientSdkEventV1,
        *mut u8,
        usize,
        *mut usize,
    ) -> SampClientSdkResult,
    /// Copies and queues a local R1 dialog request for the verified game-thread pump.
    pub show_local_dialog: unsafe extern "system" fn(
        u16,
        u32,
        *const u8,
        usize,
        *const u8,
        usize,
        *const u8,
        usize,
        *const u8,
        usize,
    ) -> SampClientSdkResult,
    /// Copies the latest host-owned local-player snapshot into `output`.
    pub local_player:
        unsafe extern "system" fn(*mut SampClientSdkLocalPlayerV1) -> SampClientSdkResult,
    /// Copies the latest R1 `CNetGame` state scalar into `output`.
    pub samp_game_state: unsafe extern "system" fn(*mut i32) -> SampClientSdkResult,
    /// Copies the detected SA-MP client version identity into `output`.
    pub samp_version: unsafe extern "system" fn(*mut u32) -> SampClientSdkResult,
    /// Decodes an owned bit stream with SA-MP's native RakNet string compressor.
    ///
    /// `input_read_offset` is the initial cursor, and `output_read_offset`
    /// receives the cursor after a successful decode. The output buffer has no
    /// required terminator; `output_len` selects its initialized prefix.
    pub decode_string: unsafe extern "system" fn(
        *const u8,
        usize,
        usize,
        usize,
        *mut u8,
        usize,
        *mut usize,
        *mut usize,
    ) -> SampClientSdkResult,
    /// Copies the latest host-owned current-server snapshot into `output`.
    pub server_info:
        unsafe extern "system" fn(*mut SampClientSdkServerInfoV1) -> SampClientSdkResult,
    /// Copies and queues a local R1 chat entry for the verified game-thread pump.
    pub show_local_chat_message: unsafe extern "system" fn(
        u32,
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
    ) -> SampClientSdkResult,
    /// Copies and queues a local R1 death-window entry for the game-thread pump.
    pub show_local_death_message: unsafe extern "system" fn(
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
        u8,
    ) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1/R3-1 chat display mode into `output`.
    pub local_chat_display_mode: unsafe extern "system" fn(*mut i32) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1/R3-1 cursor mode into `output`.
    pub local_cursor_mode: unsafe extern "system" fn(*mut i32) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1 scoreboard-open flag into `output`.
    pub local_scoreboard_open: unsafe extern "system" fn(*mut u8) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1 dialog-active flag into `output`.
    pub local_dialog_active: unsafe extern "system" fn(*mut u8) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached R1 chat-input-active flag into `output`.
    pub local_chat_input_active: unsafe extern "system" fn(*mut u8) -> SampClientSdkResult,
    /// Copies one entry from the cached R1 animation table into `output`.
    pub local_animation:
        unsafe extern "system" fn(u16, *mut SampClientSdkAnimationV1) -> SampClientSdkResult,
    /// Finds an R1 animation-table entry by copied name and file bytes.
    pub local_animation_id: unsafe extern "system" fn(
        *const u8,
        usize,
        *const u8,
        usize,
        *mut i32,
    ) -> SampClientSdkResult,
    /// Copies a cached local or demand-refreshed remote R1 player directory entry.
    pub player_info:
        unsafe extern "system" fn(u16, *mut SampClientSdkPlayerInfoV1) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached supported player-pool count into `output`.
    pub player_count: unsafe extern "system" fn(u8, *mut u16) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached supported player-pool largest ID into `output`.
    pub player_max_id: unsafe extern "system" fn(*mut u16) -> SampClientSdkResult,
    /// Copies a cached R1 vehicle-pool existence flag into `output`.
    pub vehicle_exists: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies the latest game-thread-cached active R1 dialog core into `output`.
    pub active_local_dialog:
        unsafe extern "system" fn(*mut SampClientSdkActiveDialogV1) -> SampClientSdkResult,
    /// Copies a cached R1 3D text-label-pool existence flag into `output`.
    pub text_label_exists: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies a cached R1 textdraw-pool existence flag into `output`.
    pub textdraw_exists: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies a cached R1 object-pool existence flag into `output`.
    pub object_exists: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies a cached R1 gangzone record into `output`.
    pub gangzone_info:
        unsafe extern "system" fn(u16, *mut SampClientSdkGangzoneV1) -> SampClientSdkResult,
    /// Copies a cached R1 3D text-label record into `output`.
    pub text_label_info:
        unsafe extern "system" fn(u16, *mut SampClientSdkTextLabelV1) -> SampClientSdkResult,
    /// Copies a cached R1 numeric textdraw record into `output`.
    pub textdraw_info:
        unsafe extern "system" fn(u16, *mut SampClientSdkTextDrawV1) -> SampClientSdkResult,
    /// Copies a cached R1 player-world-defined flag into `output`.
    pub player_defined: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies a cached R1 player-paused flag into `output`.
    pub player_paused: unsafe extern "system" fn(u16, *mut u8) -> SampClientSdkResult,
    /// Copies a cached R1 remote-player volatile state record into `output`.
    pub remote_player_state: unsafe extern "system" fn(
        u16,
        *mut SampClientSdkRemotePlayerStateV1,
    ) -> SampClientSdkResult,
    /// Copies and submits a local R1 dialog request, returning a completion receipt.
    pub submit_local_dialog: unsafe extern "system" fn(
        u16,
        u32,
        *const u8,
        usize,
        *const u8,
        usize,
        *const u8,
        usize,
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and submits a local R1 chat entry, returning a completion receipt.
    pub submit_local_chat_message: unsafe extern "system" fn(
        u32,
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and submits a local R1 death-window entry, returning a completion receipt.
    pub submit_local_death_message: unsafe extern "system" fn(
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
        u8,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Consumes an available completion, or returns `CommandPending` without consuming it.
    pub command_try_take: unsafe extern "system" fn(
        SampClientSdkCommandReceipt,
        *mut SampClientSdkCommandResultV1,
    ) -> SampClientSdkResult,
    /// Waits for and consumes a completion. A timeout leaves the receipt valid for retry.
    pub command_wait: unsafe extern "system" fn(
        SampClientSdkCommandReceipt,
        u32,
        *mut SampClientSdkCommandResultV1,
    ) -> SampClientSdkResult,
    /// Detaches a pending receipt without cancelling its owned command.
    pub command_release:
        unsafe extern "system" fn(SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies and queues a server-bound packet, returning its game-thread completion receipt.
    pub submit_packet: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        SampClientSdkSendOptions,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and queues a server-bound RPC, returning its game-thread completion receipt.
    pub submit_rpc: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        SampClientSdkSendOptions,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and queues a locally generated incoming packet, returning its completion receipt.
    pub submit_emulate_incoming_packet: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and queues a locally generated incoming RPC, returning its completion receipt.
    pub submit_emulate_incoming_rpc: unsafe extern "system" fn(
        u8,
        *const u8,
        usize,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies the host-captured RakClient address into `output` as an opaque pointer.
    pub raw_rakclient: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    /// Copies the latest game-thread-captured player-pool address into `output`.
    pub raw_player_pool: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    /// Copies the latest game-thread-captured vehicle-pool address into `output`.
    pub raw_vehicle_pool: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    /// Queues one validated R1 cursor-mode write and returns its completion receipt.
    pub submit_local_cursor_mode:
        unsafe extern "system" fn(i32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one R1 scoreboard-enabled write and returns its completion receipt.
    pub submit_local_scoreboard_open:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one R1 dialog client-side write and returns its completion receipt.
    pub submit_local_dialog_client_side:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one validated R1 CNetGame-state write and returns its completion receipt.
    pub submit_samp_game_state:
        unsafe extern "system" fn(i32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies the latest game-thread-captured local-player address into `output`.
    pub raw_local_player: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    /// Queues the R1 local-player spawn path and returns its completion receipt.
    pub submit_local_player_spawn:
        unsafe extern "system" fn(*mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one established R1 local-player special action and returns its completion receipt.
    pub submit_local_player_special_action:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one R1 replication send-rate write and returns its completion receipt.
    pub submit_send_rate:
        unsafe extern "system" fn(u8, u32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues the R1 cursor toggle transition and returns its completion receipt.
    pub submit_local_cursor_toggle:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one R1 chat display-mode write and returns its completion receipt.
    pub submit_local_chat_display_mode:
        unsafe extern "system" fn(i32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies the validated R1 RakPeer base into `output` as an opaque pointer.
    pub raw_rakpeer: unsafe extern "system" fn(*mut *mut c_void) -> SampClientSdkResult,
    /// Queues one R1 dialog close with the selected response button.
    pub submit_local_dialog_close:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies and queues a R1 chat-input text update.
    pub submit_local_chat_input_text: unsafe extern "system" fn(
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues the native R1 chat-input open or close transition.
    pub submit_local_chat_input_enabled:
        unsafe extern "system" fn(u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies text and queues R1 chat-input command processing.
    pub submit_local_chat_input_process: unsafe extern "system" fn(
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies the game-thread-cached R1 chat-input text into `output`.
    pub local_chat_input_text:
        unsafe extern "system" fn(*mut SampClientSdkChatInputTextV1) -> SampClientSdkResult,
    /// Queues a documented R1 local- or remote-player colour change.
    pub submit_player_colour: unsafe extern "system" fn(
        u16,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies and queues a documented R1 local-player nickname update.
    pub submit_local_player_name: unsafe extern "system" fn(
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues one verified unoccupied-vehicle synchronization send.
    pub submit_force_unoccupied_sync:
        unsafe extern "system" fn(u16, u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues the documented R1 aim synchronization send.
    pub submit_force_aim_sync:
        unsafe extern "system" fn(*mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues the documented R1 on-foot synchronization send.
    pub submit_force_onfoot_sync:
        unsafe extern "system" fn(*mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues the documented R1 stats synchronization send.
    pub submit_force_stats_sync:
        unsafe extern "system" fn(*mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies and queues the documented R1 reconnect sequence.
    pub submit_connect_to_server: unsafe extern "system" fn(
        *const u8,
        usize,
        u16,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues the documented R1 RakClient disconnect and restart sequence.
    pub submit_disconnect_with_reason:
        unsafe extern "system" fn(u32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues a documented R1 textdraw-pool deletion.
    pub submit_delete_textdraw:
        unsafe extern "system" fn(u16, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues a finite R1 textdraw screen-position update.
    pub submit_set_textdraw_position: unsafe extern "system" fn(
        u16,
        f32,
        f32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues finite R1 textdraw letter dimensions and a native colour value.
    pub submit_set_textdraw_letter_style: unsafe extern "system" fn(
        u16,
        f32,
        f32,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues an R1 textdraw proportional-flag update.
    pub submit_set_textdraw_proportional:
        unsafe extern "system" fn(u16, u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues an R1 textdraw shadow and background-colour update.
    pub submit_set_textdraw_shadow: unsafe extern "system" fn(
        u16,
        u8,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues an R1 textdraw outline and background-colour update.
    pub submit_set_textdraw_outline: unsafe extern "system" fn(
        u16,
        u8,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues a finite R1 textdraw box update.
    pub submit_set_textdraw_box: unsafe extern "system" fn(
        u16,
        u8,
        u32,
        f32,
        f32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues a validated R1 textdraw alignment update.
    pub submit_set_textdraw_alignment:
        unsafe extern "system" fn(u16, u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues a bounded R1 textdraw display-string update.
    pub submit_set_textdraw_string: unsafe extern "system" fn(
        u16,
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies the game-thread-cached R1 dialog list selection.
    pub local_dialog_selected_item: unsafe extern "system" fn(*mut i32) -> SampClientSdkResult,
    /// Queues an R1 dialog list-selection write.
    pub submit_local_dialog_selected_item:
        unsafe extern "system" fn(i32, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues a documented R1 3D text-label-pool deletion.
    pub submit_delete_text_label:
        unsafe extern "system" fn(u16, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies the game-thread-cached count of items in the active R1 dialog list.
    pub local_dialog_list_item_count: unsafe extern "system" fn(*mut i32) -> SampClientSdkResult,
    /// Queues a finite R1 textdraw model rotation, zoom, and vehicle-colour update.
    pub submit_set_textdraw_model_style: unsafe extern "system" fn(
        u16,
        f32,
        f32,
        f32,
        f32,
        u16,
        u16,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues one bounded R1 chat-history entry replacement.
    pub submit_local_chat_entry: unsafe extern "system" fn(
        u16,
        *const u8,
        usize,
        *const u8,
        usize,
        u32,
        u32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies one cached fixed R1 chat-history entry into `output`.
    pub chat_entry_info:
        unsafe extern "system" fn(u16, *mut SampClientSdkChatEntryV1) -> SampClientSdkResult,
    /// Queues a documented R1 3D text-label-pool creation at a caller-selected ID.
    pub submit_create_text_label: unsafe extern "system" fn(
        u16,
        *const u8,
        usize,
        u32,
        Vector3,
        f32,
        u8,
        u16,
        u16,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies one coherent game-thread-cached R1 dialog snapshot.
    pub local_dialog_snapshot:
        unsafe extern "system" fn(*mut SampClientSdkDialogSnapshotV1) -> SampClientSdkResult,
    /// Queues a bounded R1 dialog editbox text write.
    pub submit_local_dialog_editbox_text: unsafe extern "system" fn(
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies one cached R1 object GTAREF for an object-pool ID.
    pub local_object_handle: unsafe extern "system" fn(u16, *mut i32) -> SampClientSdkResult,
    /// Resolves one cached R1 object-pool ID from its GTAREF.
    pub local_object_id_by_handle: unsafe extern "system" fn(i32, *mut u16) -> SampClientSdkResult,
    /// Copies one cached R1 pickup GTAREF for a pickup-pool ID.
    pub local_pickup_handle: unsafe extern "system" fn(u16, *mut i32) -> SampClientSdkResult,
    /// Resolves one cached R1 pickup-pool ID from its GTAREF.
    pub local_pickup_id_by_handle: unsafe extern "system" fn(i32, *mut u16) -> SampClientSdkResult,
    /// Copies one cached R1 vehicle GTA handle for a vehicle-pool ID.
    pub local_vehicle_handle: unsafe extern "system" fn(u16, *mut i32) -> SampClientSdkResult,
    /// Resolves one cached R1 vehicle-pool ID from its GTA handle.
    pub local_vehicle_id_by_handle: unsafe extern "system" fn(i32, *mut u16) -> SampClientSdkResult,
    /// Copies one cached R1 player GTA ped handle for a player-pool ID.
    pub local_player_ped_handle: unsafe extern "system" fn(u16, *mut i32) -> SampClientSdkResult,
    /// Resolves one cached R1 player-pool ID from its GTA ped handle.
    pub local_player_id_by_ped_handle:
        unsafe extern "system" fn(i32, *mut u16) -> SampClientSdkResult,
    /// Queues one bounded native R1 chat-command registration.
    pub submit_register_chat_command: unsafe extern "system" fn(
        *const u8,
        usize,
        Option<SampClientSdkChatCommandCallbackV1>,
        *mut c_void,
        *mut SampClientSdkSubscription,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Reports whether an exact bounded name is present in the game-thread-cached R1 command table.
    pub local_chat_command_defined:
        unsafe extern "system" fn(*const u8, usize, *mut u8) -> SampClientSdkResult,
    /// Queues R1 3D text-label creation at the first native free pool slot.
    pub submit_create_text_label_auto: unsafe extern "system" fn(
        *const u8,
        usize,
        u32,
        Vector3,
        f32,
        u8,
        u16,
        u16,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Polls a text-label creation receipt and copies its typed completion.
    pub text_label_create_try_take: unsafe extern "system" fn(
        SampClientSdkCommandReceipt,
        *mut SampClientSdkTextLabelCreateResultV1,
    ) -> SampClientSdkResult,
    /// Waits for a text-label creation receipt and copies its typed completion.
    pub text_label_create_wait: unsafe extern "system" fn(
        SampClientSdkCommandReceipt,
        u32,
        *mut SampClientSdkTextLabelCreateResultV1,
    ) -> SampClientSdkResult,
    /// Queues replacement text for an existing R1 3D text label.
    pub submit_set_text_label_text: unsafe extern "system" fn(
        u16,
        *const u8,
        usize,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Copies a cached owned R1 on-foot synchronization record into `output`.
    pub onfoot_sync:
        unsafe extern "system" fn(u16, *mut SampClientSdkOnFootSyncV1) -> SampClientSdkResult,
    /// Copies a cached owned R1 in-car synchronization record into `output`.
    pub vehicle_sync:
        unsafe extern "system" fn(u16, *mut SampClientSdkInCarSyncV1) -> SampClientSdkResult,
    /// Copies a cached owned R1 passenger synchronization record into `output`.
    pub passenger_sync:
        unsafe extern "system" fn(u16, *mut SampClientSdkPassengerSyncV1) -> SampClientSdkResult,
    /// Copies a cached owned R1 trailer synchronization record into `output`.
    pub trailer_sync:
        unsafe extern "system" fn(u16, *mut SampClientSdkTrailerSyncV1) -> SampClientSdkResult,
    /// Copies a cached owned R1 aim synchronization record into `output`.
    pub aim_sync:
        unsafe extern "system" fn(u16, *mut SampClientSdkAimSyncV1) -> SampClientSdkResult,
    /// Queues one documented R1 trailer synchronization send.
    pub submit_force_trailer_sync:
        unsafe extern "system" fn(u16, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one documented R1 in-car synchronization send.
    pub submit_force_vehicle_sync:
        unsafe extern "system" fn(u16, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies and queues a documented R1 textdraw-pool creation in an explicit slot.
    pub submit_create_textdraw: unsafe extern "system" fn(
        u16,
        *const u8,
        usize,
        f32,
        f32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Queues a documented R1 textdraw font/style update.
    pub submit_set_textdraw_style: unsafe extern "system" fn(
        u16,
        i32,
        *mut SampClientSdkCommandReceipt,
    ) -> SampClientSdkResult,
    /// Takes the newest owned R1 client-side dialog-close response, if one is pending.
    pub take_local_dialog_response:
        unsafe extern "system" fn(*mut SampClientSdkDialogResponseV1) -> SampClientSdkResult,
    /// Queues one documented R1 passenger synchronization send.
    pub submit_force_passenger_sync:
        unsafe extern "system" fn(u16, u8, *mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Queues one documented R1 weapons synchronization send.
    pub submit_force_weapons_sync:
        unsafe extern "system" fn(*mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
    /// Copies a cached owned R1 streamed-out player marker position into `output`.
    pub streamed_out_player_position: unsafe extern "system" fn(
        u16,
        *mut SampClientSdkStreamedOutPlayerPositionV1,
    ) -> SampClientSdkResult,
    /// Reports whether `SAMPFUNCS.asi` is loaded in the process.
    pub sampfuncs_loaded: extern "system" fn() -> u8,
    /// Writes a bounded NUL-free byte string through SAMPFUNCS's console logger.
    pub sampfuncs_log_console: unsafe extern "system" fn(*const u8, usize) -> SampClientSdkResult,
    /// Reports whether incoming packet emulation has its host-captured native receiver.
    /// This is a copied readiness scalar, not a native address.
    pub incoming_emulation_ready: extern "system" fn() -> u8,
}

pub type SampClientSdkGetApiV1 = unsafe extern "system" fn(u32) -> *const SampClientSdkApiV1;
