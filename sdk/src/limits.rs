/// Maximum decoded byte length accepted by [`crate::HostApi::decode_string`].
///
/// The extra byte used by the host's native decoder is reserved for its NUL
/// terminator and is not included in this limit.
pub const MAX_RAKNET_DECODED_STRING_BYTES: usize = 4_095;
/// Maximum byte length accepted by [`crate::Sampfuncs::log_console`].
pub const MAX_SAMPFUNCS_LOG_BYTES: usize = 4_095;
/// Number of addressable SA-MP player IDs in the R1 player pool.
pub const MAX_SAMP_PLAYERS: u16 = 1_004;
/// Number of addressable SA-MP vehicle IDs in the R1 vehicle pool.
pub const MAX_SAMP_VEHICLES: u16 = 2_000;
/// Number of addressable SA-MP 3D text-label IDs in the R1 label pool.
pub const MAX_SAMP_TEXT_LABELS: u16 = 2_048;
/// Number of raw global and local SA-MP textdraw-pool slots in R1.
pub const MAX_SAMP_TEXTDRAWS: u16 = 2_304;
/// Maximum non-NUL bytes in one R1 textdraw display string.
pub const MAX_SAMP_TEXTDRAW_STRING_BYTES: usize = 1_601;
/// Number of fixed R1 chat-history entries.
pub const MAX_SAMP_CHAT_ENTRIES: u16 = 100;
/// Maximum non-NUL bytes in one R1 chat-history text field.
pub const MAX_SAMP_CHAT_ENTRY_TEXT_BYTES: usize = 143;
/// Maximum non-NUL bytes in one R1 chat-history prefix field.
pub const MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES: usize = 27;
/// Number of addressable SA-MP object IDs in the R1 object pool.
pub const MAX_SAMP_OBJECTS: u16 = 1_000;
/// Number of addressable SA-MP pickup slots in the R1 pickup pool.
pub const MAX_SAMP_PICKUPS: u16 = 4_096;
/// Number of addressable SA-MP gangzone IDs in the R1 gangzone pool.
pub const MAX_SAMP_GANGZONES: u16 = 1_024;
/// Maximum copied byte length of an R1 3D text-label string.
///
/// R1 receives label text through its bounded `encodedString4096` path; the
/// native pool stores the resulting NUL-terminated allocation. The copied
/// result excludes that native terminator.
pub const MAX_SAMP_TEXT_LABEL_TEXT_BYTES: usize = 4_095;
/// Maximum non-NUL bytes in the R1 dialog body text.
pub const MAX_SAMP_DIALOG_TEXT_BYTES: usize = 4_096;
/// Maximum non-NUL bytes in the R1 dialog editbox text.
pub const MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES: usize = 128;
/// Maximum non-NUL bytes in an R1 local chat-command argument string.
pub const MAX_SAMP_CHAT_INPUT_TEXT_BYTES: usize = 128;
/// Maximum non-NUL bytes in an R1 local chat-command name.
pub const MAX_SAMP_CHAT_COMMAND_NAME_BYTES: usize = 32;
/// Maximum copied item strings retained for one active R1 dialog listbox.
pub const MAX_SAMP_DIALOG_LISTBOX_ITEMS: usize = 100;
/// Maximum non-NUL bytes in one R1 dialog listbox item string.
///
/// The native field has 256 bytes including its required NUL terminator.
pub const MAX_SAMP_DIALOG_LISTBOX_ITEM_BYTES: usize = 255;
