//! Runtime snapshot conversion into fixed ABI output storage.

use crate::runtime::{
    AimSyncSnapshot, AnimationSnapshot, ChatEntrySnapshot, GangzoneSnapshot, InCarSyncSnapshot,
    LocalDialogResponseSnapshot, LocalDialogSnapshot, LocalPlayerSnapshot, OnFootSyncSnapshot,
    PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot, ServerInfoSnapshot,
    TextLabelSnapshot, TextdrawSnapshot, TrailerSyncSnapshot,
};
use sdk_abi::limits::{
    MAX_SAMP_CHAT_ENTRIES, MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES, MAX_SAMP_CHAT_ENTRY_TEXT_BYTES,
    MAX_SAMP_TEXT_LABEL_TEXT_BYTES, MAX_SAMP_TEXTDRAW_STRING_BYTES,
};
use sdk_abi::{
    SampClientSdkActiveDialogV1, SampClientSdkAimSyncV1, SampClientSdkAnimationV1,
    SampClientSdkChatEntryV1, SampClientSdkDialogResponseV1, SampClientSdkDialogSnapshotV1,
    SampClientSdkGangzoneV1, SampClientSdkInCarSyncV1, SampClientSdkLocalPlayerV1,
    SampClientSdkOnFootSyncV1, SampClientSdkPassengerSyncV1, SampClientSdkPlayerInfoV1,
    SampClientSdkRemotePlayerStateV1, SampClientSdkServerInfoV1, SampClientSdkTextDrawV1,
    SampClientSdkTextLabelV1, SampClientSdkTrailerSyncV1, Vector3,
};

pub(super) fn local_player_to_abi(
    snapshot: LocalPlayerSnapshot,
) -> Result<SampClientSdkLocalPlayerV1, ()> {
    let nickname_len = u16::try_from(snapshot.nickname.len()).map_err(|_| ())?;
    if snapshot.nickname.len() > 256 {
        return Err(());
    }
    let mut nickname = [0; 256];
    nickname[..snapshot.nickname.len()].copy_from_slice(&snapshot.nickname);
    Ok(SampClientSdkLocalPlayerV1 {
        id: snapshot.id,
        nickname_len,
        nickname,
        colour: snapshot.colour,
        spawned: u8::from(snapshot.spawned),
        special_action: snapshot.special_action,
        animation_id: snapshot.animation_id,
        health: snapshot.health,
        armour: snapshot.armour,
        position: Vector3 {
            x: snapshot.position.x,
            y: snapshot.position.y,
            z: snapshot.position.z,
        },
        velocity: Vector3 {
            x: snapshot.velocity.x,
            y: snapshot.velocity.y,
            z: snapshot.velocity.z,
        },
        has_vehicle: u8::from(snapshot.vehicle_id.is_some()),
        _reserved: 0,
        vehicle_id: snapshot.vehicle_id.unwrap_or_default(),
        score: snapshot.score,
        ping: snapshot.ping,
    })
}

pub(super) fn local_dialog_state_to_abi(
    snapshot: &LocalDialogSnapshot,
) -> Result<SampClientSdkActiveDialogV1, ()> {
    let title_len = u8::try_from(snapshot.title.len()).map_err(|_| ())?;
    if snapshot.title.len() > 65 || snapshot.title.contains(&0) {
        return Err(());
    }
    let mut title = [0; 65];
    title[..snapshot.title.len()].copy_from_slice(&snapshot.title);
    Ok(SampClientSdkActiveDialogV1 {
        active: 1,
        style: snapshot.style.as_raw() as u8,
        server_side: u8::from(snapshot.server_side),
        _reserved: 0,
        id: snapshot.id,
        title_len,
        title,
    })
}

pub(super) fn local_dialog_snapshot_to_abi(
    snapshot: LocalDialogSnapshot,
) -> Result<SampClientSdkDialogSnapshotV1, ()> {
    let core = local_dialog_state_to_abi(&snapshot)?;
    let text_len = u16::try_from(snapshot.text.len()).map_err(|_| ())?;
    let listbox_item_count = u8::try_from(snapshot.listbox_items.len()).map_err(|_| ())?;
    let mut output = SampClientSdkDialogSnapshotV1::default();
    if snapshot.text.len() > output.text.len()
        || snapshot.text.contains(&0)
        || snapshot.listbox_items.len() > output.listbox_items.len()
    {
        return Err(());
    }

    output.active = core.active;
    output.style = core.style;
    output.server_side = core.server_side;
    output.id = core.id;
    output.title_len = core.title_len;
    output.title = core.title;
    output.text_len = text_len;
    output.text[..snapshot.text.len()].copy_from_slice(&snapshot.text);
    output.listbox_item_count = listbox_item_count;

    if let Some(editbox_text) = snapshot.editbox_text {
        let editbox_text_len = u8::try_from(editbox_text.len()).map_err(|_| ())?;
        if editbox_text.len() > output.editbox_text.len() || editbox_text.contains(&0) {
            return Err(());
        }
        output.has_editbox = 1;
        output.editbox_text_len = editbox_text_len;
        output.editbox_text[..editbox_text.len()].copy_from_slice(&editbox_text);
    }

    for (raw, item) in output.listbox_items.iter_mut().zip(snapshot.listbox_items) {
        let len = u8::try_from(item.len()).map_err(|_| ())?;
        if item.len() > raw.bytes.len() || item.contains(&0) {
            return Err(());
        }
        raw.len = len;
        raw.bytes[..item.len()].copy_from_slice(&item);
    }

    Ok(output)
}

pub(super) fn local_dialog_response_to_abi(
    response: LocalDialogResponseSnapshot,
) -> Result<SampClientSdkDialogResponseV1, ()> {
    let input_len = u8::try_from(response.input.len()).map_err(|_| ())?;
    let mut output = SampClientSdkDialogResponseV1::default();
    if response.input.len() > output.input.len() || response.input.contains(&0) {
        return Err(());
    }
    output.available = 1;
    output.button = response.button;
    output.input_len = input_len;
    output.dialog_id = response.dialog_id;
    output.list_item = response.list_item;
    output.input[..response.input.len()].copy_from_slice(&response.input);
    Ok(output)
}

pub(super) fn player_info_to_abi(
    snapshot: PlayerInfoSnapshot,
) -> Result<SampClientSdkPlayerInfoV1, ()> {
    let nickname_len = u16::try_from(snapshot.nickname.len()).map_err(|_| ())?;
    if snapshot.nickname.is_empty()
        || snapshot.nickname.len() > 256
        || snapshot.nickname.contains(&0)
        || (snapshot.is_local && snapshot.is_npc)
    {
        return Err(());
    }
    let mut nickname = [0; 256];
    nickname[..snapshot.nickname.len()].copy_from_slice(&snapshot.nickname);
    Ok(SampClientSdkPlayerInfoV1 {
        exists: 1,
        is_local: u8::from(snapshot.is_local),
        is_npc: u8::from(snapshot.is_npc),
        _reserved: 0,
        id: snapshot.id,
        nickname_len,
        nickname,
        colour: snapshot.colour,
        score: snapshot.score,
        ping: snapshot.ping,
    })
}

pub(super) fn remote_player_state_to_abi(
    snapshot: RemotePlayerStateSnapshot,
) -> Result<SampClientSdkRemotePlayerStateV1, ()> {
    if !snapshot.health.is_finite() || !snapshot.armour.is_finite() {
        return Err(());
    }
    Ok(SampClientSdkRemotePlayerStateV1 {
        exists: 1,
        special_action: snapshot.special_action,
        _reserved: 0,
        id: snapshot.id,
        animation_id: snapshot.animation_id,
        health: snapshot.health,
        armour: snapshot.armour,
    })
}

pub(super) fn onfoot_sync_to_abi(
    snapshot: OnFootSyncSnapshot,
) -> Result<SampClientSdkOnFootSyncV1, ()> {
    if !snapshot.position.x.is_finite()
        || !snapshot.position.y.is_finite()
        || !snapshot.position.z.is_finite()
        || !snapshot.quaternion.iter().all(|value| value.is_finite())
        || !snapshot.speed.x.is_finite()
        || !snapshot.speed.y.is_finite()
        || !snapshot.speed.z.is_finite()
        || !snapshot.surfing_offset.x.is_finite()
        || !snapshot.surfing_offset.y.is_finite()
        || !snapshot.surfing_offset.z.is_finite()
    {
        return Err(());
    }
    Ok(SampClientSdkOnFootSyncV1 {
        exists: 1,
        health: snapshot.health,
        armour: snapshot.armour,
        weapon: snapshot.weapon,
        special_action: snapshot.special_action,
        _reserved: [0; 3],
        id: snapshot.id,
        controller_left_stick_x: snapshot.controller_left_stick_x,
        controller_left_stick_y: snapshot.controller_left_stick_y,
        controller_buttons: snapshot.controller_buttons,
        _reserved2: 0,
        position: Vector3 {
            x: snapshot.position.x,
            y: snapshot.position.y,
            z: snapshot.position.z,
        },
        quaternion: snapshot.quaternion,
        speed: Vector3 {
            x: snapshot.speed.x,
            y: snapshot.speed.y,
            z: snapshot.speed.z,
        },
        surfing_offset: Vector3 {
            x: snapshot.surfing_offset.x,
            y: snapshot.surfing_offset.y,
            z: snapshot.surfing_offset.z,
        },
        surfing_vehicle_id: snapshot.surfing_vehicle_id,
        _reserved3: 0,
        animation: snapshot.animation,
    })
}

pub(super) fn vehicle_sync_to_abi(
    snapshot: InCarSyncSnapshot,
) -> Result<SampClientSdkInCarSyncV1, ()> {
    if !snapshot.quaternion.iter().all(|value| value.is_finite())
        || !snapshot.position.x.is_finite()
        || !snapshot.position.y.is_finite()
        || !snapshot.position.z.is_finite()
        || !snapshot.speed.x.is_finite()
        || !snapshot.speed.y.is_finite()
        || !snapshot.speed.z.is_finite()
        || !snapshot.vehicle_health.is_finite()
    {
        return Err(());
    }
    Ok(SampClientSdkInCarSyncV1 {
        exists: 1,
        driver_health: snapshot.driver_health,
        driver_armour: snapshot.driver_armour,
        weapon: snapshot.weapon,
        siren: u8::from(snapshot.siren),
        landing_gear: u8::from(snapshot.landing_gear),
        _reserved: [0; 2],
        id: snapshot.id,
        vehicle_id: snapshot.vehicle_id,
        controller_left_stick_x: snapshot.controller_left_stick_x,
        controller_left_stick_y: snapshot.controller_left_stick_y,
        controller_buttons: snapshot.controller_buttons,
        _reserved2: 0,
        quaternion: snapshot.quaternion,
        position: Vector3 {
            x: snapshot.position.x,
            y: snapshot.position.y,
            z: snapshot.position.z,
        },
        speed: Vector3 {
            x: snapshot.speed.x,
            y: snapshot.speed.y,
            z: snapshot.speed.z,
        },
        vehicle_health: snapshot.vehicle_health,
        trailer_id: snapshot.trailer_id,
        vehicle_specific: snapshot.vehicle_specific,
    })
}

pub(super) fn passenger_sync_to_abi(
    snapshot: PassengerSyncSnapshot,
) -> Result<SampClientSdkPassengerSyncV1, ()> {
    if !snapshot.position.x.is_finite()
        || !snapshot.position.y.is_finite()
        || !snapshot.position.z.is_finite()
    {
        return Err(());
    }
    Ok(SampClientSdkPassengerSyncV1 {
        exists: 1,
        seat_id: snapshot.seat_id,
        weapon: snapshot.weapon,
        health: snapshot.health,
        armour: snapshot.armour,
        _reserved: [0; 3],
        id: snapshot.id,
        vehicle_id: snapshot.vehicle_id,
        controller_left_stick_x: snapshot.controller_left_stick_x,
        controller_left_stick_y: snapshot.controller_left_stick_y,
        controller_buttons: snapshot.controller_buttons,
        _reserved2: 0,
        position: Vector3 {
            x: snapshot.position.x,
            y: snapshot.position.y,
            z: snapshot.position.z,
        },
    })
}

pub(super) fn trailer_sync_to_abi(
    snapshot: TrailerSyncSnapshot,
) -> Result<SampClientSdkTrailerSyncV1, ()> {
    if !snapshot.position.x.is_finite()
        || !snapshot.position.y.is_finite()
        || !snapshot.position.z.is_finite()
        || !snapshot.quaternion.iter().all(|value| value.is_finite())
        || !snapshot.speed.x.is_finite()
        || !snapshot.speed.y.is_finite()
        || !snapshot.speed.z.is_finite()
        || !snapshot.turn_speed.x.is_finite()
        || !snapshot.turn_speed.y.is_finite()
        || !snapshot.turn_speed.z.is_finite()
    {
        return Err(());
    }
    Ok(SampClientSdkTrailerSyncV1 {
        exists: 1,
        _reserved: [0; 3],
        id: snapshot.id,
        trailer_id: snapshot.trailer_id,
        position: Vector3 {
            x: snapshot.position.x,
            y: snapshot.position.y,
            z: snapshot.position.z,
        },
        quaternion: snapshot.quaternion,
        speed: Vector3 {
            x: snapshot.speed.x,
            y: snapshot.speed.y,
            z: snapshot.speed.z,
        },
        turn_speed: Vector3 {
            x: snapshot.turn_speed.x,
            y: snapshot.turn_speed.y,
            z: snapshot.turn_speed.z,
        },
    })
}

pub(super) fn aim_sync_to_abi(snapshot: AimSyncSnapshot) -> Result<SampClientSdkAimSyncV1, ()> {
    if !snapshot.aim_first.x.is_finite()
        || !snapshot.aim_first.y.is_finite()
        || !snapshot.aim_first.z.is_finite()
        || !snapshot.aim_position.x.is_finite()
        || !snapshot.aim_position.y.is_finite()
        || !snapshot.aim_position.z.is_finite()
        || !snapshot.aim_z.is_finite()
    {
        return Err(());
    }
    Ok(SampClientSdkAimSyncV1 {
        exists: 1,
        camera_mode: snapshot.camera_mode,
        zoom_and_weapon_state: snapshot.zoom_and_weapon_state,
        aspect_ratio: snapshot.aspect_ratio,
        id: snapshot.id,
        _reserved: 0,
        aim_first: Vector3 {
            x: snapshot.aim_first.x,
            y: snapshot.aim_first.y,
            z: snapshot.aim_first.z,
        },
        aim_position: Vector3 {
            x: snapshot.aim_position.x,
            y: snapshot.aim_position.y,
            z: snapshot.aim_position.z,
        },
        aim_z: snapshot.aim_z,
    })
}

pub(super) fn gangzone_to_abi(snapshot: GangzoneSnapshot) -> Result<SampClientSdkGangzoneV1, ()> {
    if !snapshot.left.is_finite()
        || !snapshot.bottom.is_finite()
        || !snapshot.right.is_finite()
        || !snapshot.top.is_finite()
    {
        return Err(());
    }
    Ok(SampClientSdkGangzoneV1 {
        exists: 1,
        _reserved: [0; 3],
        id: snapshot.id,
        _reserved2: 0,
        left: snapshot.left,
        bottom: snapshot.bottom,
        right: snapshot.right,
        top: snapshot.top,
        colour: snapshot.colour,
        alternate_colour: snapshot.alternate_colour,
    })
}

pub(super) fn text_label_to_abi(
    snapshot: TextLabelSnapshot,
) -> Result<SampClientSdkTextLabelV1, ()> {
    let text_len = u16::try_from(snapshot.text.len()).map_err(|_| ())?;
    if snapshot.text.len() > MAX_SAMP_TEXT_LABEL_TEXT_BYTES
        || snapshot.text.contains(&0)
        || !snapshot.position.x.is_finite()
        || !snapshot.position.y.is_finite()
        || !snapshot.position.z.is_finite()
        || !snapshot.draw_distance.is_finite()
    {
        return Err(());
    }
    let mut text = [0; MAX_SAMP_TEXT_LABEL_TEXT_BYTES];
    text[..snapshot.text.len()].copy_from_slice(&snapshot.text);
    Ok(SampClientSdkTextLabelV1 {
        exists: 1,
        behind_walls: u8::from(snapshot.behind_walls),
        _reserved: [0; 2],
        id: snapshot.id,
        attached_player_id: snapshot.attached_player_id.unwrap_or(u16::MAX),
        attached_vehicle_id: snapshot.attached_vehicle_id.unwrap_or(u16::MAX),
        _reserved2: 0,
        colour: snapshot.colour,
        position: Vector3 {
            x: snapshot.position.x,
            y: snapshot.position.y,
            z: snapshot.position.z,
        },
        draw_distance: snapshot.draw_distance,
        text_len,
        _reserved3: [0; 2],
        text,
    })
}

pub(super) fn textdraw_to_abi(snapshot: TextdrawSnapshot) -> Result<SampClientSdkTextDrawV1, ()> {
    if !snapshot.letter_width.is_finite()
        || !snapshot.letter_height.is_finite()
        || !snapshot.x.is_finite()
        || !snapshot.y.is_finite()
        || !snapshot.box_width.is_finite()
        || !snapshot.box_height.is_finite()
        || !snapshot.rotation.x.is_finite()
        || !snapshot.rotation.y.is_finite()
        || !snapshot.rotation.z.is_finite()
        || !snapshot.zoom.is_finite()
    {
        return Err(());
    }
    if snapshot.text.len() > MAX_SAMP_TEXTDRAW_STRING_BYTES || snapshot.text.contains(&0) {
        return Err(());
    }
    let mut text = [0; MAX_SAMP_TEXTDRAW_STRING_BYTES];
    text[..snapshot.text.len()].copy_from_slice(&snapshot.text);
    Ok(SampClientSdkTextDrawV1 {
        exists: 1,
        proportional: u8::from(snapshot.proportional),
        align_left: u8::from(snapshot.align_left),
        align_center: u8::from(snapshot.align_center),
        align_right: u8::from(snapshot.align_right),
        box_enabled: u8::from(snapshot.box_enabled),
        _reserved: [0; 2],
        pool_index: snapshot.pool_index,
        shadow: snapshot.shadow,
        outline: snapshot.outline,
        letter_width: snapshot.letter_width,
        letter_height: snapshot.letter_height,
        letter_colour: snapshot.letter_colour,
        x: snapshot.x,
        y: snapshot.y,
        background_colour: snapshot.background_colour,
        style: snapshot.style,
        box_width: snapshot.box_width,
        box_height: snapshot.box_height,
        box_colour: snapshot.box_colour,
        model_id: snapshot.model_id,
        _reserved2: 0,
        rotation: Vector3 {
            x: snapshot.rotation.x,
            y: snapshot.rotation.y,
            z: snapshot.rotation.z,
        },
        zoom: snapshot.zoom,
        model_colour1: snapshot.model_colour1,
        model_colour2: snapshot.model_colour2,
        text_len: snapshot.text.len() as u16,
        _reserved3: [0; 2],
        text,
    })
}

pub(super) fn chat_entry_to_abi(
    snapshot: ChatEntrySnapshot,
) -> Result<SampClientSdkChatEntryV1, ()> {
    if snapshot.id >= MAX_SAMP_CHAT_ENTRIES
        || snapshot.text.len() > MAX_SAMP_CHAT_ENTRY_TEXT_BYTES
        || snapshot.prefix.len() > MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES
        || snapshot.text.contains(&0)
        || snapshot.prefix.contains(&0)
    {
        return Err(());
    }
    let mut text = [0; MAX_SAMP_CHAT_ENTRY_TEXT_BYTES];
    text[..snapshot.text.len()].copy_from_slice(&snapshot.text);
    let mut prefix = [0; MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES];
    prefix[..snapshot.prefix.len()].copy_from_slice(&snapshot.prefix);
    Ok(SampClientSdkChatEntryV1 {
        id: snapshot.id,
        text_len: snapshot.text.len() as u8,
        prefix_len: snapshot.prefix.len() as u8,
        text_colour: snapshot.text_colour,
        prefix_colour: snapshot.prefix_colour,
        text,
        prefix,
    })
}

pub(super) fn server_info_to_abi(
    snapshot: ServerInfoSnapshot,
) -> Result<SampClientSdkServerInfoV1, ()> {
    let address_len = u16::try_from(snapshot.address.len()).map_err(|_| ())?;
    let hostname_len = u16::try_from(snapshot.hostname.len()).map_err(|_| ())?;
    if snapshot.address.is_empty()
        || snapshot.port == 0
        || snapshot.address.len() > 257
        || snapshot.hostname.len() > 257
    {
        return Err(());
    }
    let mut address = [0; 257];
    address[..snapshot.address.len()].copy_from_slice(&snapshot.address);
    let mut hostname = [0; 257];
    hostname[..snapshot.hostname.len()].copy_from_slice(&snapshot.hostname);
    Ok(SampClientSdkServerInfoV1 {
        address_len,
        hostname_len,
        address,
        hostname,
        port: snapshot.port,
    })
}

pub(super) fn animation_to_abi(
    snapshot: AnimationSnapshot,
) -> Result<SampClientSdkAnimationV1, ()> {
    let name_len = u8::try_from(snapshot.name.len()).map_err(|_| ())?;
    let file_len = u8::try_from(snapshot.file.len()).map_err(|_| ())?;
    if snapshot.name.is_empty()
        || snapshot.file.is_empty()
        || snapshot.name.len() > 35
        || snapshot.file.len() > 35
        || snapshot.name.contains(&0)
        || snapshot.file.contains(&0)
    {
        return Err(());
    }
    let mut name = [0; 36];
    name[..snapshot.name.len()].copy_from_slice(&snapshot.name);
    let mut file = [0; 36];
    file[..snapshot.file.len()].copy_from_slice(&snapshot.file);
    Ok(SampClientSdkAnimationV1 {
        name_len,
        file_len,
        name,
        file,
    })
}
