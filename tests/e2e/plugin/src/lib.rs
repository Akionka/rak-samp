//! Independently loaded plugin used by the ASI ABI end-to-end fixture.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_e2e_plugin supports only 32-bit Windows x86 targets");

use rak_samp_plugin_api::{
    Gangzone, LocalAnimation, LocalChatDisplayMode, LocalChatMessage, LocalChatMessageStyle,
    LocalCursorMode, LocalDeathMessage, LocalDialog, LocalDialogState, LocalDialogStyle,
    PlayerInfo, RakSampDirection, RakSampHookAction, Subscription, TextLabel, raknet::BitStream,
    wait_for_default_host,
};
use std::{
    ffi::c_void,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering},
    },
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{HINSTANCE, TRUE},
    System::{LibraryLoader::DisableThreadLibraryCalls, SystemServices::DLL_PROCESS_ATTACH},
};
use windows_sys::core::BOOL;

const TEST_RPC_ID: u8 = 42;

static SUBSCRIPTION: Mutex<Option<Subscription>> = Mutex::new(None);
static READY: AtomicBool = AtomicBool::new(false);
static STOP: AtomicBool = AtomicBool::new(false);
static CALLBACKS: AtomicU32 = AtomicU32::new(0);
static DIALOG_RESULT: AtomicU32 = AtomicU32::new(u32::MAX);
static LOCAL_CHAT_RESULT: AtomicU32 = AtomicU32::new(u32::MAX);
static LOCAL_DEATH_RESULT: AtomicU32 = AtomicU32::new(u32::MAX);
static LOCAL_PLAYER_ID: AtomicU32 = AtomicU32::new(u32::MAX);
static SAMP_GAME_STATE: AtomicI32 = AtomicI32::new(i32::MIN);
static LOCAL_CHAT_DISPLAY_MODE: AtomicI32 = AtomicI32::new(i32::MIN);
static LOCAL_CURSOR_MODE: AtomicI32 = AtomicI32::new(i32::MIN);
static LOCAL_SCOREBOARD_OPEN: AtomicI32 = AtomicI32::new(i32::MIN);
static LOCAL_DIALOG_ACTIVE: AtomicI32 = AtomicI32::new(i32::MIN);
static LOCAL_CHAT_INPUT_ACTIVE: AtomicI32 = AtomicI32::new(i32::MIN);
static LOCAL_ANIMATION_ID: AtomicI32 = AtomicI32::new(i32::MIN);
static PLAYER_INFO_ID: AtomicI32 = AtomicI32::new(i32::MIN);
static PLAYER_COUNT: AtomicI32 = AtomicI32::new(i32::MIN);
static PLAYER_MAX_ID: AtomicI32 = AtomicI32::new(i32::MIN);
static VEHICLE_EXISTS: AtomicI32 = AtomicI32::new(i32::MIN);
static ACTIVE_DIALOG_STATE: AtomicI32 = AtomicI32::new(i32::MIN);
static TEXT_LABEL_EXISTS: AtomicI32 = AtomicI32::new(i32::MIN);
static TEXTDRAW_EXISTS: AtomicI32 = AtomicI32::new(i32::MIN);
static OBJECT_EXISTS: AtomicI32 = AtomicI32::new(i32::MIN);
static GANGZONE_ID: AtomicI32 = AtomicI32::new(i32::MIN);
static TEXT_LABEL_ID: AtomicI32 = AtomicI32::new(i32::MIN);
static SAMP_VERSION: AtomicU32 = AtomicU32::new(u32::MAX);
static SERVER_PORT: AtomicU32 = AtomicU32::new(u32::MAX);
static DECODE_RESULT: AtomicU32 = AtomicU32::new(u32::MAX);

#[unsafe(no_mangle)]
/// Windows invokes this with loader-owned arguments while the plugin module is loaded.
///
/// # Safety
///
/// It must only be called by the Windows loader with a valid module handle and reason code.
pub unsafe extern "system" fn DllMain(
    instance: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        unsafe { DisableThreadLibraryCalls(instance) };
        let _ = std::thread::Builder::new()
            .name("rak-samp-e2e-plugin".into())
            .spawn(initialize);
    }
    TRUE
}

fn initialize() {
    let Ok(api) = wait_for_default_host(Duration::from_secs(5)) else {
        return;
    };
    if STOP.load(Ordering::Acquire) {
        return;
    }
    let dialog_result = api.show_local_dialog(LocalDialog {
        id: 0x7000,
        style: LocalDialogStyle::MessageBox,
        title: b"e2e",
        text: b"direct local dialog",
        button1: b"ok",
        button2: b"",
    });
    DIALOG_RESULT.store(dialog_result as u32, Ordering::Release);
    let local_chat_result = api.show_local_chat_message(LocalChatMessage {
        style: LocalChatMessageStyle::Debug,
        text: b"e2e local chat",
        prefix: b"[e2e]",
        text_colour: 0xFF_A9_C4_E4,
        prefix_colour: u32::MAX,
    });
    LOCAL_CHAT_RESULT.store(local_chat_result as u32, Ordering::Release);
    let local_death_result = api.show_local_death_message(LocalDeathMessage {
        killer: b"killer",
        victim: b"victim",
        killer_colour: 0xFFFF_0000,
        victim_colour: 0xFF00_FF00,
        weapon: 24,
    });
    LOCAL_DEATH_RESULT.store(local_death_result as u32, Ordering::Release);
    if let Ok(local_player) = api.local_player() {
        LOCAL_PLAYER_ID.store(u32::from(local_player.id), Ordering::Release);
    }
    if let Ok(game_state) = api.samp_game_state() {
        SAMP_GAME_STATE.store(game_state, Ordering::Release);
    }
    if api.local_chat_display_mode() == Ok(LocalChatDisplayMode::Normal)
        && api.is_local_chat_visible() == Ok(true)
    {
        LOCAL_CHAT_DISPLAY_MODE.store(2, Ordering::Release);
    }
    if api.local_cursor_mode() == Ok(LocalCursorMode::LockCamera)
        && api.is_local_cursor_active() == Ok(true)
    {
        LOCAL_CURSOR_MODE.store(3, Ordering::Release);
    }
    if api.is_local_scoreboard_open() == Ok(false) {
        LOCAL_SCOREBOARD_OPEN.store(0, Ordering::Release);
    }
    if api.is_local_dialog_active() == Ok(false) {
        LOCAL_DIALOG_ACTIVE.store(0, Ordering::Release);
    }
    if api.is_local_chat_input_active() == Ok(false) {
        LOCAL_CHAT_INPUT_ACTIVE.store(0, Ordering::Release);
    }
    if api.local_animation(0)
        == Ok(LocalAnimation {
            name: b"AIRPORT".to_vec(),
            file: b"THRW_BARL_THRW".to_vec(),
        })
        && api.local_animation_id(b"AIRPORT", b"THRW_BARL_THRW") == Ok(Some(0))
    {
        LOCAL_ANIMATION_ID.store(0, Ordering::Release);
    }
    if api.player_info(7)
        == Ok(Some(PlayerInfo {
            id: 7,
            nickname: b"bot".to_vec(),
            is_local: false,
            is_npc: true,
            colour: 0xFF44_5566,
            score: 12,
            ping: 34,
        }))
        && api.is_player_connected(7) == Ok(true)
        && api.is_player_connected(8) == Ok(false)
    {
        PLAYER_INFO_ID.store(7, Ordering::Release);
    }
    if api.player_count(true) == Ok(2) && api.player_count(false) == Ok(1) {
        PLAYER_COUNT.store(2, Ordering::Release);
    }
    if api.player_max_id() == Ok(77) {
        PLAYER_MAX_ID.store(77, Ordering::Release);
    }
    if api.is_vehicle_defined(7) == Ok(true) && api.is_vehicle_defined(8) == Ok(false) {
        VEHICLE_EXISTS.store(7, Ordering::Release);
    }
    if api.active_local_dialog()
        == Ok(Some(LocalDialogState {
            id: 0x7000,
            style: LocalDialogStyle::MessageBox,
            title: b"e2e".to_vec(),
            server_side: false,
        }))
    {
        ACTIVE_DIALOG_STATE.store(1, Ordering::Release);
    }
    if api.is_text_label_defined(7) == Ok(true) && api.is_text_label_defined(8) == Ok(false) {
        TEXT_LABEL_EXISTS.store(7, Ordering::Release);
    }
    if api.is_textdraw_defined(7) == Ok(true) && api.is_textdraw_defined(8) == Ok(false) {
        TEXTDRAW_EXISTS.store(7, Ordering::Release);
    }
    if api.is_object_defined(7) == Ok(true) && api.is_object_defined(8) == Ok(false) {
        OBJECT_EXISTS.store(7, Ordering::Release);
    }
    if api.gangzone(7)
        == Ok(Some(Gangzone {
            id: 7,
            left: -1.0,
            bottom: -2.0,
            right: 3.0,
            top: 4.0,
            colour: 0xFF11_2233,
            alternate_colour: 0xFF44_5566,
        }))
        && api.gangzone(8) == Ok(None)
    {
        GANGZONE_ID.store(7, Ordering::Release);
    }
    if api.text_label(7)
        == Ok(Some(TextLabel {
            id: 7,
            text: b"e2e".to_vec(),
            colour: 0xFF11_2233,
            position: rak_samp_plugin_api::Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            draw_distance: 25.0,
            behind_walls: true,
            attached_player_id: Some(8),
            attached_vehicle_id: None,
        }))
        && api.text_label(8) == Ok(None)
    {
        TEXT_LABEL_ID.store(7, Ordering::Release);
    }
    if let Ok(version) = api.samp_version() {
        SAMP_VERSION.store(version as u32, Ordering::Release);
    }
    if let Ok(server) = api.server_info()
        && server.address == b"127.0.0.1"
        && server.hostname == b"e2e"
    {
        SERVER_PORT.store(u32::from(server.port), Ordering::Release);
    }
    let mut compressed = match BitStream::from_bits(vec![0b1010_0000], 3) {
        Ok(value) => value,
        Err(_) => return,
    };
    if let Ok(decoded) = api.decode_string(&mut compressed)
        && decoded == b"e2e"
        && compressed.read_offset_bits() == 3
    {
        DECODE_RESULT.store(1, Ordering::Release);
    }
    let subscription = api.on_rpc_id(RakSampDirection::Incoming, TEST_RPC_ID, |_| {
        CALLBACKS.fetch_add(1, Ordering::AcqRel);
        RakSampHookAction::Continue
    });
    let Ok(subscription) = subscription else {
        return;
    };
    if STOP.load(Ordering::Acquire) {
        let _ = subscription.unregister_and_wait();
        return;
    }
    let mut slot = SUBSCRIPTION
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if slot.is_some() {
        drop(slot);
        let _ = subscription.unregister_and_wait();
        return;
    }
    *slot = Some(subscription);
    READY.store(true, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_Ready() -> i32 {
    i32::from(READY.load(Ordering::Acquire))
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_CallbackCount() -> u32 {
    CALLBACKS.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_DialogResult() -> u32 {
    DIALOG_RESULT.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_LocalChatResult() -> u32 {
    LOCAL_CHAT_RESULT.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_LocalDeathResult() -> u32 {
    LOCAL_DEATH_RESULT.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_LocalPlayerId() -> u32 {
    LOCAL_PLAYER_ID.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_SampGameState() -> i32 {
    SAMP_GAME_STATE.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_LocalChatDisplayMode() -> i32 {
    LOCAL_CHAT_DISPLAY_MODE.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_LocalCursorMode() -> i32 {
    LOCAL_CURSOR_MODE.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_LocalScoreboardOpen() -> i32 {
    LOCAL_SCOREBOARD_OPEN.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_LocalDialogActive() -> i32 {
    LOCAL_DIALOG_ACTIVE.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_LocalChatInputActive() -> i32 {
    LOCAL_CHAT_INPUT_ACTIVE.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_LocalAnimationId() -> i32 {
    LOCAL_ANIMATION_ID.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_PlayerInfoId() -> i32 {
    PLAYER_INFO_ID.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_PlayerCount() -> i32 {
    PLAYER_COUNT.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_PlayerMaxId() -> i32 {
    PLAYER_MAX_ID.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_VehicleExists() -> i32 {
    VEHICLE_EXISTS.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_ActiveDialogState() -> i32 {
    ACTIVE_DIALOG_STATE.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_TextLabelExists() -> i32 {
    TEXT_LABEL_EXISTS.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_TextdrawExists() -> i32 {
    TEXTDRAW_EXISTS.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_ObjectExists() -> i32 {
    OBJECT_EXISTS.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_GangzoneId() -> i32 {
    GANGZONE_ID.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_TextLabelId() -> i32 {
    TEXT_LABEL_ID.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_SampVersion() -> u32 {
    SAMP_VERSION.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_ServerPort() -> u32 {
    SERVER_PORT.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_DecodeResult() -> u32 {
    DECODE_RESULT.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2ePlugin_Shutdown() -> i32 {
    STOP.store(true, Ordering::Release);
    READY.store(false, Ordering::Release);
    let subscription = SUBSCRIPTION
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take();
    let Some(subscription) = subscription else {
        return 1;
    };
    i32::from(subscription.unregister_and_wait().is_ok())
}
