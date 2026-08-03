//! Private SA-MP 0.3.7 R1 client profile for direct local helpers.
//!
//! This deliberately does not share [`crate::AddressSet`]: RakNet hook offsets
//! are supported across several clients, while these object layouts and native
//! calls are safe only for the one fingerprinted R1 profile below.

use crate::runtime::{
    AnimationSnapshot, DirectClientError, GangzoneSnapshot, LocalChatMessageRequest,
    LocalDeathMessageRequest, LocalDialogRequest, LocalDialogSnapshot, LocalDialogStyle,
    LocalPlayerSnapshot, PlayerInfoSnapshot, ServerInfoSnapshot, TextLabelSnapshot,
    TextdrawSnapshot, Vector3,
};
use std::{ffi::c_void, mem};
use windows_sys::Win32::System::{
    LibraryLoader::GetModuleHandleA,
    Memory::{MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_GUARD, PAGE_NOACCESS, VirtualQuery},
};

const SAMP_R1_TIMESTAMP: u32 = 0x5542_F47A;
const SAMP_R1_ENTRY_POINT: u32 = 0x31DF13;

// GTA San Andreas 1.0 US (the standard 14,383,616-byte executable) has this
// in-memory PE identity. The compact executable and every later game build
// have a different SizeOfImage and are intentionally rejected.
const GTA_SA_10_US_IMAGE_BASE: u32 = 0x0040_0000;
const GTA_SA_10_US_IMAGE_SIZE: u32 = 0x0117_7000;
const GTA_SA_10_US_ENTRY_POINT: u32 = 0x0042_4570;

const DIALOG_SINGLETON_RVA: usize = 0x21A0B8;
const DIALOG_SHOW_RVA: usize = 0x6B9C0;
const INPUT_SINGLETON_RVA: usize = 0x21A0E8;
const INPUT_OPEN_RVA: usize = 0x657E0;
const INPUT_CLOSE_RVA: usize = 0x658E0;
const CHAT_SINGLETON_RVA: usize = 0x21A0E4;
const CHAT_ADD_ENTRY_RVA: usize = 0x64010;
const CHAT_GET_MODE_RVA: usize = 0x5D7A0;
const SCOREBOARD_SINGLETON_RVA: usize = 0x21A0B4;
const SCOREBOARD_CLOSE_RVA: usize = 0x6A320;
const SCOREBOARD_ENABLE_RVA: usize = 0x6AD30;
const DEATH_WINDOW_SINGLETON_RVA: usize = 0x21A0EC;
const DEATH_WINDOW_ADD_ENTRY_RVA: usize = 0x66930;
const DEATH_WINDOW_ADD_MESSAGE_RVA: usize = 0x66A10;
const NET_GAME_SINGLETON_RVA: usize = 0x21A0F8;
const NET_GAME_GET_STATE_RVA: usize = 0x2E20;
const NET_GAME_GET_PLAYER_POOL_RVA: usize = 0x1160;
const NET_GAME_GET_VEHICLE_POOL_RVA: usize = 0x1170;
const NET_GAME_RESET_LABEL_POOL_RVA: usize = 0x8F00;
const NET_GAME_RESET_TEXTDRAW_POOL_RVA: usize = 0x8C20;
const NET_GAME_RESET_OBJECT_POOL_RVA: usize = 0x8CC0;
const NET_GAME_RESET_GANGZONE_POOL_RVA: usize = 0x8D60;
const GANG_ZONE_POOL_CREATE_RVA: usize = 0x2170;
const TEXT_LABEL_POOL_CREATE_RVA: usize = 0x11C0;
const TEXTDRAW_CTOR_RVA: usize = 0xACF10;
const PLAYER_POOL_GET_LOCAL_PLAYER_RVA: usize = 0x1A30;
const PLAYER_POOL_GET_LOCAL_NAME_RVA: usize = 0x13CD0;
const PLAYER_POOL_GET_LOCAL_SCORE_RVA: usize = 0x6A1F0;
const PLAYER_POOL_GET_LOCAL_PING_RVA: usize = 0x6A200;
const PLAYER_POOL_IS_CONNECTED_RVA: usize = 0x10B0;
const PLAYER_POOL_GET_REMOTE_PLAYER_RVA: usize = 0x10F0;
const PLAYER_POOL_IS_NPC_RVA: usize = 0xB680;
const PLAYER_POOL_GET_NAME_RVA: usize = 0x13CE0;
const PLAYER_POOL_GET_SCORE_RVA: usize = 0x6A190;
const PLAYER_POOL_GET_PING_RVA: usize = 0x6A1C0;
const PLAYER_POOL_GET_COUNT_RVA: usize = 0x10520;
const PLAYER_POOL_UPDATE_LARGEST_ID_RVA: usize = 0x102B0;
const VEHICLE_POOL_DOES_EXIST_RVA: usize = 0x1140;
const REMOTE_PLAYER_GET_COLOUR_ARGB_RVA: usize = 0x12A00;
const LOCAL_PLAYER_GET_PED_RVA: usize = 0x2D60;
const LOCAL_PLAYER_GET_COLOUR_ARGB_RVA: usize = 0x3D90;
const PED_GET_HEALTH_RVA: usize = 0xA6610;
const PED_GET_ARMOUR_RVA: usize = 0xA6650;
const GAME_SINGLETON_RVA: usize = 0x21A10C;
const GAME_PROCESS_INPUT_ENABLING_RVA: usize = 0x9BC10;
const ANIMATION_TABLE_RVA: usize = 0xF15B0;
const ANIMATION_TABLE_ENTRY_COUNT: usize = 1812;
const ANIMATION_TABLE_ENTRY_SIZE: usize = 36;
const MAX_SAMP_PLAYERS: u16 = 1004;
const MAX_SAMP_VEHICLES: u16 = 2000;
const MAX_SAMP_TEXT_LABELS: u16 = 2048;
const MAX_SAMP_TEXTDRAWS: u16 = 2304;
const MAX_SAMP_OBJECTS: u16 = 1000;
const MAX_SAMP_GANGZONES: u16 = 1024;

const PLAYER_POOL_LOCAL_ID_OFFSET: usize = 0x04;
const PLAYER_POOL_LARGEST_ID_OFFSET: usize = 0x00;
const VEHICLE_POOL_NOT_EMPTY_OFFSET: usize = 0x3074;
const NET_GAME_POOLS_OFFSET: usize = 0x3CD;
const NET_GAME_POOLS_LABEL_POOL_OFFSET: usize = 0x0C;
const NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET: usize = 0x10;
const NET_GAME_POOLS_OBJECT_POOL_OFFSET: usize = 0x04;
const NET_GAME_POOLS_GANGZONE_POOL_OFFSET: usize = 0x08;
const LABEL_POOL_NOT_EMPTY_OFFSET: usize = 0xE800;
const LABEL_TEXT_OFFSET: usize = 0x00;
const LABEL_COLOUR_OFFSET: usize = 0x04;
const LABEL_POSITION_OFFSET: usize = 0x08;
const LABEL_DRAW_DISTANCE_OFFSET: usize = 0x14;
const LABEL_BEHIND_WALLS_OFFSET: usize = 0x18;
const LABEL_ATTACHED_PLAYER_OFFSET: usize = 0x19;
const LABEL_ATTACHED_VEHICLE_OFFSET: usize = 0x1B;
const LABEL_SIZE: usize = 0x1D;
const MAX_TEXT_LABEL_TEXT_BYTES: usize = 4_095;
const TEXTDRAW_POOL_NOT_EMPTY_OFFSET: usize = 0;
const TEXTDRAW_POOL_OBJECTS_OFFSET: usize = 0x2400;
const OBJECT_POOL_NOT_EMPTY_OFFSET: usize = 0x04;
const GANGZONE_POOL_NOT_EMPTY_OFFSET: usize = 0x1000;
const GANGZONE_LEFT_OFFSET: usize = 0x00;
const GANGZONE_BOTTOM_OFFSET: usize = 0x04;
const GANGZONE_RIGHT_OFFSET: usize = 0x08;
const GANGZONE_TOP_OFFSET: usize = 0x0C;
const GANGZONE_COLOUR_OFFSET: usize = 0x10;
const GANGZONE_ALTERNATE_COLOUR_OFFSET: usize = 0x14;
// These packed CNetGame fields are cross-checked by the independently written
// fixture. `GetGameState`'s signed R1 target reads offset 0x3BD from this same
// layout, which anchors the packed field sequence.
const NET_GAME_HOST_ADDRESS_OFFSET: usize = 0x20;
const NET_GAME_HOSTNAME_OFFSET: usize = 0x121;
const NET_GAME_PORT_OFFSET: usize = 0x225;
const NET_GAME_HOST_STRING_CAPACITY: usize = 257;
const SCOREBOARD_ENABLED_OFFSET: usize = 0x00;
const GAME_CURSOR_MODE_OFFSET: usize = 0x55;
const DIALOG_ACTIVE_OFFSET: usize = 0x28;
const DIALOG_TYPE_OFFSET: usize = 0x2C;
const DIALOG_ID_OFFSET: usize = 0x30;
const DIALOG_CAPTION_OFFSET: usize = 0x40;
const DIALOG_CAPTION_CAPACITY: usize = 65;
const DIALOG_SERVER_SIDE_OFFSET: usize = 0x81;
const INPUT_ENABLED_OFFSET: usize = 0x14E0;
const TEXTDRAW_DATA_OFFSET: usize = 0x963;
const TEXTDRAW_LETTER_WIDTH_OFFSET: usize = TEXTDRAW_DATA_OFFSET;
const TEXTDRAW_LETTER_HEIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x04;
const TEXTDRAW_LETTER_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x08;
const TEXTDRAW_ALIGN_CENTER_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0D;
const TEXTDRAW_BOX_ENABLED_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0E;
const TEXTDRAW_BOX_WIDTH_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0F;
const TEXTDRAW_BOX_HEIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x13;
const TEXTDRAW_BOX_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x17;
const TEXTDRAW_PROPORTIONAL_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x1B;
const TEXTDRAW_BACKGROUND_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x1C;
const TEXTDRAW_SHADOW_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x20;
const TEXTDRAW_OUTLINE_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x21;
const TEXTDRAW_ALIGN_LEFT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x22;
const TEXTDRAW_ALIGN_RIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x23;
const TEXTDRAW_STYLE_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x24;
const TEXTDRAW_X_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x28;
const TEXTDRAW_Y_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x2C;
const TEXTDRAW_MODEL_ID_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x45;
const TEXTDRAW_ROTATION_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x47;
const TEXTDRAW_ZOOM_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x53;
const TEXTDRAW_MODEL_COLOUR1_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x57;
const TEXTDRAW_MODEL_COLOUR2_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x59;

// First 16 bytes of SA-MP 0.3.7 R1's `CDialog::Show` at `DIALOG_SHOW_RVA`.
// The function uses a frame-less prologue; do not substitute the common
// `55 8B EC` prologue here, or the valid R1 profile will be rejected.
const DIALOG_SHOW_SIGNATURE: [u8; 16] = [
    0x83, 0xEC, 0x10, 0x53, 0x56, 0x57, 0x8B, 0x7C, 0x24, 0x20, 0x33, 0xDB, 0x3B, 0xFB, 0x8B, 0xF1,
];

// The `CDialog::Show` active-state comparison immediately follows the original
// show-target signature. Verify it separately to pin the copied `m_bIsActive`
// read without widening the existing show-call signature.
const DIALOG_SHOW_ACTIVE_SIGNATURE: [u8; 22] = [
    0x83, 0xEC, 0x10, 0x53, 0x56, 0x57, 0x8B, 0x7C, 0x24, 0x20, 0x33, 0xDB, 0x3B, 0xFB, 0x8B, 0xF1,
    0x7D, 0x17, 0x39, 0x5E, 0x28, 0x0F,
];

// The same fingerprinted R1 `CDialog::Show` target writes the copied core
// snapshot fields after the active-state branch: ID at 0x30, type at 0x2C,
// server-side flag at 0x81, and then addresses the fixed caption buffer at
// 0x40. Dynamic text/control storage is intentionally not read here.
const DIALOG_SHOW_CORE_FIELDS_SIGNATURE: [u8; 15] = [
    0x89, 0x7E, 0x30, 0x89, 0x46, 0x2C, 0x89, 0x8E, 0x81, 0x00, 0x00, 0x00, 0x8D, 0x56, 0x40,
];

// `CInput::Open` and `Close` both read the packed `m_bEnabled` flag at
// offset 0x14E0 before proceeding with their UI work. They are evidence only:
// the safe helper below copies the flag and never invokes either mutation.
const INPUT_OPEN_SIGNATURE: [u8; 16] = [
    0x83, 0xEC, 0x10, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xE0, 0x14, 0x00, 0x00, 0x85, 0xC0, 0x0F, 0x85,
];
const INPUT_CLOSE_SIGNATURE: [u8; 16] = [
    0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xE0, 0x14, 0x00, 0x00, 0x85, 0xC0, 0x74, 0x39, 0x8B, 0x4E, 0x08,
];

// First 16 bytes of SA-MP 0.3.7 R1's `CChat::AddEntry` at
// `CHAT_ADD_ENTRY_RVA`. The target's x86 prologue moves `this` from ECX into
// EBP, slides the 100-entry ring, and then consumes the five stack arguments.
const CHAT_ADD_ENTRY_SIGNATURE: [u8; 16] = [
    0x55, 0x56, 0x8B, 0xE9, 0x57, 0x8D, 0xBD, 0x32, 0x01, 0x00, 0x00, 0x8D, 0xB5, 0x2E, 0x02, 0x00,
];

// `CChat::GetMode` is a leaf R1 accessor: `mov eax, [ecx + 8]; ret`. Keep
// the exact code signature rather than reading the field directly so the
// private layout remains behind the fingerprinted native profile.
const CHAT_GET_MODE_SIGNATURE: [u8; 4] = [0x8B, 0x41, 0x08, 0xC3];

// `CScoreboard::Close` and `Enable` both start by comparing the packed
// `m_bIsEnabled` field at offset zero. Together they anchor the copied boolean
// read below without turning the field into a public client layout.
const SCOREBOARD_CLOSE_SIGNATURE: [u8; 16] = [
    0x56, 0x8B, 0xF1, 0x83, 0x3E, 0x00, 0x74, 0x3C, 0x8B, 0x46, 0x34, 0x85, 0xC0, 0x74, 0x35, 0xC6,
];
const SCOREBOARD_ENABLE_SIGNATURE: [u8; 16] = [
    0x56, 0x8B, 0xF1, 0x83, 0x3E, 0x00, 0x75, 0x43, 0x8B, 0x46, 0x34, 0x85, 0xC0, 0x74, 0x3C, 0xC6,
];

// `CGame::ProcessInputEnabling` loads `m_nCursorMode` from offset 0x55 before
// checking the associated input-enable state. Its exact R1 signature anchors
// the narrow copied cursor-mode field below.
const GAME_PROCESS_INPUT_ENABLING_SIGNATURE: [u8; 16] = [
    0x56, 0x8B, 0xF1, 0x8B, 0x46, 0x55, 0x57, 0x33, 0xFF, 0x3B, 0xC7, 0x0F, 0x85, 0x07, 0x01, 0x00,
];

// R1 stores 1,812 fixed 36-byte `group:name` animation entries in a static
// table. Its complete first entry fingerprints the data format before the
// game-thread pump makes an owned copy.
const ANIMATION_TABLE_SIGNATURE: [u8; 36] = [
    0x41, 0x49, 0x52, 0x50, 0x4F, 0x52, 0x54, 0x3A, 0x54, 0x48, 0x52, 0x57, 0x5F, 0x42, 0x41, 0x52,
    0x4C, 0x5F, 0x54, 0x48, 0x52, 0x57, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

// `CDeathWindow::AddMessage` is an R1 thunk to `AddEntry`; verify both its
// five-byte relative jump and the start of the final target before enabling
// the direct death-window helper.
const DEATH_WINDOW_ADD_MESSAGE_SIGNATURE: [u8; 5] = [0xE9, 0x1B, 0xFF, 0xFF, 0xFF];
const DEATH_WINDOW_ADD_ENTRY_SIGNATURE: [u8; 16] = [
    0x8B, 0xD1, 0xE8, 0x49, 0xF6, 0xFF, 0xFF, 0x8A, 0x44, 0x24, 0x14, 0x8B, 0x4C, 0x24, 0x10, 0x88,
];

// `CNetGame::GetGameState` returns the client's native state enum by value.
// Keep this signature separate from the dialog target: callers expose the
// value only as an opaque scalar, rather than depending on enum names from an
// unversioned client header.
const NET_GAME_GET_STATE_SIGNATURE: [u8; 7] = [0x8B, 0x81, 0xBD, 0x03, 0x00, 0x00, 0xC3];

// Accessor-only R1 player-directory targets. The safe API calls these only on
// the verified game-thread pump, immediately copies their scalar/string
// outputs, and never exports a pool or remote-player pointer.
const NET_GAME_GET_PLAYER_POOL_SIGNATURE: [u8; 9] =
    [0x8B, 0x81, 0xCD, 0x03, 0x00, 0x00, 0x8B, 0x40, 0x18];
const NET_GAME_GET_VEHICLE_POOL_SIGNATURE: [u8; 10] =
    [0x8B, 0x81, 0xCD, 0x03, 0x00, 0x00, 0x8B, 0x40, 0x1C, 0xC3];
// R1's `CNetGame::ResetLabelPool` reads `m_pPools` at `0x3CD` then the label
// pool pointer at `0x0C`. Start after the compiler-generated SEH setup so the
// signature does not depend on dynamic exception-chain storage.
const NET_GAME_RESET_LABEL_POOL_FIELDS_SIGNATURE: [u8; 18] = [
    0x51, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xCD, 0x03, 0x00, 0x00, 0x57, 0x8B, 0x78, 0x0C, 0x85, 0xFF,
    0x74, 0x10,
];
// R1's `CNetGame::ResetTextDrawPool` reads `m_pPools` at `0x3CD` then the
// textdraw pool pointer at `0x10`. The pool starts with 2,304 packed BOOL
// existence flags: 2,048 global slots followed by 256 local slots.
const NET_GAME_RESET_TEXTDRAW_POOL_FIELDS_SIGNATURE: [u8; 18] = [
    0x51, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xCD, 0x03, 0x00, 0x00, 0x57, 0x8B, 0x78, 0x10, 0x85, 0xFF,
    0x74, 0x10,
];
// R1's `CNetGame::ResetObjectPool` reads `m_pPools` at `0x3CD` then the
// object-pool pointer at `0x04`. The 1,000 packed BOOL existence flags follow
// the pool's signed largest-ID field at offset `0x04`.
const NET_GAME_RESET_OBJECT_POOL_FIELDS_SIGNATURE: [u8; 18] = [
    0x51, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xCD, 0x03, 0x00, 0x00, 0x57, 0x8B, 0x78, 0x04, 0x85, 0xFF,
    0x74, 0x10,
];
// R1's `CNetGame::ResetGangZonePool` reads `m_pPools` at `0x3CD` then the
// gangzone-pool pointer at `0x08` before destroying/replacing the pool.
const NET_GAME_RESET_GANGZONE_POOL_FIELDS_SIGNATURE: [u8; 18] = [
    0x51, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xCD, 0x03, 0x00, 0x00, 0x57, 0x8B, 0x78, 0x08, 0x85, 0xFF,
    0x74, 0x10,
];
// R1's `CGangZonePool::Create` clears the indexed object pointer and its
// matching 1,024-entry BOOL flag at pool offset `0x1000` before allocating a
// 24-byte rectangle-and-colour record. The subsequent stores pin all six
// scalar fields copied by the safe snapshot below.
const GANG_ZONE_POOL_CREATE_POOL_FIELDS_SIGNATURE: [u8; 18] = [
    0xC7, 0x04, 0xBE, 0x00, 0x00, 0x00, 0x00, 0xC7, 0x84, 0xBE, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00,
];
const GANG_ZONE_POOL_CREATE_RECORD_FIELDS_SIGNATURE: [u8; 37] = [
    0x8B, 0x4C, 0x24, 0x10, 0x8B, 0x54, 0x24, 0x1C, 0x89, 0x08, 0x8B, 0x4C, 0x24, 0x18, 0x89, 0x50,
    0x04, 0x8B, 0x54, 0x24, 0x14, 0x89, 0x48, 0x08, 0x8B, 0x4C, 0x24, 0x20, 0x89, 0x50, 0x0C, 0x89,
    0x48, 0x10, 0x89, 0x48, 0x14,
];
// `CLabelPool::Create` computes the exact source-string length plus its NUL
// terminator, allocates that many bytes, stores the allocation at label offset
// zero, then copies the complete terminated string. This anchors the bounded
// copied read below; it does not make the allocation or native pointer public.
const TEXT_LABEL_POOL_CREATE_TEXT_ALLOCATION_SIGNATURE: [u8; 18] = [
    0x8D, 0x48, 0x01, 0x8B, 0xFF, 0x8A, 0x10, 0x40, 0x84, 0xD2, 0x75, 0xF9, 0x2B, 0xC1, 0x40, 0x50,
    0x6A, 0x01,
];
const TEXT_LABEL_POOL_CREATE_TEXT_COPY_SIGNATURE: [u8; 25] = [
    0x8B, 0xD3, 0x83, 0xC4, 0x08, 0x6B, 0xD2, 0x1D, 0x8D, 0x34, 0x2A, 0x89, 0x06, 0x8B, 0xCF, 0x8A,
    0x11, 0x41, 0x88, 0x10, 0x40, 0x84, 0xD2, 0x75, 0xF6,
];
// The tail of that same R1 target writes the fixed label scalars at offsets
// 0x04 through 0x1C. Together with the independent packed fixture, this pins
// every non-pointer field exported by the copied snapshot.
const TEXT_LABEL_POOL_CREATE_SCALAR_FIELDS_SIGNATURE: [u8; 48] = [
    0x89, 0x46, 0x04, 0x8B, 0x44, 0x24, 0x24, 0x89, 0x4E, 0x08, 0x8B, 0x4C, 0x24, 0x28, 0x89, 0x56,
    0x0C, 0x8A, 0x54, 0x24, 0x2C, 0x89, 0x46, 0x10, 0x66, 0x8B, 0x44, 0x24, 0x30, 0x89, 0x4E, 0x14,
    0x66, 0x8B, 0x4C, 0x24, 0x34, 0x88, 0x56, 0x18, 0x66, 0x89, 0x46, 0x19, 0x66, 0x89, 0x4E, 0x1B,
];
// R1's `CTextDraw` constructor sets `m_data` at offset `0x963`, then copies
// all scalar fields from the packed transmit record. The two signatures cover
// the letter/box/alignment/position stores and the model/rotation/zoom/colour
// stores separately so the safe snapshot below never relies on a public C++
// header as its layout authority.
const TEXTDRAW_CTOR_CORE_FIELDS_SIGNATURE: [u8; 155] = [
    0x8B, 0x44, 0x24, 0x10, 0x8B, 0x48, 0x01, 0x89, 0x0A, 0x8B, 0x50, 0x05, 0x89, 0x96, 0x67, 0x09,
    0x00, 0x00, 0x8B, 0x48, 0x09, 0x89, 0x8E, 0x6B, 0x09, 0x00, 0x00, 0x33, 0xDB, 0x88, 0x9E, 0x6F,
    0x09, 0x00, 0x00, 0x8A, 0x10, 0xC0, 0xEA, 0x03, 0x80, 0xE2, 0x01, 0x88, 0x96, 0x70, 0x09, 0x00,
    0x00, 0x8A, 0x08, 0x80, 0xE1, 0x01, 0x88, 0x8E, 0x71, 0x09, 0x00, 0x00, 0x8B, 0x50, 0x0D, 0x89,
    0x96, 0x72, 0x09, 0x00, 0x00, 0x8B, 0x48, 0x11, 0x89, 0x8E, 0x76, 0x09, 0x00, 0x00, 0x8B, 0x50,
    0x15, 0x89, 0x96, 0x7A, 0x09, 0x00, 0x00, 0x8A, 0x08, 0xC0, 0xE9, 0x04, 0x80, 0xE1, 0x01, 0x88,
    0x8E, 0x7E, 0x09, 0x00, 0x00, 0x8B, 0x50, 0x1B, 0x89, 0x96, 0x7F, 0x09, 0x00, 0x00, 0x8A, 0x48,
    0x19, 0x88, 0x8E, 0x83, 0x09, 0x00, 0x00, 0x8A, 0x50, 0x1A, 0x88, 0x96, 0x84, 0x09, 0x00, 0x00,
    0x8A, 0x08, 0xD0, 0xE9, 0x80, 0xE1, 0x01, 0x88, 0x8E, 0x85, 0x09, 0x00, 0x00, 0x8A, 0x10, 0xC0,
    0xEA, 0x02, 0x80, 0xE2, 0x01, 0x88, 0x96, 0x86, 0x09, 0x00, 0x00,
];
const TEXTDRAW_CTOR_MODEL_FIELDS_SIGNATURE: [u8; 126] = [
    0x0F, 0xB6, 0x48, 0x1F, 0x89, 0x8E, 0x87, 0x09, 0x00, 0x00, 0x8B, 0x50, 0x21, 0x89, 0x96, 0x8B,
    0x09, 0x00, 0x00, 0x8B, 0x48, 0x25, 0x89, 0x8E, 0x8F, 0x09, 0x00, 0x00, 0x83, 0xCF, 0xFF, 0x89,
    0xBE, 0x9B, 0x09, 0x00, 0x00, 0x89, 0xBE, 0x9F, 0x09, 0x00, 0x00, 0x8A, 0x50, 0x20, 0x88, 0x96,
    0xA7, 0x09, 0x00, 0x00, 0x66, 0x8B, 0x48, 0x29, 0x66, 0x89, 0x8E, 0xA8, 0x09, 0x00, 0x00, 0x8B,
    0x50, 0x2B, 0x89, 0x96, 0xAA, 0x09, 0x00, 0x00, 0x8B, 0x48, 0x2F, 0x89, 0x8E, 0xAE, 0x09, 0x00,
    0x00, 0x8B, 0x50, 0x33, 0x89, 0x96, 0xB2, 0x09, 0x00, 0x00, 0x8B, 0x48, 0x37, 0x89, 0x8E, 0xB6,
    0x09, 0x00, 0x00, 0x66, 0x8B, 0x50, 0x3B, 0x8B, 0x4C, 0x24, 0x14, 0x66, 0x89, 0x96, 0xBA, 0x09,
    0x00, 0x00, 0x66, 0x8B, 0x40, 0x3D, 0x51, 0x66, 0x89, 0x86, 0xBC, 0x09, 0x00, 0x00,
];
const PLAYER_POOL_IS_CONNECTED_SIGNATURE: [u8; 16] = [
    0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xEC, 0x03, 0x72, 0x05, 0x33, 0xC0, 0xC2, 0x04, 0x00,
];
const PLAYER_POOL_GET_REMOTE_PLAYER_SIGNATURE: [u8; 16] = [
    0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xEC, 0x03, 0x77, 0x10, 0x0F, 0xB7, 0xC0, 0x8B, 0x44,
];
const PLAYER_POOL_IS_NPC_SIGNATURE: [u8; 16] = [
    0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xEC, 0x03, 0x77, 0x0B, 0x0F, 0xB7, 0xC0, 0x8B, 0x44,
];
const PLAYER_POOL_GET_NAME_SIGNATURE: [u8; 16] = [
    0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3B, 0x41, 0x04, 0x75, 0x12, 0x83, 0x79, 0x1E, 0x10, 0x72,
];
const PLAYER_POOL_GET_SCORE_SIGNATURE: [u8; 16] = [
    0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xEC, 0x03, 0x77, 0x0B, 0x0F, 0xB7, 0xC0, 0x8B, 0x44,
];
const PLAYER_POOL_GET_PING_SIGNATURE: [u8; 16] = [
    0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xEC, 0x03, 0x77, 0x0B, 0x0F, 0xB7, 0xC0, 0x8B, 0x44,
];
const PLAYER_POOL_GET_COUNT_SIGNATURE: [u8; 16] = [
    0x8B, 0x54, 0x24, 0x04, 0x56, 0x33, 0xC0, 0x85, 0xD2, 0x57, 0x74, 0x71, 0x33, 0xD2, 0x8B, 0xFF,
];
const PLAYER_POOL_UPDATE_LARGEST_ID_SIGNATURE: [u8; 16] = [
    0x56, 0x57, 0x33, 0xF6, 0xB8, 0x02, 0x00, 0x00, 0x00, 0x8D, 0x91, 0xE2, 0x0F, 0x00, 0x00, 0x90,
];
// `CVehiclePool::DoesExist` first rejects IDs >= 2,000, then returns the
// packed `m_bNotEmpty[id]` BOOL at offset 0x3074. The independent fixture
// below anchors the array offset; this exact target signature prevents the
// profile from accepting a same-shaped pointer accessor from another build.
const VEHICLE_POOL_DOES_EXIST_SIGNATURE: [u8; 29] = [
    0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xD0, 0x07, 0x72, 0x05, 0x33, 0xC0, 0xC2, 0x04, 0x00,
    0x0F, 0xB7, 0xC0, 0x8B, 0x84, 0x81, 0x74, 0x30, 0x00, 0x00, 0xC2, 0x04, 0x00,
];
const REMOTE_PLAYER_GET_COLOUR_ARGB_SIGNATURE: [u8; 16] = [
    0x0F, 0xB7, 0x81, 0xAB, 0x00, 0x00, 0x00, 0x50, 0xE8, 0x63, 0xAB, 0x09, 0x00, 0xC1, 0xE8, 0x08,
];

const LOCAL_PLAYER_ACTIVE_OFFSET: usize = 0x0C;
const LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET: usize = 0x14;
const LOCAL_PLAYER_ONFOOT_OFFSET: usize = 0x18;
const LOCAL_PLAYER_INCAR_OFFSET: usize = 0xAA;
const LOCAL_PLAYER_ONFOOT_POSITION_OFFSET: usize = 0x06;
const LOCAL_PLAYER_ONFOOT_SPEED_OFFSET: usize = 0x26;
const LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET: usize = 0x25;
const LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET: usize = 0x40;
const LOCAL_PLAYER_INCAR_POSITION_OFFSET: usize = 0x18;
const LOCAL_PLAYER_INCAR_SPEED_OFFSET: usize = 0x24;
// `CPed` inherits a 0x48-byte `CEntity`, then owns its accessory arrays before
// its GTA-ped pointer.
const SAMP_PED_GAME_PED_OFFSET: usize = 0x2A4;
const INVALID_ID: u16 = u16::MAX;

/// A narrow R1-only profile whose fields and call targets never cross the
/// plugin ABI. `verify` has to succeed before any profile address is used.
#[derive(Clone, Copy, Debug)]
pub(super) struct R1ClientProfile {
    module_base: usize,
}

impl R1ClientProfile {
    pub(super) fn verify(module_base: usize, entry_point: u32) -> Option<Self> {
        (entry_point == SAMP_R1_ENTRY_POINT
            && unsafe { samp_r1_pe_matches(module_base) }
            && unsafe { gta_sa_10_us_matches() }
            && unsafe { r1_targets_match(module_base) })
        .then_some(Self { module_base })
    }

    pub(super) fn show_dialog(self, request: LocalDialogRequest) -> Result<(), DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;

        let title = nul_terminated(request.title);
        let text = nul_terminated(request.text);
        let button1 = nul_terminated(request.button1);
        let button2 = nul_terminated(request.button2);
        let show: DialogShowFn = unsafe { mem::transmute(self.module_base + DIALOG_SHOW_RVA) };
        unsafe {
            show(
                dialog,
                i32::from(request.id),
                request.style.as_raw() as i32,
                title.as_ptr().cast(),
                text.as_ptr().cast(),
                button1.as_ptr().cast(),
                button2.as_ptr().cast(),
                0,
            );
        }
        Ok(())
    }

    pub(super) fn show_chat_message(
        self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let text = nul_terminated(request.text);
        let prefix = nul_terminated(request.prefix);
        let add_entry: ChatAddEntryFn =
            unsafe { mem::transmute(self.module_base + CHAT_ADD_ENTRY_RVA) };
        unsafe {
            add_entry(
                chat,
                request.style.as_raw(),
                text.as_ptr().cast(),
                prefix.as_ptr().cast(),
                request.text_colour,
                request.prefix_colour,
            );
        }
        Ok(())
    }

    pub(super) fn show_death_message(
        self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        let death_window = self.death_window().ok_or(DirectClientError::NotReady)?;
        let killer = nul_terminated(request.killer);
        let victim = nul_terminated(request.victim);
        let add_message: DeathWindowAddMessageFn =
            unsafe { mem::transmute(self.module_base + DEATH_WINDOW_ADD_MESSAGE_RVA) };
        unsafe {
            add_message(
                death_window,
                killer.as_ptr().cast(),
                victim.as_ptr().cast(),
                request.killer_colour,
                request.victim_colour,
                request.weapon,
            );
        }
        Ok(())
    }

    pub(super) fn dialog_is_ready(self) -> bool {
        self.dialog().is_some()
    }

    pub(super) fn chat_is_ready(self) -> bool {
        self.chat().is_some()
    }

    pub(super) fn chat_display_mode(self) -> Result<i32, DirectClientError> {
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let get_mode: ChatGetModeFn =
            unsafe { mem::transmute(self.module_base + CHAT_GET_MODE_RVA) };
        let mode = unsafe { get_mode(chat) };
        matches!(mode, 0..=2)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn cursor_mode(self) -> Result<i32, DirectClientError> {
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let mode = unsafe { read_unaligned::<i32>(game as usize + GAME_CURSOR_MODE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        matches!(mode, 0..=4)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn scoreboard_is_open(self) -> Result<bool, DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        match unsafe { read_unaligned::<i32>(scoreboard as usize + SCOREBOARD_ENABLED_OFFSET) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    pub(super) fn dialog_is_active(self) -> Result<bool, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        read_r1_bool(dialog as usize + DIALOG_ACTIVE_OFFSET)
    }

    /// Copies bounded core metadata from an active R1 dialog on the game
    /// thread. It deliberately excludes the dynamically allocated text and
    /// DXUT control contents.
    pub(super) fn dialog_state(self) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            dialog.cast(),
            DIALOG_SERVER_SIDE_OFFSET + mem::size_of::<i32>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(dialog as usize + DIALOG_ACTIVE_OFFSET)? {
            return Ok(None);
        }
        let style = unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_TYPE_OFFSET) }
            .and_then(|style| u32::try_from(style).ok())
            .and_then(LocalDialogStyle::from_raw)
            .ok_or(DirectClientError::NotReady)?;
        let id = unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let title = unsafe {
            bounded_c_string(
                (dialog as usize + DIALOG_CAPTION_OFFSET) as *const u8,
                DIALOG_CAPTION_CAPACITY,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let server_side = read_r1_bool(dialog as usize + DIALOG_SERVER_SIDE_OFFSET)?;
        Ok(Some(LocalDialogSnapshot {
            id,
            style,
            title,
            server_side,
        }))
    }

    pub(super) fn chat_input_is_active(self) -> Result<bool, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        read_r1_bool(input as usize + INPUT_ENABLED_OFFSET)
    }

    pub(super) fn animation_catalog(self) -> Result<Vec<AnimationSnapshot>, DirectClientError> {
        let table = self.module_base + ANIMATION_TABLE_RVA;
        let length = ANIMATION_TABLE_ENTRY_COUNT * ANIMATION_TABLE_ENTRY_SIZE;
        if !readable_range(table as *const u8, length) {
            return Err(DirectClientError::NotReady);
        }
        let entries = unsafe { std::slice::from_raw_parts(table as *const u8, length) };
        entries
            .chunks_exact(ANIMATION_TABLE_ENTRY_SIZE)
            .map(parse_animation_entry)
            .collect()
    }

    pub(super) fn death_window_is_ready(self) -> bool {
        self.death_window().is_some()
    }

    pub(super) fn game_state(self) -> Result<i32, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_state: NetGameGetStateFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_STATE_RVA) };
        Ok(unsafe { get_state(net_game) })
    }

    pub(super) fn server_info(self) -> Result<ServerInfoSnapshot, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let address = unsafe {
            bounded_c_string(
                net_game
                    .cast::<u8>()
                    .wrapping_add(NET_GAME_HOST_ADDRESS_OFFSET),
                NET_GAME_HOST_STRING_CAPACITY,
            )
        }
        .filter(|address| !address.is_empty())
        .ok_or(DirectClientError::NotReady)?;
        let hostname = unsafe {
            bounded_c_string(
                net_game.cast::<u8>().wrapping_add(NET_GAME_HOSTNAME_OFFSET),
                NET_GAME_HOST_STRING_CAPACITY,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let port = unsafe { read_unaligned::<i32>(net_game as usize + NET_GAME_PORT_OFFSET) }
            .and_then(|port| u16::try_from(port).ok())
            .filter(|port| *port != 0)
            .ok_or(DirectClientError::NotReady)?;
        Ok(ServerInfoSnapshot {
            address,
            hostname,
            port,
        })
    }

    fn dialog(self) -> Option<*mut c_void> {
        let dialog: *mut c_void =
            unsafe { read_pointer(self.module_base + DIALOG_SINGLETON_RVA) }?.cast();
        (!dialog.is_null() && readable_range(dialog.cast(), 1)).then_some(dialog)
    }

    fn chat(self) -> Option<*mut c_void> {
        let chat: *mut c_void =
            unsafe { read_pointer(self.module_base + CHAT_SINGLETON_RVA) }?.cast();
        (!chat.is_null() && readable_range(chat.cast(), 1)).then_some(chat)
    }

    fn scoreboard(self) -> Option<*mut c_void> {
        let scoreboard: *mut c_void =
            unsafe { read_pointer(self.module_base + SCOREBOARD_SINGLETON_RVA) }?.cast();
        (!scoreboard.is_null() && readable_range(scoreboard.cast(), 4)).then_some(scoreboard)
    }

    fn input(self) -> Option<*mut c_void> {
        let input: *mut c_void =
            unsafe { read_pointer(self.module_base + INPUT_SINGLETON_RVA) }?.cast();
        (!input.is_null() && readable_range(input.cast(), INPUT_ENABLED_OFFSET + 4))
            .then_some(input)
    }

    fn death_window(self) -> Option<*mut c_void> {
        let death_window: *mut c_void =
            unsafe { read_pointer(self.module_base + DEATH_WINDOW_SINGLETON_RVA) }?.cast();
        (!death_window.is_null() && readable_range(death_window.cast(), 1)).then_some(death_window)
    }

    /// Copies one remote player through narrowly fingerprinted R1 accessors.
    /// It is invoked only by the host's game-thread pump; no client pointer
    /// survives this method.
    pub(super) fn player_info(
        self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        match unsafe { is_connected(pool, id) } {
            0 => return Ok(None),
            1 => {}
            _ => return Err(DirectClientError::NotReady),
        }

        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let remote = unsafe { get_player(pool, id) };
        if remote.is_null() || !readable_range(remote.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let is_npc: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_NPC_RVA) };
        let get_name: PlayerPoolGetPlayerNameFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_NAME_RVA) };
        let get_score: PlayerPoolGetPlayerStatFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_SCORE_RVA) };
        let get_ping: PlayerPoolGetPlayerStatFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_PING_RVA) };
        let get_colour: RemotePlayerGetColourArgbFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_GET_COLOUR_ARGB_RVA) };
        let is_npc = match unsafe { is_npc(pool, id) } {
            0 => false,
            1 => true,
            _ => return Err(DirectClientError::NotReady),
        };
        let nickname = unsafe { bounded_c_string(get_name(pool, id), 256) }
            .filter(|name| !name.is_empty())
            .ok_or(DirectClientError::NotReady)?;

        Ok(Some(PlayerInfoSnapshot {
            id,
            nickname,
            is_local: false,
            is_npc,
            colour: unsafe { get_colour(remote) },
            score: unsafe { get_score(pool, id) },
            ping: (unsafe { get_ping(pool, id) }).max(0) as u32,
        }))
    }

    /// Reads both R1 `CPlayerPool::GetCount` modes on the game-thread pump.
    /// The resulting scalar pair is published by the host; no pool layout or
    /// pointer crosses this private profile boundary.
    pub(super) fn player_counts(self) -> Result<(u16, u16), DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_count: PlayerPoolGetCountFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_COUNT_RVA) };
        let including_npcs = unsafe { get_count(pool, 1) };
        let excluding_npcs = unsafe { get_count(pool, 0) };
        let including_npcs = u16::try_from(including_npcs)
            .ok()
            .filter(|count| *count <= MAX_SAMP_PLAYERS)
            .ok_or(DirectClientError::NotReady)?;
        let excluding_npcs = u16::try_from(excluding_npcs)
            .ok()
            .filter(|count| *count <= including_npcs)
            .ok_or(DirectClientError::NotReady)?;
        Ok((including_npcs, excluding_npcs))
    }

    pub(super) fn player_max_id(self) -> Result<u16, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null()
            || !readable_range(
                pool.cast(),
                PLAYER_POOL_LARGEST_ID_OFFSET + mem::size_of::<i32>(),
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let largest_id =
            unsafe { read_unaligned::<i32>(pool as usize + PLAYER_POOL_LARGEST_ID_OFFSET) }
                .and_then(|id| u16::try_from(id).ok())
                .filter(|id| *id < MAX_SAMP_PLAYERS)
                .ok_or(DirectClientError::NotReady)?;
        Ok(largest_id)
    }

    /// Reads one R1 vehicle-pool existence flag on the game-thread pump.
    /// Only the copied boolean crosses the private profile boundary.
    pub(super) fn vehicle_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_VEHICLES {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_vehicle_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_VEHICLE_POOL_RVA) };
        let pool = unsafe { get_vehicle_pool(net_game) };
        let checked_len =
            VEHICLE_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if pool.is_null() || !readable_range(pool.cast(), checked_len) {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + VEHICLE_POOL_DOES_EXIST_RVA) };
        match unsafe { does_exist(pool, id) } {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Reads one R1 3D text-label-pool existence flag on the game-thread pump.
    /// Only the copied boolean crosses the private profile boundary.
    pub(super) fn text_label_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_LABEL_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_LABEL_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            LABEL_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        read_r1_bool(pool + LABEL_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>())
    }

    /// Copies one R1 3D text-label record on the game-thread pump. The native
    /// string allocation is read only after its matching pool flag is true,
    /// bounded by the R1 encoded-string limit, and copied before this method
    /// returns. No native pointer crosses the private profile boundary.
    pub(super) fn text_label(
        self,
        id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_LABEL_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_LABEL_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            LABEL_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + LABEL_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>(),
        )? {
            return Ok(None);
        }
        let label = pool + usize::from(id) * LABEL_SIZE;
        if !readable_range(label as *const u8, LABEL_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let text = unsafe { read_unaligned::<usize>(label + LABEL_TEXT_OFFSET) }
            .filter(|text| *text != 0)
            .ok_or(DirectClientError::NotReady)?;
        let text = unsafe { bounded_c_string(text as *const u8, MAX_TEXT_LABEL_TEXT_BYTES + 1) }
            .ok_or(DirectClientError::NotReady)?;
        let colour = unsafe { read_unaligned::<u32>(label + LABEL_COLOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let position = unsafe { read_vector3(label + LABEL_POSITION_OFFSET) }
            .filter(|position| {
                position.x.is_finite() && position.y.is_finite() && position.z.is_finite()
            })
            .ok_or(DirectClientError::NotReady)?;
        let draw_distance = unsafe { read_unaligned::<f32>(label + LABEL_DRAW_DISTANCE_OFFSET) }
            .filter(|draw_distance| draw_distance.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let behind_walls = match unsafe { read_unaligned::<u8>(label + LABEL_BEHIND_WALLS_OFFSET) }
        {
            Some(0) => false,
            Some(1) => true,
            _ => return Err(DirectClientError::NotReady),
        };
        let attached_player =
            unsafe { read_unaligned::<u16>(label + LABEL_ATTACHED_PLAYER_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let attached_vehicle =
            unsafe { read_unaligned::<u16>(label + LABEL_ATTACHED_VEHICLE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        Ok(Some(TextLabelSnapshot {
            id,
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id: (attached_player != u16::MAX).then_some(attached_player),
            attached_vehicle_id: (attached_vehicle != u16::MAX).then_some(attached_vehicle),
        }))
    }

    /// Reads one R1 textdraw-pool existence flag on the game-thread pump.
    /// The raw pool index covers the 2,048 global and 256 local slots. Only
    /// the copied boolean crosses the private profile boundary.
    pub(super) fn textdraw_exists(self, pool_index: u16) -> Result<bool, DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )
    }

    /// Copies one R1 numeric textdraw record on the game-thread pump. The raw
    /// index preserves the native 2,048-global then 256-local pool order. No
    /// textdraw/pool pointer or unproven display-string buffer crosses the
    /// private profile boundary.
    pub(super) fn textdraw(
        self,
        pool_index: u16,
    ) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, flags_end) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Ok(None);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let last_field_end = TEXTDRAW_MODEL_COLOUR2_OFFSET + mem::size_of::<u16>();
        if !readable_range(object as *const u8, last_field_end) {
            return Err(DirectClientError::NotReady);
        }
        let letter_width = unsafe { read_unaligned::<f32>(object + TEXTDRAW_LETTER_WIDTH_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let letter_height =
            unsafe { read_unaligned::<f32>(object + TEXTDRAW_LETTER_HEIGHT_OFFSET) }
                .filter(|value| value.is_finite())
                .ok_or(DirectClientError::NotReady)?;
        let letter_colour =
            unsafe { read_unaligned::<u32>(object + TEXTDRAW_LETTER_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let x = unsafe { read_unaligned::<f32>(object + TEXTDRAW_X_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let y = unsafe { read_unaligned::<f32>(object + TEXTDRAW_Y_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let box_width = unsafe { read_unaligned::<f32>(object + TEXTDRAW_BOX_WIDTH_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let box_height = unsafe { read_unaligned::<f32>(object + TEXTDRAW_BOX_HEIGHT_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let box_colour = unsafe { read_unaligned::<u32>(object + TEXTDRAW_BOX_COLOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let background_colour =
            unsafe { read_unaligned::<u32>(object + TEXTDRAW_BACKGROUND_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let style = unsafe { read_unaligned::<i32>(object + TEXTDRAW_STYLE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let model_id = unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let rotation = unsafe { read_vector3(object + TEXTDRAW_ROTATION_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let zoom = unsafe { read_unaligned::<f32>(object + TEXTDRAW_ZOOM_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        Ok(Some(TextdrawSnapshot {
            pool_index,
            letter_width,
            letter_height,
            letter_colour,
            x,
            y,
            shadow: unsafe { read_unaligned::<u8>(object + TEXTDRAW_SHADOW_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            outline: unsafe { read_unaligned::<u8>(object + TEXTDRAW_OUTLINE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            background_colour,
            style,
            proportional: read_u8_bool(object + TEXTDRAW_PROPORTIONAL_OFFSET)?,
            align_left: read_u8_bool(object + TEXTDRAW_ALIGN_LEFT_OFFSET)?,
            align_center: read_u8_bool(object + TEXTDRAW_ALIGN_CENTER_OFFSET)?,
            align_right: read_u8_bool(object + TEXTDRAW_ALIGN_RIGHT_OFFSET)?,
            box_enabled: read_u8_bool(object + TEXTDRAW_BOX_ENABLED_OFFSET)?,
            box_width,
            box_height,
            box_colour,
            model_id,
            rotation,
            zoom,
            model_colour1: unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_COLOUR1_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            model_colour2: unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_COLOUR2_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
        }))
    }

    /// Reads one R1 object-pool existence flag on the game-thread pump.
    /// Only the copied boolean crosses the private profile boundary.
    pub(super) fn object_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_OBJECTS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_OBJECT_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_OBJECT_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            OBJECT_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        read_r1_bool(pool + OBJECT_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>())
    }

    /// Copies one R1 gangzone record on the game-thread pump. No client or
    /// GTA pointer crosses the private profile boundary.
    pub(super) fn gangzone(self, id: u16) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_GANGZONES {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_GANGZONE_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_GANGZONE_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            GANGZONE_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + GANGZONE_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>(),
        )? {
            return Ok(None);
        }
        let gangzone =
            unsafe { read_unaligned::<usize>(pool + usize::from(id) * mem::size_of::<usize>()) }
                .filter(|gangzone| *gangzone != 0)
                .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            gangzone as *const u8,
            GANGZONE_ALTERNATE_COLOUR_OFFSET + mem::size_of::<u32>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let left = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_LEFT_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let bottom = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_BOTTOM_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let right = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_RIGHT_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let top = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_TOP_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let colour = unsafe { read_unaligned::<u32>(gangzone + GANGZONE_COLOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let alternate_colour =
            unsafe { read_unaligned::<u32>(gangzone + GANGZONE_ALTERNATE_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        Ok(Some(GangzoneSnapshot {
            id,
            left,
            bottom,
            right,
            top,
            colour,
            alternate_colour,
        }))
    }

    pub(super) fn local_player(self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let get_local_player: PlayerPoolGetLocalPlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
        let local = unsafe { get_local_player(pool) };
        if local.is_null() || !readable_range(local.cast(), LOCAL_PLAYER_INCAR_OFFSET + 0x30) {
            return Err(DirectClientError::NotReady);
        }

        let id = unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
            .and_then(assigned_player_id)
            .ok_or(DirectClientError::NotReady)?;

        let get_ped: LocalPlayerGetPedFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_GET_PED_RVA) };
        let ped = unsafe { get_ped(local) };
        if ped.is_null()
            || !readable_range(
                ped.cast(),
                SAMP_PED_GAME_PED_OFFSET + mem::size_of::<usize>(),
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let game_ped = unsafe { read_pointer(ped as usize + SAMP_PED_GAME_PED_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if game_ped.is_null() || !readable_range(game_ped.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let get_name: PlayerPoolGetLocalNameFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_NAME_RVA) };
        let get_score: PlayerPoolGetLocalScoreFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_SCORE_RVA) };
        let get_ping: PlayerPoolGetLocalPingFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PING_RVA) };
        let get_colour: LocalPlayerGetColourArgbFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_GET_COLOUR_ARGB_RVA) };
        let get_health: PedGetStatFn =
            unsafe { mem::transmute(self.module_base + PED_GET_HEALTH_RVA) };
        let get_armour: PedGetStatFn =
            unsafe { mem::transmute(self.module_base + PED_GET_ARMOUR_RVA) };

        let nickname =
            unsafe { bounded_c_string(get_name(pool), 256) }.ok_or(DirectClientError::NotReady)?;
        let current_vehicle =
            unsafe { read_unaligned::<u16>(local as usize + LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let vehicle_id = (current_vehicle != INVALID_ID).then_some(current_vehicle);
        let (position, velocity) = if vehicle_id.is_some() {
            (
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_INCAR_OFFSET
                            + LOCAL_PLAYER_INCAR_POSITION_OFFSET,
                    )
                },
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_INCAR_OFFSET
                            + LOCAL_PLAYER_INCAR_SPEED_OFFSET,
                    )
                },
            )
        } else {
            (
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_ONFOOT_OFFSET
                            + LOCAL_PLAYER_ONFOOT_POSITION_OFFSET,
                    )
                },
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_ONFOOT_OFFSET
                            + LOCAL_PLAYER_ONFOOT_SPEED_OFFSET,
                    )
                },
            )
        };
        let position = position.ok_or(DirectClientError::NotReady)?;
        let velocity = velocity.ok_or(DirectClientError::NotReady)?;
        let spawned = unsafe { read_unaligned::<u32>(local as usize + LOCAL_PLAYER_ACTIVE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?
            != 0;
        let special_action = unsafe {
            read_unaligned::<u8>(
                local as usize
                    + LOCAL_PLAYER_ONFOOT_OFFSET
                    + LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let animation = unsafe {
            read_unaligned::<u32>(
                local as usize + LOCAL_PLAYER_ONFOOT_OFFSET + LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET,
            )
        }
        .ok_or(DirectClientError::NotReady)?;

        Ok(LocalPlayerSnapshot {
            id,
            nickname,
            colour: unsafe { get_colour(local) },
            spawned,
            health: unsafe { get_health(ped) },
            armour: unsafe { get_armour(ped) },
            position,
            velocity,
            special_action,
            animation_id: animation as u16,
            vehicle_id,
            score: unsafe { get_score(pool) },
            ping: (unsafe { get_ping(pool) }).max(0) as u32,
        })
    }

    fn net_game(self) -> Option<*mut c_void> {
        let net_game: *mut c_void =
            unsafe { read_pointer(self.module_base + NET_GAME_SINGLETON_RVA) }?.cast();
        (!net_game.is_null() && readable_range(net_game.cast(), 1)).then_some(net_game)
    }

    fn game(self) -> Option<*mut c_void> {
        let game: *mut c_void =
            unsafe { read_pointer(self.module_base + GAME_SINGLETON_RVA) }?.cast();
        (!game.is_null() && readable_range(game.cast(), GAME_CURSOR_MODE_OFFSET + 4))
            .then_some(game)
    }
}

fn assigned_player_id(id: u16) -> Option<u16> {
    (id != INVALID_ID).then_some(id)
}

type DialogShowFn = unsafe extern "thiscall" fn(
    *mut c_void,
    i32,
    i32,
    *const i8,
    *const i8,
    *const i8,
    *const i8,
    i32,
);
type ChatAddEntryFn = unsafe extern "thiscall" fn(*mut c_void, i32, *const i8, *const i8, u32, u32);
type ChatGetModeFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type DeathWindowAddMessageFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, *const i8, u32, u32, u8);
type NetGameGetStateFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type NetGameGetPlayerPoolFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type PlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type PlayerPoolGetLocalNameFn = unsafe extern "thiscall" fn(*mut c_void) -> *const u8;
type PlayerPoolGetLocalScoreFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type PlayerPoolGetLocalPingFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type PlayerPoolPlayerBooleanFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type PlayerPoolGetRemotePlayerFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *mut c_void;
type PlayerPoolGetPlayerNameFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *const u8;
type PlayerPoolGetPlayerStatFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type PlayerPoolGetCountFn = unsafe extern "thiscall" fn(*mut c_void, i32) -> i32;
type LocalPlayerGetPedFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type LocalPlayerGetColourArgbFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type RemotePlayerGetColourArgbFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type PedGetStatFn = unsafe extern "thiscall" fn(*mut c_void) -> f32;

unsafe fn samp_r1_pe_matches(module_base: usize) -> bool {
    let Some(nt_header) = (unsafe { pe_header(module_base) }) else {
        return false;
    };
    (unsafe { nt_header.add(8).cast::<u32>().read_unaligned() } == SAMP_R1_TIMESTAMP)
        && (unsafe { nt_header.add(40).cast::<u32>().read_unaligned() } == SAMP_R1_ENTRY_POINT)
}

unsafe fn gta_sa_10_us_matches() -> bool {
    let module = unsafe { GetModuleHandleA(c"gta_sa.exe".as_ptr().cast()) };
    if module.is_null() {
        return false;
    }
    let Some(nt_header) = (unsafe { pe_header(module as usize) }) else {
        return false;
    };
    let machine = unsafe { nt_header.add(4).cast::<u16>().read_unaligned() };
    let image_base = unsafe { nt_header.add(52).cast::<u32>().read_unaligned() };
    let image_size = unsafe { nt_header.add(80).cast::<u32>().read_unaligned() };
    let entry_point = unsafe { nt_header.add(40).cast::<u32>().read_unaligned() };
    machine == 0x014C
        && image_base == GTA_SA_10_US_IMAGE_BASE
        && image_size == GTA_SA_10_US_IMAGE_SIZE
        && entry_point == GTA_SA_10_US_ENTRY_POINT
        && unsafe { plausible_code(module as usize + entry_point as usize) }
}

unsafe fn r1_targets_match(module_base: usize) -> bool {
    // The prologue is the R1 CDialog::Show call target; verify its signature
    // and ensure every additional native entry is mapped executable code before
    // publishing the profile. A mismatch leaves direct helpers unsupported.
    let show = module_base + DIALOG_SHOW_RVA;
    code_matches(show, &DIALOG_SHOW_SIGNATURE)
        && code_matches(module_base + CHAT_ADD_ENTRY_RVA, &CHAT_ADD_ENTRY_SIGNATURE)
        && code_matches(module_base + CHAT_GET_MODE_RVA, &CHAT_GET_MODE_SIGNATURE)
        && code_matches(module_base + DIALOG_SHOW_RVA, &DIALOG_SHOW_ACTIVE_SIGNATURE)
        && code_matches(
            module_base + DIALOG_SHOW_RVA + 0x48,
            &DIALOG_SHOW_CORE_FIELDS_SIGNATURE,
        )
        && code_matches(module_base + INPUT_OPEN_RVA, &INPUT_OPEN_SIGNATURE)
        && code_matches(module_base + INPUT_CLOSE_RVA, &INPUT_CLOSE_SIGNATURE)
        && code_matches(
            module_base + SCOREBOARD_CLOSE_RVA,
            &SCOREBOARD_CLOSE_SIGNATURE,
        )
        && code_matches(
            module_base + SCOREBOARD_ENABLE_RVA,
            &SCOREBOARD_ENABLE_SIGNATURE,
        )
        && code_matches(
            module_base + GAME_PROCESS_INPUT_ENABLING_RVA,
            &GAME_PROCESS_INPUT_ENABLING_SIGNATURE,
        )
        && bytes_match(
            module_base + ANIMATION_TABLE_RVA,
            &ANIMATION_TABLE_SIGNATURE,
        )
        && code_matches(
            module_base + DEATH_WINDOW_ADD_MESSAGE_RVA,
            &DEATH_WINDOW_ADD_MESSAGE_SIGNATURE,
        )
        && code_matches(
            module_base + DEATH_WINDOW_ADD_ENTRY_RVA,
            &DEATH_WINDOW_ADD_ENTRY_SIGNATURE,
        )
        && code_matches(
            module_base + NET_GAME_GET_STATE_RVA,
            &NET_GAME_GET_STATE_SIGNATURE,
        )
        && code_matches(
            module_base + NET_GAME_GET_PLAYER_POOL_RVA,
            &NET_GAME_GET_PLAYER_POOL_SIGNATURE,
        )
        && code_matches(
            module_base + NET_GAME_GET_VEHICLE_POOL_RVA,
            &NET_GAME_GET_VEHICLE_POOL_SIGNATURE,
        )
        && code_matches(
            module_base + NET_GAME_RESET_LABEL_POOL_RVA + 0x15,
            &NET_GAME_RESET_LABEL_POOL_FIELDS_SIGNATURE,
        )
        && code_matches(
            module_base + TEXT_LABEL_POOL_CREATE_RVA + 0x6B,
            &TEXT_LABEL_POOL_CREATE_TEXT_ALLOCATION_SIGNATURE,
        )
        && code_matches(
            module_base + TEXT_LABEL_POOL_CREATE_RVA + 0x82,
            &TEXT_LABEL_POOL_CREATE_TEXT_COPY_SIGNATURE,
        )
        && code_matches(
            module_base + TEXT_LABEL_POOL_CREATE_RVA + 0xCD,
            &TEXT_LABEL_POOL_CREATE_SCALAR_FIELDS_SIGNATURE,
        )
        && code_matches(
            module_base + TEXTDRAW_CTOR_RVA + 0x19,
            &TEXTDRAW_CTOR_CORE_FIELDS_SIGNATURE,
        )
        && code_matches(
            module_base + TEXTDRAW_CTOR_RVA + 0xB4,
            &TEXTDRAW_CTOR_MODEL_FIELDS_SIGNATURE,
        )
        && code_matches(
            module_base + NET_GAME_RESET_TEXTDRAW_POOL_RVA + 0x15,
            &NET_GAME_RESET_TEXTDRAW_POOL_FIELDS_SIGNATURE,
        )
        && code_matches(
            module_base + NET_GAME_RESET_OBJECT_POOL_RVA + 0x15,
            &NET_GAME_RESET_OBJECT_POOL_FIELDS_SIGNATURE,
        )
        && code_matches(
            module_base + NET_GAME_RESET_GANGZONE_POOL_RVA + 0x15,
            &NET_GAME_RESET_GANGZONE_POOL_FIELDS_SIGNATURE,
        )
        && code_matches(
            module_base + GANG_ZONE_POOL_CREATE_RVA + 0x19,
            &GANG_ZONE_POOL_CREATE_POOL_FIELDS_SIGNATURE,
        )
        && code_matches(
            module_base + GANG_ZONE_POOL_CREATE_RVA + 0x39,
            &GANG_ZONE_POOL_CREATE_RECORD_FIELDS_SIGNATURE,
        )
        && code_matches(
            module_base + PLAYER_POOL_IS_CONNECTED_RVA,
            &PLAYER_POOL_IS_CONNECTED_SIGNATURE,
        )
        && code_matches(
            module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA,
            &PLAYER_POOL_GET_REMOTE_PLAYER_SIGNATURE,
        )
        && code_matches(
            module_base + PLAYER_POOL_IS_NPC_RVA,
            &PLAYER_POOL_IS_NPC_SIGNATURE,
        )
        && code_matches(
            module_base + PLAYER_POOL_GET_NAME_RVA,
            &PLAYER_POOL_GET_NAME_SIGNATURE,
        )
        && code_matches(
            module_base + PLAYER_POOL_GET_SCORE_RVA,
            &PLAYER_POOL_GET_SCORE_SIGNATURE,
        )
        && code_matches(
            module_base + PLAYER_POOL_GET_PING_RVA,
            &PLAYER_POOL_GET_PING_SIGNATURE,
        )
        && code_matches(
            module_base + PLAYER_POOL_GET_COUNT_RVA,
            &PLAYER_POOL_GET_COUNT_SIGNATURE,
        )
        && code_matches(
            module_base + PLAYER_POOL_UPDATE_LARGEST_ID_RVA,
            &PLAYER_POOL_UPDATE_LARGEST_ID_SIGNATURE,
        )
        && code_matches(
            module_base + VEHICLE_POOL_DOES_EXIST_RVA,
            &VEHICLE_POOL_DOES_EXIST_SIGNATURE,
        )
        && code_matches(
            module_base + REMOTE_PLAYER_GET_COLOUR_ARGB_RVA,
            &REMOTE_PLAYER_GET_COLOUR_ARGB_SIGNATURE,
        )
        && [
            PLAYER_POOL_GET_LOCAL_PLAYER_RVA,
            PLAYER_POOL_GET_LOCAL_NAME_RVA,
            PLAYER_POOL_GET_LOCAL_SCORE_RVA,
            PLAYER_POOL_GET_LOCAL_PING_RVA,
            LOCAL_PLAYER_GET_PED_RVA,
            LOCAL_PLAYER_GET_COLOUR_ARGB_RVA,
            PED_GET_HEALTH_RVA,
            PED_GET_ARMOUR_RVA,
        ]
        .into_iter()
        .all(|rva| unsafe { plausible_code(module_base + rva) })
}

fn code_matches(address: usize, signature: &[u8]) -> bool {
    bytes_match(address, signature)
}

fn bytes_match(address: usize, signature: &[u8]) -> bool {
    readable_range(address as *const u8, signature.len())
        && unsafe { std::slice::from_raw_parts(address as *const u8, signature.len()) } == signature
}

unsafe fn pe_header(base: usize) -> Option<*const u8> {
    let image = base as *const u8;
    if !readable_range(image, 0x40) || (unsafe { image.cast::<u16>().read_unaligned() } != 0x5A4D) {
        return None;
    }
    let nt_offset = unsafe { image.add(0x3C).cast::<u32>().read_unaligned() } as usize;
    if nt_offset > 0x1000 || !readable_range(unsafe { image.add(nt_offset) }, 84) {
        return None;
    }
    let nt_header = unsafe { image.add(nt_offset) };
    ((unsafe { nt_header.cast::<u32>().read_unaligned() } == 0x0000_4550)
        && (unsafe { nt_header.add(24).cast::<u16>().read_unaligned() } == 0x10B))
        .then_some(nt_header)
}

unsafe fn read_pointer(address: usize) -> Option<*mut u8> {
    unsafe { read_unaligned::<usize>(address) }.map(|value| value as *mut u8)
}

unsafe fn read_unaligned<T: Copy>(address: usize) -> Option<T> {
    readable_range(address as *const u8, mem::size_of::<T>())
        .then(|| unsafe { (address as *const T).read_unaligned() })
}

unsafe fn read_vector3(address: usize) -> Option<Vector3> {
    Some(Vector3 {
        x: unsafe { read_unaligned::<f32>(address) }?,
        y: unsafe { read_unaligned::<f32>(address.checked_add(4)?) }?,
        z: unsafe { read_unaligned::<f32>(address.checked_add(8)?) }?,
    })
}

fn read_r1_bool(address: usize) -> Result<bool, DirectClientError> {
    match unsafe { read_unaligned::<i32>(address) } {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(DirectClientError::NotReady),
    }
}

fn read_u8_bool(address: usize) -> Result<bool, DirectClientError> {
    match unsafe { read_unaligned::<u8>(address) } {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(DirectClientError::NotReady),
    }
}

fn parse_animation_entry(entry: &[u8]) -> Result<AnimationSnapshot, DirectClientError> {
    let length = entry
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(entry.len());
    let Some(separator) = entry[..length].iter().position(|byte| *byte == b':') else {
        return Err(DirectClientError::NotReady);
    };
    let (name, file) = (&entry[..separator], &entry[separator + 1..length]);
    if name.is_empty() || file.is_empty() || file.contains(&b':') {
        return Err(DirectClientError::NotReady);
    }
    Ok(AnimationSnapshot {
        name: name.to_vec(),
        file: file.to_vec(),
    })
}

unsafe fn bounded_c_string(pointer: *const u8, maximum: usize) -> Option<Vec<u8>> {
    if pointer.is_null() {
        return None;
    }
    let mut output = Vec::new();
    for index in 0..maximum {
        let byte = unsafe { read_unaligned::<u8>((pointer as usize).checked_add(index)?) }?;
        if byte == 0 {
            return Some(output);
        }
        output.push(byte);
    }
    None
}

fn nul_terminated(mut value: Vec<u8>) -> Vec<u8> {
    value.push(0);
    value
}

fn readable_range(address: *const u8, length: usize) -> bool {
    if address.is_null() || length == 0 {
        return length == 0;
    }
    let Some(end) = (address as usize).checked_add(length) else {
        return false;
    };
    let mut info = mem::MaybeUninit::<MEMORY_BASIC_INFORMATION>::zeroed();
    let queried = unsafe {
        VirtualQuery(
            address.cast(),
            info.as_mut_ptr(),
            mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    if queried == 0 {
        return false;
    }
    let info = unsafe { info.assume_init() };
    let Some(region_end) = (info.BaseAddress as usize).checked_add(info.RegionSize) else {
        return false;
    };
    info.State == MEM_COMMIT
        && info.Protect & (PAGE_GUARD | PAGE_NOACCESS) == 0
        && end <= region_end
}

unsafe fn plausible_code(address: usize) -> bool {
    if !readable_range(address as *const u8, 3) {
        return false;
    }
    let bytes = unsafe { std::slice::from_raw_parts(address as *const u8, 3) };
    !bytes.iter().all(|byte| *byte == 0 || *byte == 0xCC) && !matches!(bytes[0], 0xC2 | 0xC3 | 0xCC)
}

#[cfg(test)]
mod tests {
    use super::{
        ANIMATION_TABLE_SIGNATURE, CHAT_ADD_ENTRY_SIGNATURE, CHAT_GET_MODE_SIGNATURE,
        DEATH_WINDOW_ADD_ENTRY_SIGNATURE, DEATH_WINDOW_ADD_MESSAGE_SIGNATURE, DIALOG_ACTIVE_OFFSET,
        DIALOG_CAPTION_OFFSET, DIALOG_ID_OFFSET, DIALOG_SERVER_SIDE_OFFSET,
        DIALOG_SHOW_ACTIVE_SIGNATURE, DIALOG_SHOW_CORE_FIELDS_SIGNATURE, DIALOG_SHOW_SIGNATURE,
        DIALOG_TYPE_OFFSET, GAME_CURSOR_MODE_OFFSET, GAME_PROCESS_INPUT_ENABLING_SIGNATURE,
        GANG_ZONE_POOL_CREATE_POOL_FIELDS_SIGNATURE, GANG_ZONE_POOL_CREATE_RECORD_FIELDS_SIGNATURE,
        GANGZONE_POOL_NOT_EMPTY_OFFSET, INPUT_CLOSE_SIGNATURE, INPUT_ENABLED_OFFSET,
        INPUT_OPEN_SIGNATURE, LABEL_ATTACHED_PLAYER_OFFSET, LABEL_ATTACHED_VEHICLE_OFFSET,
        LABEL_BEHIND_WALLS_OFFSET, LABEL_COLOUR_OFFSET, LABEL_DRAW_DISTANCE_OFFSET,
        LABEL_POOL_NOT_EMPTY_OFFSET, LABEL_POSITION_OFFSET, LABEL_SIZE, LABEL_TEXT_OFFSET,
        LOCAL_PLAYER_ACTIVE_OFFSET, LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET, LOCAL_PLAYER_INCAR_OFFSET,
        LOCAL_PLAYER_INCAR_POSITION_OFFSET, LOCAL_PLAYER_INCAR_SPEED_OFFSET,
        LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET, LOCAL_PLAYER_ONFOOT_OFFSET,
        LOCAL_PLAYER_ONFOOT_POSITION_OFFSET, LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET,
        LOCAL_PLAYER_ONFOOT_SPEED_OFFSET, MAX_TEXT_LABEL_TEXT_BYTES,
        NET_GAME_GET_PLAYER_POOL_SIGNATURE, NET_GAME_GET_STATE_SIGNATURE,
        NET_GAME_GET_VEHICLE_POOL_SIGNATURE, NET_GAME_HOST_ADDRESS_OFFSET,
        NET_GAME_HOSTNAME_OFFSET, NET_GAME_POOLS_GANGZONE_POOL_OFFSET,
        NET_GAME_POOLS_LABEL_POOL_OFFSET, NET_GAME_POOLS_OBJECT_POOL_OFFSET, NET_GAME_POOLS_OFFSET,
        NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET, NET_GAME_PORT_OFFSET,
        NET_GAME_RESET_GANGZONE_POOL_FIELDS_SIGNATURE, NET_GAME_RESET_LABEL_POOL_FIELDS_SIGNATURE,
        NET_GAME_RESET_OBJECT_POOL_FIELDS_SIGNATURE, NET_GAME_RESET_TEXTDRAW_POOL_FIELDS_SIGNATURE,
        OBJECT_POOL_NOT_EMPTY_OFFSET, PLAYER_POOL_GET_COUNT_SIGNATURE,
        PLAYER_POOL_GET_NAME_SIGNATURE, PLAYER_POOL_GET_PING_SIGNATURE,
        PLAYER_POOL_GET_REMOTE_PLAYER_SIGNATURE, PLAYER_POOL_GET_SCORE_SIGNATURE,
        PLAYER_POOL_IS_CONNECTED_SIGNATURE, PLAYER_POOL_IS_NPC_SIGNATURE,
        PLAYER_POOL_LARGEST_ID_OFFSET, PLAYER_POOL_LOCAL_ID_OFFSET,
        PLAYER_POOL_UPDATE_LARGEST_ID_SIGNATURE, REMOTE_PLAYER_GET_COLOUR_ARGB_SIGNATURE,
        SAMP_PED_GAME_PED_OFFSET, SCOREBOARD_CLOSE_SIGNATURE, SCOREBOARD_ENABLE_SIGNATURE,
        SCOREBOARD_ENABLED_OFFSET, TEXT_LABEL_POOL_CREATE_SCALAR_FIELDS_SIGNATURE,
        TEXT_LABEL_POOL_CREATE_TEXT_ALLOCATION_SIGNATURE,
        TEXT_LABEL_POOL_CREATE_TEXT_COPY_SIGNATURE, TEXTDRAW_ALIGN_CENTER_OFFSET,
        TEXTDRAW_ALIGN_LEFT_OFFSET, TEXTDRAW_ALIGN_RIGHT_OFFSET, TEXTDRAW_BACKGROUND_COLOUR_OFFSET,
        TEXTDRAW_BOX_COLOUR_OFFSET, TEXTDRAW_BOX_ENABLED_OFFSET, TEXTDRAW_BOX_HEIGHT_OFFSET,
        TEXTDRAW_BOX_WIDTH_OFFSET, TEXTDRAW_CTOR_CORE_FIELDS_SIGNATURE,
        TEXTDRAW_CTOR_MODEL_FIELDS_SIGNATURE, TEXTDRAW_DATA_OFFSET, TEXTDRAW_LETTER_COLOUR_OFFSET,
        TEXTDRAW_LETTER_HEIGHT_OFFSET, TEXTDRAW_LETTER_WIDTH_OFFSET, TEXTDRAW_MODEL_COLOUR1_OFFSET,
        TEXTDRAW_MODEL_COLOUR2_OFFSET, TEXTDRAW_MODEL_ID_OFFSET, TEXTDRAW_OUTLINE_OFFSET,
        TEXTDRAW_POOL_NOT_EMPTY_OFFSET, TEXTDRAW_POOL_OBJECTS_OFFSET, TEXTDRAW_PROPORTIONAL_OFFSET,
        TEXTDRAW_ROTATION_OFFSET, TEXTDRAW_SHADOW_OFFSET, TEXTDRAW_STYLE_OFFSET, TEXTDRAW_X_OFFSET,
        TEXTDRAW_Y_OFFSET, TEXTDRAW_ZOOM_OFFSET, VEHICLE_POOL_DOES_EXIST_SIGNATURE,
        VEHICLE_POOL_NOT_EMPTY_OFFSET, assigned_player_id, bounded_c_string, nul_terminated,
        parse_animation_entry,
    };

    unsafe extern "C" {
        fn rak_samp_fixture_r1_onfoot_size() -> usize;
        fn rak_samp_fixture_r1_incar_size() -> usize;
        fn rak_samp_fixture_r1_local_player_prefix_size() -> usize;
        fn rak_samp_fixture_r1_local_active_offset() -> usize;
        fn rak_samp_fixture_r1_local_current_vehicle_offset() -> usize;
        fn rak_samp_fixture_r1_local_onfoot_offset() -> usize;
        fn rak_samp_fixture_r1_onfoot_position_offset() -> usize;
        fn rak_samp_fixture_r1_onfoot_speed_offset() -> usize;
        fn rak_samp_fixture_r1_onfoot_special_action_offset() -> usize;
        fn rak_samp_fixture_r1_onfoot_animation_offset() -> usize;
        fn rak_samp_fixture_r1_incar_position_offset() -> usize;
        fn rak_samp_fixture_r1_incar_speed_offset() -> usize;
        fn rak_samp_fixture_r1_ped_game_ped_offset() -> usize;
        fn rak_samp_fixture_r1_player_pool_local_id_offset() -> usize;
        fn rak_samp_fixture_r1_player_pool_largest_id_offset() -> usize;
        fn rak_samp_fixture_r1_vehicle_pool_not_empty_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_host_address_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_hostname_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_port_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_game_state_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_pools_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_pools_label_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_pools_text_draw_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_pools_object_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_pools_gang_zone_offset() -> usize;
        fn rak_samp_fixture_r1_label_pool_not_empty_offset() -> usize;
        fn rak_samp_fixture_r1_text_label_size() -> usize;
        fn rak_samp_fixture_r1_text_label_text_offset() -> usize;
        fn rak_samp_fixture_r1_text_label_colour_offset() -> usize;
        fn rak_samp_fixture_r1_text_label_position_offset() -> usize;
        fn rak_samp_fixture_r1_text_label_draw_distance_offset() -> usize;
        fn rak_samp_fixture_r1_text_label_behind_walls_offset() -> usize;
        fn rak_samp_fixture_r1_text_label_attached_player_offset() -> usize;
        fn rak_samp_fixture_r1_text_label_attached_vehicle_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_pool_not_empty_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_pool_objects_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_data_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_letter_width_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_letter_height_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_letter_colour_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_align_center_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_box_enabled_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_box_width_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_box_height_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_box_colour_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_proportional_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_background_colour_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_shadow_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_outline_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_align_left_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_align_right_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_style_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_x_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_y_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_model_id_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_rotation_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_zoom_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_model_colour1_offset() -> usize;
        fn rak_samp_fixture_r1_textdraw_model_colour2_offset() -> usize;
        fn rak_samp_fixture_r1_object_pool_not_empty_offset() -> usize;
        fn rak_samp_fixture_r1_gangzone_pool_not_empty_offset() -> usize;
        fn rak_samp_fixture_r1_gangzone_size() -> usize;
        fn rak_samp_fixture_r1_game_cursor_mode_offset() -> usize;
        fn rak_samp_fixture_r1_scoreboard_enabled_offset() -> usize;
        fn rak_samp_fixture_r1_dialog_active_offset() -> usize;
        fn rak_samp_fixture_r1_dialog_type_offset() -> usize;
        fn rak_samp_fixture_r1_dialog_id_offset() -> usize;
        fn rak_samp_fixture_r1_dialog_caption_offset() -> usize;
        fn rak_samp_fixture_r1_dialog_server_side_offset() -> usize;
        fn rak_samp_fixture_r1_input_enabled_offset() -> usize;
    }

    #[test]
    fn r1_sync_offsets_match_the_independent_x86_fixture() {
        unsafe {
            assert_eq!(rak_samp_fixture_r1_onfoot_size(), 68);
            assert_eq!(rak_samp_fixture_r1_incar_size(), 63);
            assert_eq!(rak_samp_fixture_r1_local_player_prefix_size(), 92);
            assert_eq!(
                rak_samp_fixture_r1_local_active_offset(),
                LOCAL_PLAYER_ACTIVE_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_local_current_vehicle_offset(),
                LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_local_onfoot_offset(),
                LOCAL_PLAYER_ONFOOT_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_onfoot_position_offset(),
                LOCAL_PLAYER_ONFOOT_POSITION_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_onfoot_speed_offset(),
                LOCAL_PLAYER_ONFOOT_SPEED_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_onfoot_special_action_offset(),
                LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_onfoot_animation_offset(),
                LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET
            );
            assert_eq!(
                LOCAL_PLAYER_ONFOOT_OFFSET + 68 + 24 + 54,
                LOCAL_PLAYER_INCAR_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_incar_position_offset(),
                LOCAL_PLAYER_INCAR_POSITION_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_incar_speed_offset(),
                LOCAL_PLAYER_INCAR_SPEED_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_ped_game_ped_offset(),
                SAMP_PED_GAME_PED_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_player_pool_local_id_offset(),
                PLAYER_POOL_LOCAL_ID_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_player_pool_largest_id_offset(),
                PLAYER_POOL_LARGEST_ID_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_vehicle_pool_not_empty_offset(),
                VEHICLE_POOL_NOT_EMPTY_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_net_game_host_address_offset(),
                NET_GAME_HOST_ADDRESS_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_net_game_hostname_offset(),
                NET_GAME_HOSTNAME_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_net_game_port_offset(),
                NET_GAME_PORT_OFFSET
            );
            assert_eq!(rak_samp_fixture_r1_net_game_game_state_offset(), 0x3BD);
            assert_eq!(
                rak_samp_fixture_r1_net_game_pools_offset(),
                NET_GAME_POOLS_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_net_game_pools_label_offset(),
                NET_GAME_POOLS_LABEL_POOL_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_net_game_pools_text_draw_offset(),
                NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_net_game_pools_object_offset(),
                NET_GAME_POOLS_OBJECT_POOL_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_net_game_pools_gang_zone_offset(),
                NET_GAME_POOLS_GANGZONE_POOL_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_label_pool_not_empty_offset(),
                LABEL_POOL_NOT_EMPTY_OFFSET
            );
            assert_eq!(rak_samp_fixture_r1_text_label_size(), LABEL_SIZE);
            assert_eq!(
                rak_samp_fixture_r1_text_label_text_offset(),
                LABEL_TEXT_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_text_label_colour_offset(),
                LABEL_COLOUR_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_text_label_position_offset(),
                LABEL_POSITION_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_text_label_draw_distance_offset(),
                LABEL_DRAW_DISTANCE_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_text_label_behind_walls_offset(),
                LABEL_BEHIND_WALLS_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_text_label_attached_player_offset(),
                LABEL_ATTACHED_PLAYER_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_text_label_attached_vehicle_offset(),
                LABEL_ATTACHED_VEHICLE_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_textdraw_pool_not_empty_offset(),
                TEXTDRAW_POOL_NOT_EMPTY_OFFSET
            );
            let textdraw_offsets = [
                (
                    rak_samp_fixture_r1_textdraw_pool_objects_offset(),
                    TEXTDRAW_POOL_OBJECTS_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_data_offset(),
                    TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_letter_width_offset(),
                    TEXTDRAW_LETTER_WIDTH_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_letter_height_offset(),
                    TEXTDRAW_LETTER_HEIGHT_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_letter_colour_offset(),
                    TEXTDRAW_LETTER_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_align_center_offset(),
                    TEXTDRAW_ALIGN_CENTER_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_box_enabled_offset(),
                    TEXTDRAW_BOX_ENABLED_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_box_width_offset(),
                    TEXTDRAW_BOX_WIDTH_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_box_height_offset(),
                    TEXTDRAW_BOX_HEIGHT_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_box_colour_offset(),
                    TEXTDRAW_BOX_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_proportional_offset(),
                    TEXTDRAW_PROPORTIONAL_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_background_colour_offset(),
                    TEXTDRAW_BACKGROUND_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_shadow_offset(),
                    TEXTDRAW_SHADOW_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_outline_offset(),
                    TEXTDRAW_OUTLINE_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_align_left_offset(),
                    TEXTDRAW_ALIGN_LEFT_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_align_right_offset(),
                    TEXTDRAW_ALIGN_RIGHT_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_style_offset(),
                    TEXTDRAW_STYLE_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_x_offset(),
                    TEXTDRAW_X_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_y_offset(),
                    TEXTDRAW_Y_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_model_id_offset(),
                    TEXTDRAW_MODEL_ID_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_rotation_offset(),
                    TEXTDRAW_ROTATION_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_zoom_offset(),
                    TEXTDRAW_ZOOM_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_model_colour1_offset(),
                    TEXTDRAW_MODEL_COLOUR1_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    rak_samp_fixture_r1_textdraw_model_colour2_offset(),
                    TEXTDRAW_MODEL_COLOUR2_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
            ];
            for (actual, expected) in textdraw_offsets {
                assert_eq!(actual, expected);
            }
            assert_eq!(
                rak_samp_fixture_r1_object_pool_not_empty_offset(),
                OBJECT_POOL_NOT_EMPTY_OFFSET
            );
            assert_eq!(rak_samp_fixture_r1_gangzone_size(), 0x18);
            assert_eq!(
                rak_samp_fixture_r1_gangzone_pool_not_empty_offset(),
                GANGZONE_POOL_NOT_EMPTY_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_game_cursor_mode_offset(),
                GAME_CURSOR_MODE_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_scoreboard_enabled_offset(),
                SCOREBOARD_ENABLED_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_dialog_active_offset(),
                DIALOG_ACTIVE_OFFSET
            );
            assert_eq!(rak_samp_fixture_r1_dialog_type_offset(), DIALOG_TYPE_OFFSET);
            assert_eq!(rak_samp_fixture_r1_dialog_id_offset(), DIALOG_ID_OFFSET);
            assert_eq!(
                rak_samp_fixture_r1_dialog_caption_offset(),
                DIALOG_CAPTION_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_dialog_server_side_offset(),
                DIALOG_SERVER_SIDE_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_input_enabled_offset(),
                INPUT_ENABLED_OFFSET
            );
        }
    }

    #[test]
    fn native_dialog_strings_are_terminated_only_after_copying() {
        assert_eq!(nul_terminated(b"dialog".to_vec()), b"dialog\0");
    }

    #[test]
    fn bounded_label_copy_accepts_the_full_r1_text_limit() {
        let mut text = vec![b'x'; MAX_TEXT_LABEL_TEXT_BYTES];
        text.push(0);
        assert_eq!(
            unsafe { bounded_c_string(text.as_ptr(), MAX_TEXT_LABEL_TEXT_BYTES + 1) },
            Some(vec![b'x'; MAX_TEXT_LABEL_TEXT_BYTES])
        );
        assert_eq!(
            unsafe {
                bounded_c_string(
                    text[..MAX_TEXT_LABEL_TEXT_BYTES].as_ptr(),
                    MAX_TEXT_LABEL_TEXT_BYTES,
                )
            },
            None
        );
    }

    #[test]
    fn textdraw_constructor_signatures_cover_the_r1_numeric_store_regions() {
        assert_eq!(
            &TEXTDRAW_CTOR_CORE_FIELDS_SIGNATURE[..18],
            [
                0x8B, 0x44, 0x24, 0x10, 0x8B, 0x48, 0x01, 0x89, 0x0A, 0x8B, 0x50, 0x05, 0x89, 0x96,
                0x67, 0x09, 0x00, 0x00,
            ]
        );
        assert_eq!(
            &TEXTDRAW_CTOR_CORE_FIELDS_SIGNATURE[128..],
            [
                0x8A, 0x08, 0xD0, 0xE9, 0x80, 0xE1, 0x01, 0x88, 0x8E, 0x85, 0x09, 0x00, 0x00, 0x8A,
                0x10, 0xC0, 0xEA, 0x02, 0x80, 0xE2, 0x01, 0x88, 0x96, 0x86, 0x09, 0x00, 0x00,
            ]
        );
        assert_eq!(
            &TEXTDRAW_CTOR_MODEL_FIELDS_SIGNATURE[..19],
            [
                0x0F, 0xB6, 0x48, 0x1F, 0x89, 0x8E, 0x87, 0x09, 0x00, 0x00, 0x8B, 0x50, 0x21, 0x89,
                0x96, 0x8B, 0x09, 0x00, 0x00,
            ]
        );
        assert_eq!(
            &TEXTDRAW_CTOR_MODEL_FIELDS_SIGNATURE[119..],
            [0x66, 0x89, 0x86, 0xBC, 0x09, 0x00, 0x00]
        );
    }

    #[test]
    fn dialog_show_signature_matches_the_fingerprinted_r1_target() {
        assert_eq!(
            DIALOG_SHOW_SIGNATURE,
            [
                0x83, 0xEC, 0x10, 0x53, 0x56, 0x57, 0x8B, 0x7C, 0x24, 0x20, 0x33, 0xDB, 0x3B, 0xFB,
                0x8B, 0xF1,
            ]
        );
    }

    #[test]
    fn dialog_and_input_state_signatures_match_the_fingerprinted_r1_targets() {
        assert_eq!(
            DIALOG_SHOW_ACTIVE_SIGNATURE,
            [
                0x83, 0xEC, 0x10, 0x53, 0x56, 0x57, 0x8B, 0x7C, 0x24, 0x20, 0x33, 0xDB, 0x3B, 0xFB,
                0x8B, 0xF1, 0x7D, 0x17, 0x39, 0x5E, 0x28, 0x0F,
            ]
        );
        assert_eq!(
            DIALOG_SHOW_CORE_FIELDS_SIGNATURE,
            [
                0x89, 0x7E, 0x30, 0x89, 0x46, 0x2C, 0x89, 0x8E, 0x81, 0x00, 0x00, 0x00, 0x8D, 0x56,
                0x40,
            ]
        );
        assert_eq!(
            INPUT_OPEN_SIGNATURE,
            [
                0x83, 0xEC, 0x10, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xE0, 0x14, 0x00, 0x00, 0x85, 0xC0,
                0x0F, 0x85,
            ]
        );
        assert_eq!(
            INPUT_CLOSE_SIGNATURE,
            [
                0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xE0, 0x14, 0x00, 0x00, 0x85, 0xC0, 0x74, 0x39, 0x8B,
                0x4E, 0x08,
            ]
        );
    }

    #[test]
    fn animation_table_signature_and_parser_match_the_fingerprinted_r1_data() {
        assert_eq!(
            ANIMATION_TABLE_SIGNATURE,
            [
                0x41, 0x49, 0x52, 0x50, 0x4F, 0x52, 0x54, 0x3A, 0x54, 0x48, 0x52, 0x57, 0x5F, 0x42,
                0x41, 0x52, 0x4C, 0x5F, 0x54, 0x48, 0x52, 0x57, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        );
        assert_eq!(
            parse_animation_entry(&ANIMATION_TABLE_SIGNATURE),
            Ok(crate::runtime::AnimationSnapshot {
                name: b"AIRPORT".to_vec(),
                file: b"THRW_BARL_THRW".to_vec(),
            })
        );
    }

    #[test]
    fn chat_add_entry_signature_matches_the_fingerprinted_r1_target() {
        assert_eq!(
            CHAT_ADD_ENTRY_SIGNATURE,
            [
                0x55, 0x56, 0x8B, 0xE9, 0x57, 0x8D, 0xBD, 0x32, 0x01, 0x00, 0x00, 0x8D, 0xB5, 0x2E,
                0x02, 0x00,
            ]
        );
    }

    #[test]
    fn chat_get_mode_signature_matches_the_fingerprinted_r1_target() {
        assert_eq!(CHAT_GET_MODE_SIGNATURE, [0x8B, 0x41, 0x08, 0xC3]);
    }

    #[test]
    fn scoreboard_and_cursor_signatures_match_the_fingerprinted_r1_targets() {
        assert_eq!(
            SCOREBOARD_CLOSE_SIGNATURE,
            [
                0x56, 0x8B, 0xF1, 0x83, 0x3E, 0x00, 0x74, 0x3C, 0x8B, 0x46, 0x34, 0x85, 0xC0, 0x74,
                0x35, 0xC6,
            ]
        );
        assert_eq!(
            SCOREBOARD_ENABLE_SIGNATURE,
            [
                0x56, 0x8B, 0xF1, 0x83, 0x3E, 0x00, 0x75, 0x43, 0x8B, 0x46, 0x34, 0x85, 0xC0, 0x74,
                0x3C, 0xC6,
            ]
        );
        assert_eq!(
            GAME_PROCESS_INPUT_ENABLING_SIGNATURE,
            [
                0x56, 0x8B, 0xF1, 0x8B, 0x46, 0x55, 0x57, 0x33, 0xFF, 0x3B, 0xC7, 0x0F, 0x85, 0x07,
                0x01, 0x00,
            ]
        );
    }

    #[test]
    fn death_window_signatures_match_the_fingerprinted_r1_targets() {
        assert_eq!(
            DEATH_WINDOW_ADD_MESSAGE_SIGNATURE,
            [0xE9, 0x1B, 0xFF, 0xFF, 0xFF]
        );
        assert_eq!(
            DEATH_WINDOW_ADD_ENTRY_SIGNATURE,
            [
                0x8B, 0xD1, 0xE8, 0x49, 0xF6, 0xFF, 0xFF, 0x8A, 0x44, 0x24, 0x14, 0x8B, 0x4C, 0x24,
                0x10, 0x88,
            ]
        );
    }

    #[test]
    fn net_game_state_signature_matches_the_fingerprinted_r1_target() {
        assert_eq!(
            NET_GAME_GET_STATE_SIGNATURE,
            [0x8B, 0x81, 0xBD, 0x03, 0x00, 0x00, 0xC3]
        );
    }

    #[test]
    fn player_directory_accessor_signatures_match_the_fingerprinted_r1_targets() {
        assert_eq!(
            NET_GAME_GET_PLAYER_POOL_SIGNATURE,
            [0x8B, 0x81, 0xCD, 0x03, 0x00, 0x00, 0x8B, 0x40, 0x18]
        );
        assert_eq!(
            PLAYER_POOL_IS_CONNECTED_SIGNATURE,
            [
                0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xEC, 0x03, 0x72, 0x05, 0x33, 0xC0, 0xC2,
                0x04, 0x00,
            ]
        );
        assert_eq!(
            PLAYER_POOL_GET_REMOTE_PLAYER_SIGNATURE,
            [
                0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xEC, 0x03, 0x77, 0x10, 0x0F, 0xB7, 0xC0,
                0x8B, 0x44,
            ]
        );
        assert_eq!(
            PLAYER_POOL_IS_NPC_SIGNATURE,
            [
                0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xEC, 0x03, 0x77, 0x0B, 0x0F, 0xB7, 0xC0,
                0x8B, 0x44,
            ]
        );
        assert_eq!(
            PLAYER_POOL_GET_NAME_SIGNATURE,
            [
                0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3B, 0x41, 0x04, 0x75, 0x12, 0x83, 0x79, 0x1E,
                0x10, 0x72,
            ]
        );
        assert_eq!(
            PLAYER_POOL_GET_SCORE_SIGNATURE,
            [
                0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xEC, 0x03, 0x77, 0x0B, 0x0F, 0xB7, 0xC0,
                0x8B, 0x44,
            ]
        );
        assert_eq!(
            PLAYER_POOL_GET_PING_SIGNATURE,
            [
                0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xEC, 0x03, 0x77, 0x0B, 0x0F, 0xB7, 0xC0,
                0x8B, 0x44,
            ]
        );
        assert_eq!(
            PLAYER_POOL_GET_COUNT_SIGNATURE,
            [
                0x8B, 0x54, 0x24, 0x04, 0x56, 0x33, 0xC0, 0x85, 0xD2, 0x57, 0x74, 0x71, 0x33, 0xD2,
                0x8B, 0xFF,
            ]
        );
        assert_eq!(
            PLAYER_POOL_UPDATE_LARGEST_ID_SIGNATURE,
            [
                0x56, 0x57, 0x33, 0xF6, 0xB8, 0x02, 0x00, 0x00, 0x00, 0x8D, 0x91, 0xE2, 0x0F, 0x00,
                0x00, 0x90,
            ]
        );
        assert_eq!(
            NET_GAME_GET_VEHICLE_POOL_SIGNATURE,
            [0x8B, 0x81, 0xCD, 0x03, 0x00, 0x00, 0x8B, 0x40, 0x1C, 0xC3]
        );
        assert_eq!(
            NET_GAME_RESET_LABEL_POOL_FIELDS_SIGNATURE,
            [
                0x51, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xCD, 0x03, 0x00, 0x00, 0x57, 0x8B, 0x78, 0x0C,
                0x85, 0xFF, 0x74, 0x10,
            ]
        );
        assert_eq!(
            NET_GAME_RESET_TEXTDRAW_POOL_FIELDS_SIGNATURE,
            [
                0x51, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xCD, 0x03, 0x00, 0x00, 0x57, 0x8B, 0x78, 0x10,
                0x85, 0xFF, 0x74, 0x10,
            ]
        );
        assert_eq!(
            NET_GAME_RESET_OBJECT_POOL_FIELDS_SIGNATURE,
            [
                0x51, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xCD, 0x03, 0x00, 0x00, 0x57, 0x8B, 0x78, 0x04,
                0x85, 0xFF, 0x74, 0x10,
            ]
        );
        assert_eq!(
            NET_GAME_RESET_GANGZONE_POOL_FIELDS_SIGNATURE,
            [
                0x51, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xCD, 0x03, 0x00, 0x00, 0x57, 0x8B, 0x78, 0x08,
                0x85, 0xFF, 0x74, 0x10,
            ]
        );
        assert_eq!(
            GANG_ZONE_POOL_CREATE_POOL_FIELDS_SIGNATURE,
            [
                0xC7, 0x04, 0xBE, 0x00, 0x00, 0x00, 0x00, 0xC7, 0x84, 0xBE, 0x00, 0x10, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]
        );
        assert_eq!(
            GANG_ZONE_POOL_CREATE_RECORD_FIELDS_SIGNATURE,
            [
                0x8B, 0x4C, 0x24, 0x10, 0x8B, 0x54, 0x24, 0x1C, 0x89, 0x08, 0x8B, 0x4C, 0x24, 0x18,
                0x89, 0x50, 0x04, 0x8B, 0x54, 0x24, 0x14, 0x89, 0x48, 0x08, 0x8B, 0x4C, 0x24, 0x20,
                0x89, 0x50, 0x0C, 0x89, 0x48, 0x10, 0x89, 0x48, 0x14,
            ]
        );
        assert_eq!(
            TEXT_LABEL_POOL_CREATE_TEXT_ALLOCATION_SIGNATURE,
            [
                0x8D, 0x48, 0x01, 0x8B, 0xFF, 0x8A, 0x10, 0x40, 0x84, 0xD2, 0x75, 0xF9, 0x2B, 0xC1,
                0x40, 0x50, 0x6A, 0x01,
            ]
        );
        assert_eq!(
            TEXT_LABEL_POOL_CREATE_TEXT_COPY_SIGNATURE,
            [
                0x8B, 0xD3, 0x83, 0xC4, 0x08, 0x6B, 0xD2, 0x1D, 0x8D, 0x34, 0x2A, 0x89, 0x06, 0x8B,
                0xCF, 0x8A, 0x11, 0x41, 0x88, 0x10, 0x40, 0x84, 0xD2, 0x75, 0xF6,
            ]
        );
        assert_eq!(
            TEXT_LABEL_POOL_CREATE_SCALAR_FIELDS_SIGNATURE,
            [
                0x89, 0x46, 0x04, 0x8B, 0x44, 0x24, 0x24, 0x89, 0x4E, 0x08, 0x8B, 0x4C, 0x24, 0x28,
                0x89, 0x56, 0x0C, 0x8A, 0x54, 0x24, 0x2C, 0x89, 0x46, 0x10, 0x66, 0x8B, 0x44, 0x24,
                0x30, 0x89, 0x4E, 0x14, 0x66, 0x8B, 0x4C, 0x24, 0x34, 0x88, 0x56, 0x18, 0x66, 0x89,
                0x46, 0x19, 0x66, 0x89, 0x4E, 0x1B,
            ]
        );
        assert_eq!(
            VEHICLE_POOL_DOES_EXIST_SIGNATURE,
            [
                0x66, 0x8B, 0x44, 0x24, 0x04, 0x66, 0x3D, 0xD0, 0x07, 0x72, 0x05, 0x33, 0xC0, 0xC2,
                0x04, 0x00, 0x0F, 0xB7, 0xC0, 0x8B, 0x84, 0x81, 0x74, 0x30, 0x00, 0x00, 0xC2, 0x04,
                0x00,
            ]
        );
        assert_eq!(
            REMOTE_PLAYER_GET_COLOUR_ARGB_SIGNATURE,
            [
                0x0F, 0xB7, 0x81, 0xAB, 0x00, 0x00, 0x00, 0x50, 0xE8, 0x63, 0xAB, 0x09, 0x00, 0xC1,
                0xE8, 0x08,
            ]
        );
    }

    #[test]
    fn unassigned_local_player_id_is_not_a_snapshot() {
        assert_eq!(assigned_player_id(u16::MAX), None);
        assert_eq!(assigned_player_id(42), Some(42));
    }
}
