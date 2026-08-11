use super::*;
use crate::SampVersion;
use crate::runtime::{
    ChatEntrySnapshot, LocalDialogSnapshot, LocalPlayerSnapshot, ServerInfoSnapshot,
    TextLabelSnapshot, TextdrawSnapshot,
};
use sdk_abi::{
    SampClientSdkActiveDialogV1, SampClientSdkAnimationV1, SampClientSdkCommandResultV1,
    SampClientSdkDialogSnapshotV1, SampClientSdkGangzoneV1, SampClientSdkLocalPlayerV1,
    SampClientSdkPlayerInfoV1, SampClientSdkServerInfoV1, SampClientSdkTextDrawV1,
    SampClientSdkTextLabelV1,
    limits::{MAX_SAMP_GANGZONES, MAX_SAMP_OBJECTS},
};
use std::sync::{Arc, OnceLock};

#[test]
fn initialized_runtime_slot_can_be_reentered_while_a_handle_is_alive() {
    let slot = OnceLock::new();
    slot.set(Arc::new(7_u8)).unwrap();

    let outer = clone_initialized(&slot).unwrap();
    let nested = clone_initialized(&slot).unwrap();

    assert_eq!((*outer, *nested), (7, 7));
    assert_eq!(Arc::strong_count(&outer), 3);
}

#[test]
fn direct_client_busy_maps_to_the_retryable_abi_result() {
    assert_eq!(
        direct_client_result(DirectClientError::Busy),
        SampClientSdkResult::Busy
    );
}

#[test]
fn direct_command_completion_writes_the_receipt_only_after_success() {
    let mut receipt = SampClientSdkCommandReceipt::default();
    assert_eq!(
        unsafe { finish_direct_command(&mut receipt, Ok(42)) },
        SampClientSdkResult::Ok
    );
    assert_eq!(receipt.id, 42);

    assert_eq!(
        unsafe { finish_direct_command(&mut receipt, Err(DirectClientError::Busy)) },
        SampClientSdkResult::Busy
    );
    assert_eq!(receipt.id, 42);
}

#[test]
fn dialog_snapshot_conversion_is_coherent_and_preserves_absence() {
    let raw = conversions::local_dialog_snapshot_to_abi(LocalDialogSnapshot {
        id: 7,
        style: LocalDialogStyle::MessageBox,
        title: b"fixture".to_vec(),
        server_side: false,
        selected_item: None,
        list_item_count: None,
        text: b"body".to_vec(),
        editbox_text: None,
        listbox_items: vec![vec![b'x'; u8::MAX as usize]],
    })
    .expect("bounded snapshot converts");

    assert_eq!(raw.active, 1);
    assert_eq!(raw.has_editbox, 0);
    assert_eq!(raw.editbox_text_len, 0);
    assert_eq!(raw.listbox_item_count, 1);
    assert_eq!(raw.listbox_items[0].len, u8::MAX);
    assert_eq!(raw.listbox_items[0].bytes, [b'x'; u8::MAX as usize]);
}

#[test]
fn dialog_snapshot_conversion_rejects_a_256_byte_list_item() {
    assert!(
        conversions::local_dialog_snapshot_to_abi(LocalDialogSnapshot {
            id: 7,
            style: LocalDialogStyle::List,
            title: b"fixture".to_vec(),
            server_side: false,
            selected_item: None,
            list_item_count: Some(1),
            text: Vec::new(),
            editbox_text: None,
            listbox_items: vec![vec![b'x'; usize::from(u8::MAX) + 1]],
        })
        .is_err()
    );
}

#[test]
fn direct_client_abi_is_not_ready_without_a_runtime() {
    let mut output = SampClientSdkLocalPlayerV1::default();
    assert_eq!(
        unsafe { players::local_player(&mut output) },
        SampClientSdkResult::NotReady
    );
    let mut game_state = 0;
    assert_eq!(
        unsafe { environment::samp_game_state(&mut game_state) },
        SampClientSdkResult::NotReady
    );
    let mut chat_display_mode = 0;
    assert_eq!(
        unsafe { local_state::local_chat_display_mode(&mut chat_display_mode) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { local_state::local_chat_display_mode(std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut cursor_mode = 0;
    assert_eq!(
        unsafe { local_state::local_cursor_mode(&mut cursor_mode) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { local_state::local_cursor_mode(std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut scoreboard_open = 0;
    assert_eq!(
        unsafe { local_state::local_scoreboard_open(&mut scoreboard_open) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { local_state::local_scoreboard_open(std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut dialog_active = 0;
    assert_eq!(
        unsafe { local_state::local_dialog_active(&mut dialog_active) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { local_state::local_dialog_active(std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut active_dialog = SampClientSdkActiveDialogV1::default();
    assert_eq!(
        unsafe { local_state::active_local_dialog(&mut active_dialog) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { local_state::active_local_dialog(std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut dialog_snapshot = SampClientSdkDialogSnapshotV1::default();
    assert_eq!(
        unsafe { dialog::local_dialog_snapshot(&mut dialog_snapshot) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { dialog::local_dialog_snapshot(std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut chat_input_active = 0;
    assert_eq!(
        unsafe { local_state::local_chat_input_active(&mut chat_input_active) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { local_state::local_chat_input_active(std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut animation = SampClientSdkAnimationV1::default();
    assert_eq!(
        unsafe { animations::local_animation(0, &mut animation) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { animations::local_animation(0, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut animation_id = 0;
    assert_eq!(
        unsafe {
            animations::local_animation_id(
                b"AIRPORT".as_ptr(),
                b"AIRPORT".len(),
                b"THRW_BARL_THRW".as_ptr(),
                b"THRW_BARL_THRW".len(),
                &mut animation_id,
            )
        },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe {
            animations::local_animation_id(
                std::ptr::null(),
                1,
                b"THRW_BARL_THRW".as_ptr(),
                b"THRW_BARL_THRW".len(),
                &mut animation_id,
            )
        },
        SampClientSdkResult::InvalidArgument
    );
    let mut player = SampClientSdkPlayerInfoV1::default();
    assert_eq!(
        unsafe { players::player_info(7, &mut player) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { players::player_info(MAX_SAMP_PLAYERS, &mut player) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { players::player_info(7, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut player_defined_output = 0;
    assert_eq!(
        unsafe { players::player_defined(7, &mut player_defined_output) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { players::player_defined(MAX_SAMP_PLAYERS, &mut player_defined_output) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { players::player_defined(7, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut player_paused_output = 0;
    assert_eq!(
        unsafe { players::player_paused(7, &mut player_paused_output) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { players::player_paused(MAX_SAMP_PLAYERS, &mut player_paused_output) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { players::player_paused(7, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut count = 0;
    assert_eq!(
        unsafe { players::player_count(1, &mut count) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { players::player_count(2, &mut count) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { players::player_count(1, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut max_id = 0;
    assert_eq!(
        unsafe { players::player_max_id(&mut max_id) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { players::player_max_id(std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut vehicle_exists_output = 0;
    assert_eq!(
        unsafe { pools::vehicle_exists(7, &mut vehicle_exists_output) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { pools::vehicle_exists(MAX_SAMP_VEHICLES, &mut vehicle_exists_output) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { pools::vehicle_exists(7, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut text_label_exists_output = 0;
    assert_eq!(
        unsafe { pools::text_label_exists(7, &mut text_label_exists_output) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { pools::text_label_exists(MAX_SAMP_TEXT_LABELS, &mut text_label_exists_output) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { pools::text_label_exists(7, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut text_label = SampClientSdkTextLabelV1::default();
    assert_eq!(
        unsafe { snapshots::text_label_info(7, &mut text_label) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { snapshots::text_label_info(MAX_SAMP_TEXT_LABELS, &mut text_label) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { snapshots::text_label_info(7, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut textdraw = SampClientSdkTextDrawV1::default();
    assert_eq!(
        unsafe { snapshots::textdraw_info(7, &mut textdraw) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { snapshots::textdraw_info(MAX_SAMP_TEXTDRAWS, &mut textdraw) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { snapshots::textdraw_info(7, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut textdraw_exists_output = 0;
    assert_eq!(
        unsafe { pools::textdraw_exists(7, &mut textdraw_exists_output) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { pools::textdraw_exists(MAX_SAMP_TEXTDRAWS, &mut textdraw_exists_output) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { pools::textdraw_exists(7, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut object_exists_output = 0;
    assert_eq!(
        unsafe { pools::object_exists(7, &mut object_exists_output) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { pools::object_exists(MAX_SAMP_OBJECTS, &mut object_exists_output) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { pools::object_exists(7, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut gangzone = SampClientSdkGangzoneV1::default();
    assert_eq!(
        unsafe { snapshots::gangzone_info(7, &mut gangzone) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { snapshots::gangzone_info(MAX_SAMP_GANGZONES, &mut gangzone) },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe { snapshots::gangzone_info(7, std::ptr::null_mut()) },
        SampClientSdkResult::InvalidArgument
    );
    let mut server = SampClientSdkServerInfoV1::default();
    assert_eq!(
        unsafe { environment::server_info(&mut server) },
        SampClientSdkResult::NotReady
    );
    let mut version = 0;
    assert_eq!(
        unsafe { environment::samp_version(&mut version) },
        SampClientSdkResult::NotReady
    );
    let mut decoded = [0; 1];
    let mut decoded_len = 0;
    let mut read_offset = 0;
    assert_eq!(
        unsafe {
            events::decode_string(
                std::ptr::null(),
                0,
                0,
                0,
                decoded.as_mut_ptr(),
                decoded.len(),
                &raw mut decoded_len,
                &raw mut read_offset,
            )
        },
        SampClientSdkResult::NotReady
    );
    let mut receipt = SampClientSdkCommandReceipt::default();
    assert_eq!(
        unsafe {
            dialog::submit_local_dialog(
                7,
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                0,
                &mut receipt,
            )
        },
        SampClientSdkResult::NotReady
    );
    let mut command_result = SampClientSdkCommandResultV1::default();
    let receipt = SampClientSdkCommandReceipt { id: 1 };
    assert_eq!(
        unsafe { commands::command_try_take(receipt, &mut command_result) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { commands::command_wait(receipt, 0, &mut command_result) },
        SampClientSdkResult::NotReady
    );
    assert_eq!(
        unsafe { commands::command_release(receipt) },
        SampClientSdkResult::NotReady
    );
}

#[test]
fn owned_string_decode_rejects_invalid_abi_metadata_before_runtime_access() {
    let mut decoded = [0; 1];
    let mut decoded_len = 0;
    let mut read_offset = 0;
    assert_eq!(
        unsafe {
            events::decode_string(
                std::ptr::null(),
                0,
                1,
                0,
                decoded.as_mut_ptr(),
                decoded.len(),
                &raw mut decoded_len,
                &raw mut read_offset,
            )
        },
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(
        unsafe {
            events::decode_string(
                std::ptr::null(),
                0,
                0,
                0,
                decoded.as_mut_ptr(),
                events::MAX_CODEC_OUTPUT_BYTES + 1,
                &raw mut decoded_len,
                &raw mut read_offset,
            )
        },
        SampClientSdkResult::PayloadTooLarge
    );
}

#[test]
fn client_version_uses_stable_abi_values() {
    assert_eq!(environment::samp_version_to_abi(SampVersion::R1), 1);
    assert_eq!(environment::samp_version_to_abi(SampVersion::R5_1), 5);
    assert_eq!(environment::samp_version_to_abi(SampVersion::Dl), 6);
}

#[test]
fn local_snapshot_conversion_uses_only_fixed_abi_storage() {
    let snapshot = LocalPlayerSnapshot {
        id: 5,
        nickname: b"player".to_vec(),
        colour: 0xAABB_CCDD,
        spawned: true,
        health: 75.0,
        armour: 25.0,
        position: crate::runtime::Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        velocity: crate::runtime::Vector3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
        special_action: 7,
        animation_id: 8,
        vehicle_id: Some(9),
        score: 10,
        ping: 11,
    };

    let raw = conversions::local_player_to_abi(snapshot).expect("fixture snapshot fits the ABI");
    assert_eq!(raw.nickname_len, 6);
    assert_eq!(&raw.nickname[..6], b"player");
    assert_eq!(raw.has_vehicle, 1);
    assert_eq!(raw.vehicle_id, 9);
    assert_eq!(
        raw.position,
        Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
}

#[test]
fn server_snapshot_conversion_uses_only_fixed_abi_storage() {
    let raw = conversions::server_info_to_abi(ServerInfoSnapshot {
        address: b"127.0.0.1".to_vec(),
        hostname: b"fixture".to_vec(),
        port: 7777,
    })
    .expect("fixture server snapshot fits the ABI");
    assert_eq!(raw.address_len, 9);
    assert_eq!(&raw.address[..9], b"127.0.0.1");
    assert_eq!(raw.hostname_len, 7);
    assert_eq!(&raw.hostname[..7], b"fixture");
    assert_eq!(raw.port, 7777);
}

#[test]
fn text_label_snapshot_conversion_uses_only_fixed_abi_storage() {
    let raw = conversions::text_label_to_abi(TextLabelSnapshot {
        id: 7,
        text: b"fixture label".to_vec(),
        colour: 0xFF11_2233,
        position: crate::runtime::Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        draw_distance: 25.0,
        behind_walls: true,
        attached_player_id: Some(8),
        attached_vehicle_id: None,
    })
    .expect("fixture text label fits the ABI");
    assert_eq!(raw.exists, 1);
    assert_eq!(raw.text_len, 13);
    assert_eq!(&raw.text[..13], b"fixture label");
    assert_eq!(raw.attached_player_id, 8);
    assert_eq!(raw.attached_vehicle_id, u16::MAX);
}

#[test]
fn textdraw_snapshot_conversion_uses_only_fixed_abi_storage() {
    let raw = conversions::textdraw_to_abi(TextdrawSnapshot {
        pool_index: 7,
        text: Vec::new(),
        letter_width: 1.0,
        letter_height: 2.0,
        letter_colour: 0xFF11_2233,
        x: 3.0,
        y: 4.0,
        shadow: 2,
        outline: 3,
        background_colour: 0xFF44_5566,
        style: 5,
        proportional: true,
        align_left: false,
        align_center: true,
        align_right: false,
        box_enabled: true,
        box_width: 6.0,
        box_height: 7.0,
        box_colour: 0xFF77_8899,
        model_id: 10,
        rotation: crate::runtime::Vector3 {
            x: 8.0,
            y: 9.0,
            z: 10.0,
        },
        zoom: 11.0,
        model_colour1: 12,
        model_colour2: 13,
    })
    .expect("fixture textdraw fits the ABI");
    assert_eq!(raw.exists, 1);
    assert_eq!(raw.pool_index, 7);
    assert_eq!(raw.align_center, 1);
    assert_eq!(raw.model_colour2, 13);
}

#[test]
fn chat_entry_snapshot_conversion_uses_only_fixed_abi_storage() {
    let raw = conversions::chat_entry_to_abi(ChatEntrySnapshot {
        id: 7,
        text: b"fixture".to_vec(),
        prefix: b"prefix".to_vec(),
        text_colour: 0xFF11_2233,
        prefix_colour: 0xFF44_5566,
    })
    .expect("fixture chat entry fits the ABI");
    assert_eq!(raw.id, 7);
    assert_eq!(raw.text_len, 7);
    assert_eq!(&raw.text[..7], b"fixture");
    assert_eq!(raw.prefix_len, 6);
    assert_eq!(&raw.prefix[..6], b"prefix");
}
