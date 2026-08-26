use super::types::*;
use crate::events::core::PayloadWriter;
use crate::{
    HostApi, SampClientSdkResult,
    events::{
        EncodedPayload, Event, EventError, MAX_ENCODED_STRING_BYTES, MAX_STRING32_BYTES, Rpc,
        Vector2, Vector3,
    },
};

/// The `onShowDialog` descriptor.
pub const SHOW_DIALOG: Rpc<ShowDialog> = Rpc::new_bits(61, decode_show_dialog, encode_show_dialog);
/// The `onSetVehiclePosition` descriptor.
pub const SET_VEHICLE_POSITION: Rpc<VehiclePosition> =
    Rpc::new(159, decode_vehicle_position, encode_vehicle_position);
/// The `onSetVehicleAngle` descriptor (`vehicle_id`, then angle).
pub const SET_VEHICLE_ANGLE: Rpc<VehicleAngle> =
    Rpc::new(160, decode_vehicle_angle, encode_vehicle_angle);
/// The `onSetVehicleHealth` descriptor.
pub const SET_VEHICLE_HEALTH: Rpc<VehicleHealth> =
    Rpc::new(147, decode_vehicle_health, encode_vehicle_health);
/// The `onResetPlayerMoney` descriptor.
pub const RESET_PLAYER_MONEY: Rpc<()> = Rpc::new(20, decode_empty, encode_empty);
/// The `onResetPlayerWeapons` descriptor.
pub const RESET_PLAYER_WEAPONS: Rpc<()> = Rpc::new(21, decode_empty, encode_empty);
/// The `onCancelEdit` descriptor.
pub const CANCEL_EDIT: Rpc<()> = Rpc::new(28, decode_empty, encode_empty);
/// The `onSetToggleClock` descriptor.
pub const SET_TOGGLE_CLOCK: Rpc<bool> = Rpc::new(30, decode_bool8, encode_bool8);
/// The `onSetPlayerDrunk` descriptor.
pub const SET_PLAYER_DRUNK: Rpc<i32> = Rpc::new(35, decode_i32, encode_i32);
/// The `onSetRaceCheckpoint` descriptor.
pub const SET_RACE_CHECKPOINT: Rpc<RaceCheckpoint> =
    Rpc::new(38, decode_race_checkpoint, encode_race_checkpoint);
/// The `onPlayAudioStream` descriptor.
pub const PLAY_AUDIO_STREAM: Rpc<AudioStream> =
    Rpc::new(41, decode_audio_stream, encode_audio_stream);
/// The `onSetObjectPosition` descriptor.
pub const SET_OBJECT_POSITION: Rpc<ObjectPosition> =
    Rpc::new(45, decode_object_position, encode_object_position);
/// The `onSetObjectRotation` descriptor.
pub const SET_OBJECT_ROTATION: Rpc<ObjectRotation> =
    Rpc::new(46, decode_object_rotation, encode_object_rotation);
/// The `onDestroyObject` descriptor.
pub const DESTROY_OBJECT: Rpc<u16> = Rpc::new(47, decode_u16, encode_u16);
/// The `onPlayerDeathNotification` descriptor.
pub const PLAYER_DEATH_NOTIFICATION: Rpc<PlayerDeathNotification> = Rpc::new(
    55,
    decode_player_death_notification,
    encode_player_death_notification,
);
/// The `onSetMapIcon` descriptor.
pub const SET_MAP_ICON: Rpc<MapIcon> = Rpc::new(56, decode_map_icon, encode_map_icon);
/// The `onRemoveVehicleComponent` descriptor.
pub const REMOVE_VEHICLE_COMPONENT: Rpc<VehicleComponent> =
    Rpc::new(57, decode_vehicle_component, encode_vehicle_component);
/// The `onRemove3DTextLabel` descriptor.
pub const REMOVE_3D_TEXT_LABEL: Rpc<u16> = Rpc::new(58, decode_u16, encode_u16);
/// The `onUpdateGlobalTimer` descriptor.
pub const UPDATE_GLOBAL_TIMER: Rpc<i32> = Rpc::new(60, decode_i32, encode_i32);
/// The `onDestroyPickup` descriptor.
pub const DESTROY_PICKUP: Rpc<i32> = Rpc::new(63, decode_i32, encode_i32);
/// The `onLinkVehicleToInterior` descriptor.
pub const LINK_VEHICLE_TO_INTERIOR: Rpc<VehicleInterior> =
    Rpc::new(65, decode_vehicle_interior, encode_vehicle_interior);
/// The `onSetPlayerColor` descriptor.
pub const SET_PLAYER_COLOR: Rpc<PlayerColor> =
    Rpc::new(72, decode_player_color, encode_player_color);
/// The `onRequestSpawnResponse` descriptor.
pub const REQUEST_SPAWN_RESPONSE: Rpc<bool> = Rpc::new(129, decode_bool8, encode_bool8);
/// The `onSetShopName` descriptor. The protocol field is exactly 32 bytes.
pub const SET_SHOP_NAME: Rpc<[u8; 32]> = Rpc::new(33, decode_fixed_string32, encode_fixed_string32);
/// The `onSetPlayerSkillLevel` descriptor.
pub const SET_PLAYER_SKILL_LEVEL: Rpc<PlayerSkill> =
    Rpc::new(34, decode_player_skill, encode_player_skill);
/// The `onRemoveBuilding` descriptor.
pub const REMOVE_BUILDING: Rpc<RemoveBuilding> =
    Rpc::new(43, decode_remove_building, encode_remove_building);
/// The `onAttachObjectToPlayer` descriptor.
pub const ATTACH_OBJECT_TO_PLAYER: Rpc<AttachObjectToPlayer> = Rpc::new(
    75,
    decode_attach_object_to_player,
    encode_attach_object_to_player,
);
/// The `onShowMenu` descriptor.
pub const SHOW_MENU: Rpc<u8> = Rpc::new(77, decode_u8, encode_u8);
/// The `onHideMenu` descriptor.
pub const HIDE_MENU: Rpc<u8> = Rpc::new(78, decode_u8, encode_u8);
/// The `onCreateExplosion` descriptor.
pub const CREATE_EXPLOSION: Rpc<Explosion> = Rpc::new(79, decode_explosion, encode_explosion);
/// The `onShowPlayerNameTag` descriptor.
pub const SHOW_PLAYER_NAME_TAG: Rpc<PlayerNameTag> =
    Rpc::new(80, decode_player_name_tag, encode_player_name_tag);
/// The `onAttachCameraToObject` descriptor.
pub const ATTACH_CAMERA_TO_OBJECT: Rpc<u16> = Rpc::new(81, decode_u16, encode_u16);
/// The `onGangZoneStopFlash` descriptor.
pub const GANG_ZONE_STOP_FLASH: Rpc<u16> = Rpc::new(85, decode_u16, encode_u16);
/// The `onClearPlayerAnimation` descriptor.
pub const CLEAR_PLAYER_ANIMATION: Rpc<u16> = Rpc::new(87, decode_u16, encode_u16);
/// The `onSetPlayerSpecialAction` descriptor.
pub const SET_PLAYER_SPECIAL_ACTION: Rpc<u8> = Rpc::new(88, decode_u8, encode_u8);
/// The `onSetPlayerFightingStyle` descriptor.
pub const SET_PLAYER_FIGHTING_STYLE: Rpc<PlayerFightingStyle> = Rpc::new(
    89,
    decode_player_fighting_style,
    encode_player_fighting_style,
);
/// The `onSetPlayerVelocity` descriptor.
pub const SET_PLAYER_VELOCITY: Rpc<Vector3> = Rpc::new(90, decode_vector3, encode_vector3);
/// The `onSetVehicleVelocity` descriptor.
pub const SET_VEHICLE_VELOCITY: Rpc<VehicleVelocity> =
    Rpc::new(91, decode_vehicle_velocity, encode_vehicle_velocity);
/// The `onCreatePickup` descriptor.
pub const CREATE_PICKUP: Rpc<Pickup> = Rpc::new(95, decode_pickup, encode_pickup);
/// The `onMoveObject` descriptor.
pub const MOVE_OBJECT: Rpc<MoveObject> = Rpc::new(99, decode_move_object, encode_move_object);
/// The `onTextDrawSetString` descriptor.
pub const TEXT_DRAW_SET_STRING: Rpc<TextDrawString> =
    Rpc::new(105, decode_text_draw_string, encode_text_draw_string);
/// The `onCreateGangZone` descriptor.
pub const CREATE_GANG_ZONE: Rpc<GangZone> = Rpc::new(108, decode_gang_zone, encode_gang_zone);
/// The `onGangZoneDestroy` descriptor.
pub const GANG_ZONE_DESTROY: Rpc<u16> = Rpc::new(120, decode_u16, encode_u16);
/// The `onGangZoneFlash` descriptor.
pub const GANG_ZONE_FLASH: Rpc<(u16, i32)> = Rpc::new(121, decode_u16_i32, encode_u16_i32);
/// The `onStopObject` descriptor.
pub const STOP_OBJECT: Rpc<u16> = Rpc::new(122, decode_u16, encode_u16);
/// The `onSetVehicleNumberPlate` descriptor.
pub const SET_VEHICLE_NUMBER_PLATE: Rpc<VehicleNumberPlate> = Rpc::new(
    123,
    decode_vehicle_number_plate,
    encode_vehicle_number_plate,
);
/// The `onSpectatePlayer` descriptor.
pub const SPECTATE_PLAYER: Rpc<Spectate> = Rpc::new(126, decode_spectate, encode_spectate);
/// The `onSpectateVehicle` descriptor.
pub const SPECTATE_VEHICLE: Rpc<Spectate> = Rpc::new(127, decode_spectate, encode_spectate);
/// The `onConnectionRejected` descriptor.
pub const CONNECTION_REJECTED: Rpc<u8> = Rpc::new(130, decode_u8, encode_u8);
/// The `onRemoveMapIcon` descriptor.
pub const REMOVE_MAP_ICON: Rpc<u8> = Rpc::new(144, decode_u8, encode_u8);
/// The `onSetWeaponAmmo` descriptor.
pub const SET_WEAPON_AMMO: Rpc<WeaponAmmo> = Rpc::new(145, decode_weapon_ammo, encode_weapon_ammo);
/// The `onSetGravity` descriptor.
pub const SET_GRAVITY: Rpc<f32> = Rpc::new(146, decode_f32, encode_f32);
/// The `onAttachTrailerToVehicle` descriptor.
pub const ATTACH_TRAILER_TO_VEHICLE: Rpc<TrailerAttachment> =
    Rpc::new(148, decode_trailer_attachment, encode_trailer_attachment);
/// The `onDetachTrailerFromVehicle` descriptor.
pub const DETACH_TRAILER_FROM_VEHICLE: Rpc<u16> = Rpc::new(149, decode_u16, encode_u16);
/// The `onSetCameraPosition` descriptor.
pub const SET_CAMERA_POSITION: Rpc<Vector3> = Rpc::new(157, decode_vector3, encode_vector3);
/// The `onSetCameraLookAt` descriptor.
pub const SET_CAMERA_LOOK_AT: Rpc<CameraLookAt> =
    Rpc::new(158, decode_camera_look_at, encode_camera_look_at);
/// The `onSetVehicleParams` descriptor.
pub const SET_VEHICLE_PARAMS: Rpc<VehicleParams> =
    Rpc::new(161, decode_vehicle_params, encode_vehicle_params);
/// The `onPlayerDeath` descriptor.
pub const PLAYER_DEATH: Rpc<u16> = Rpc::new(166, decode_u16, encode_u16);
/// The `onPlayerEnterVehicle` descriptor.
pub const PLAYER_ENTER_VEHICLE: Rpc<PlayerEnterVehicle> =
    Rpc::new(26, decode_player_enter_vehicle, encode_player_enter_vehicle);
/// The `onPlayerExitVehicle` descriptor.
pub const PLAYER_EXIT_VEHICLE: Rpc<PlayerExitVehicle> =
    Rpc::new(154, decode_player_exit_vehicle, encode_player_exit_vehicle);
/// The `onClientCheck` descriptor.
pub const CLIENT_CHECK: Rpc<ClientCheck> = Rpc::new(103, decode_client_check, encode_client_check);
/// The `onSetVehicleParamsEx` descriptor.
pub const SET_VEHICLE_PARAMS_EX: Rpc<VehicleParamsEx> =
    Rpc::new(24, decode_vehicle_params_ex, encode_vehicle_params_ex);
/// The `onVehicleTuningNotification` descriptor.
pub const VEHICLE_TUNING_NOTIFICATION: Rpc<VehicleTuningNotification> = Rpc::new(
    96,
    decode_vehicle_tuning_notification,
    encode_vehicle_tuning_notification,
);
/// The `onSetVehicleTires` descriptor.
pub const SET_VEHICLE_TIRES: Rpc<(u16, u8)> = Rpc::new(98, decode_u16_u8, encode_u16_u8);
/// The `onVehicleDamageStatusUpdate` descriptor.
pub const VEHICLE_DAMAGE_STATUS_UPDATE: Rpc<VehicleDamageStatus> = Rpc::new(
    106,
    decode_vehicle_damage_status,
    encode_vehicle_damage_status,
);
/// The `onToggleWidescreen` descriptor.
pub const TOGGLE_WIDESCREEN: Rpc<bool> = Rpc::new(111, decode_bool8, encode_bool8);
/// The `onDestroyActor` descriptor.
pub const DESTROY_ACTOR: Rpc<u16> = Rpc::new(172, decode_u16, encode_u16);
/// The `onDestroyWeaponPickup` descriptor.
pub const DESTROY_WEAPON_PICKUP: Rpc<u8> = Rpc::new(151, decode_u8, encode_u8);
/// The `onEditAttachedObject` descriptor.
pub const EDIT_ATTACHED_OBJECT: Rpc<i32> = Rpc::new(116, decode_i32, encode_i32);
/// The `onEnterSelectObject` descriptor.
pub const ENTER_SELECT_OBJECT: Rpc<()> = Rpc::new(27, decode_empty, encode_empty);
/// The `onServerStatisticsResponse` descriptor.
pub const SERVER_STATISTICS_RESPONSE: Rpc<()> = Rpc::new(102, decode_empty, encode_empty);
/// The `onSetPlayerDrunkVisuals` descriptor.
pub const SET_PLAYER_DRUNK_VISUALS: Rpc<i32> = Rpc::new(92, decode_i32, encode_i32);
/// The `onSetPlayerDrunkHandling` descriptor.
pub const SET_PLAYER_DRUNK_HANDLING: Rpc<i32> = Rpc::new(150, decode_i32, encode_i32);
/// The `onCreateActor` descriptor.
pub const CREATE_ACTOR: Rpc<Actor> = Rpc::new(171, decode_actor, encode_actor);
/// The `onClearActorAnimation` descriptor.
pub const CLEAR_ACTOR_ANIMATION: Rpc<u16> = Rpc::new(174, decode_u16, encode_u16);
/// The `onSetActorFacingAngle` descriptor.
pub const SET_ACTOR_FACING_ANGLE: Rpc<ActorAngle> =
    Rpc::new(175, decode_actor_angle, encode_actor_angle);
/// The `onSetActorPos` descriptor.
pub const SET_ACTOR_POSITION: Rpc<ActorPosition> =
    Rpc::new(176, decode_actor_position, encode_actor_position);
/// The `onSetActorHealth` descriptor.
pub const SET_ACTOR_HEALTH: Rpc<ActorHealth> =
    Rpc::new(178, decode_actor_health, encode_actor_health);
/// The `onSetPlayerObjectNoCameraCol` descriptor.
pub const SET_PLAYER_OBJECT_NO_CAMERA_COL: Rpc<u16> = Rpc::new(169, decode_u16, encode_u16);
/// The `onDisableCheckpoint` descriptor.
pub const DISABLE_CHECKPOINT: Rpc<()> = Rpc::new(37, decode_empty, encode_empty);
/// The `onDisableRaceCheckpoint` descriptor.
pub const DISABLE_RACE_CHECKPOINT: Rpc<()> = Rpc::new(39, decode_empty, encode_empty);
/// The `onGamemodeRestart` descriptor.
pub const GAMEMODE_RESTART: Rpc<()> = Rpc::new(40, decode_empty, encode_empty);
/// The `onStopAudioStream` descriptor.
pub const STOP_AUDIO_STREAM: Rpc<()> = Rpc::new(42, decode_empty, encode_empty);
/// The `onRemovePlayerFromVehicle` descriptor.
pub const REMOVE_PLAYER_FROM_VEHICLE: Rpc<()> = Rpc::new(71, decode_empty, encode_empty);
/// The `onForceClassSelection` descriptor.
pub const FORCE_CLASS_SELECTION: Rpc<()> = Rpc::new(74, decode_empty, encode_empty);
/// The `onSetCameraBehind` descriptor.
pub const SET_CAMERA_BEHIND: Rpc<()> = Rpc::new(162, decode_empty, encode_empty);

fn decode_show_dialog(event: &mut Event<'_>) -> Result<ShowDialog, EventError> {
    Ok(ShowDialog {
        dialog_id: event.read_u16()?,
        style: event.read_u8()?,
        title: event.read_string8()?,
        button1: event.read_string8()?,
        button2: event.read_string8()?,
        text: event.read_encoded_string(MAX_ENCODED_STRING_BYTES + 1)?,
    })
}

fn encode_show_dialog(api: HostApi, value: ShowDialog) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.dialog_id);
    writer.u8(value.style);
    writer.string8(&value.title)?;
    writer.string8(&value.button1)?;
    writer.string8(&value.button2)?;
    writer.encoded_string(api, &value.text)?;
    Ok(writer.finish_bits())
}

pub(super) fn decode_vector3(event: &mut Event<'_>) -> Result<Vector3, EventError> {
    Ok(Vector3 {
        x: event.read_f32()?,
        y: event.read_f32()?,
        z: event.read_f32()?,
    })
}

fn encode_vector3(value: Vector3) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.vector3(value);
    Ok(writer.finish())
}

fn decode_f32(event: &mut Event<'_>) -> Result<f32, EventError> {
    event.read_f32()
}

fn encode_f32(value: f32) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.f32(value);
    Ok(writer.finish())
}

pub(super) fn decode_bool8(event: &mut Event<'_>) -> Result<bool, EventError> {
    Ok(event.read_u8()? != 0)
}

fn encode_bool8(value: bool) -> Result<Vec<u8>, EventError> {
    Ok(vec![u8::from(value)])
}

fn decode_race_checkpoint(event: &mut Event<'_>) -> Result<RaceCheckpoint, EventError> {
    Ok(RaceCheckpoint {
        checkpoint_type: event.read_u8()?,
        position: decode_vector3(event)?,
        next_position: decode_vector3(event)?,
        size: event.read_f32()?,
    })
}

fn encode_race_checkpoint(value: RaceCheckpoint) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(value.checkpoint_type);
    writer.vector3(value.position);
    writer.vector3(value.next_position);
    writer.f32(value.size);
    Ok(writer.finish())
}

fn decode_audio_stream(event: &mut Event<'_>) -> Result<AudioStream, EventError> {
    Ok(AudioStream {
        url: event.read_string8()?,
        position: decode_vector3(event)?,
        radius: event.read_f32()?,
        use_position: decode_bool8(event)?,
    })
}

fn encode_audio_stream(value: AudioStream) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.string8(&value.url)?;
    writer.vector3(value.position);
    writer.f32(value.radius);
    writer.u8(u8::from(value.use_position));
    Ok(writer.finish())
}

fn decode_object_position(event: &mut Event<'_>) -> Result<ObjectPosition, EventError> {
    Ok(ObjectPosition {
        object_id: event.read_u16()?,
        position: decode_vector3(event)?,
    })
}

fn encode_object_position(value: ObjectPosition) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.object_id);
    writer.vector3(value.position);
    Ok(writer.finish())
}

fn decode_object_rotation(event: &mut Event<'_>) -> Result<ObjectRotation, EventError> {
    Ok(ObjectRotation {
        object_id: event.read_u16()?,
        rotation: decode_vector3(event)?,
    })
}

fn encode_object_rotation(value: ObjectRotation) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.object_id);
    writer.vector3(value.rotation);
    Ok(writer.finish())
}

fn decode_player_death_notification(
    event: &mut Event<'_>,
) -> Result<PlayerDeathNotification, EventError> {
    Ok(PlayerDeathNotification {
        killer_id: event.read_u16()?,
        killed_id: event.read_u16()?,
        reason: event.read_u8()?,
    })
}

fn encode_player_death_notification(value: PlayerDeathNotification) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.killer_id);
    writer.u16(value.killed_id);
    writer.u8(value.reason);
    Ok(writer.finish())
}

fn decode_map_icon(event: &mut Event<'_>) -> Result<MapIcon, EventError> {
    Ok(MapIcon {
        icon_id: event.read_u8()?,
        position: decode_vector3(event)?,
        icon_type: event.read_u8()?,
        color: decode_i32(event)?,
        style: event.read_u8()?,
    })
}

fn encode_map_icon(value: MapIcon) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(value.icon_id);
    writer.vector3(value.position);
    writer.u8(value.icon_type);
    writer.u32(value.color as u32);
    writer.u8(value.style);
    Ok(writer.finish())
}

fn decode_vehicle_component(event: &mut Event<'_>) -> Result<VehicleComponent, EventError> {
    Ok(VehicleComponent {
        vehicle_id: event.read_u16()?,
        component_id: event.read_u16()?,
    })
}

fn encode_vehicle_component(value: VehicleComponent) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u16(value.component_id);
    Ok(writer.finish())
}

fn decode_vehicle_interior(event: &mut Event<'_>) -> Result<VehicleInterior, EventError> {
    Ok(VehicleInterior {
        vehicle_id: event.read_u16()?,
        interior_id: event.read_u8()?,
    })
}

fn encode_vehicle_interior(value: VehicleInterior) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u8(value.interior_id);
    Ok(writer.finish())
}

fn decode_player_color(event: &mut Event<'_>) -> Result<PlayerColor, EventError> {
    Ok(PlayerColor {
        player_id: event.read_u16()?,
        color: decode_i32(event)?,
    })
}

fn encode_player_color(value: PlayerColor) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u32(value.color as u32);
    Ok(writer.finish())
}

fn decode_fixed_string32(event: &mut Event<'_>) -> Result<[u8; 32], EventError> {
    event
        .read_bytes(32)?
        .try_into()
        .map_err(|_| EventError::Host(SampClientSdkResult::NativeCallFailed))
}

fn encode_fixed_string32(value: [u8; 32]) -> Result<Vec<u8>, EventError> {
    Ok(value.to_vec())
}

fn decode_player_skill(event: &mut Event<'_>) -> Result<PlayerSkill, EventError> {
    Ok(PlayerSkill {
        player_id: event.read_u16()?,
        skill: decode_i32(event)?,
        level: event.read_u16()?,
    })
}

fn encode_player_skill(value: PlayerSkill) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u32(value.skill as u32);
    writer.u16(value.level);
    Ok(writer.finish())
}

fn decode_remove_building(event: &mut Event<'_>) -> Result<RemoveBuilding, EventError> {
    Ok(RemoveBuilding {
        model_id: decode_i32(event)?,
        position: decode_vector3(event)?,
        radius: event.read_f32()?,
    })
}

fn encode_remove_building(value: RemoveBuilding) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.model_id as u32);
    writer.vector3(value.position);
    writer.f32(value.radius);
    Ok(writer.finish())
}

fn decode_attach_object_to_player(
    event: &mut Event<'_>,
) -> Result<AttachObjectToPlayer, EventError> {
    Ok(AttachObjectToPlayer {
        object_id: event.read_u16()?,
        player_id: event.read_u16()?,
        offsets: decode_vector3(event)?,
        rotation: decode_vector3(event)?,
    })
}

fn encode_attach_object_to_player(value: AttachObjectToPlayer) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.object_id);
    writer.u16(value.player_id);
    writer.vector3(value.offsets);
    writer.vector3(value.rotation);
    Ok(writer.finish())
}

fn decode_explosion(event: &mut Event<'_>) -> Result<Explosion, EventError> {
    Ok(Explosion {
        position: decode_vector3(event)?,
        style: decode_i32(event)?,
        radius: event.read_f32()?,
    })
}

fn encode_explosion(value: Explosion) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.vector3(value.position);
    writer.u32(value.style as u32);
    writer.f32(value.radius);
    Ok(writer.finish())
}

fn decode_player_name_tag(event: &mut Event<'_>) -> Result<PlayerNameTag, EventError> {
    Ok(PlayerNameTag {
        player_id: event.read_u16()?,
        show: decode_bool8(event)?,
    })
}

fn encode_player_name_tag(value: PlayerNameTag) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u8(u8::from(value.show));
    Ok(writer.finish())
}

fn decode_player_fighting_style(event: &mut Event<'_>) -> Result<PlayerFightingStyle, EventError> {
    Ok(PlayerFightingStyle {
        player_id: event.read_u16()?,
        style_id: event.read_u8()?,
    })
}

fn encode_player_fighting_style(value: PlayerFightingStyle) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u8(value.style_id);
    Ok(writer.finish())
}

fn decode_vehicle_velocity(event: &mut Event<'_>) -> Result<VehicleVelocity, EventError> {
    Ok(VehicleVelocity {
        turn: decode_bool8(event)?,
        velocity: decode_vector3(event)?,
    })
}

fn encode_vehicle_velocity(value: VehicleVelocity) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(u8::from(value.turn));
    writer.vector3(value.velocity);
    Ok(writer.finish())
}

fn decode_pickup(event: &mut Event<'_>) -> Result<Pickup, EventError> {
    Ok(Pickup {
        id: decode_i32(event)?,
        model: decode_i32(event)?,
        pickup_type: decode_i32(event)?,
        position: decode_vector3(event)?,
    })
}

fn encode_pickup(value: Pickup) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.id as u32);
    writer.u32(value.model as u32);
    writer.u32(value.pickup_type as u32);
    writer.vector3(value.position);
    Ok(writer.finish())
}

fn decode_move_object(event: &mut Event<'_>) -> Result<MoveObject, EventError> {
    Ok(MoveObject {
        object_id: event.read_u16()?,
        from_position: decode_vector3(event)?,
        destination: decode_vector3(event)?,
        speed: event.read_f32()?,
        rotation: decode_vector3(event)?,
    })
}

fn encode_move_object(value: MoveObject) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.object_id);
    writer.vector3(value.from_position);
    writer.vector3(value.destination);
    writer.f32(value.speed);
    writer.vector3(value.rotation);
    Ok(writer.finish())
}

fn decode_text_draw_string(event: &mut Event<'_>) -> Result<TextDrawString, EventError> {
    let textdraw_id = event.read_u16()?;
    let length = usize::from(event.read_u16()?);
    if length > MAX_STRING32_BYTES {
        return Err(EventError::LengthExceedsLimit {
            length,
            limit: MAX_STRING32_BYTES,
        });
    }
    Ok(TextDrawString {
        textdraw_id,
        text: event.read_bytes(length)?,
    })
}

fn encode_text_draw_string(value: TextDrawString) -> Result<Vec<u8>, EventError> {
    if value.text.len() > MAX_STRING32_BYTES {
        return Err(EventError::LengthExceedsLimit {
            length: value.text.len(),
            limit: MAX_STRING32_BYTES,
        });
    }
    let mut writer = PayloadWriter::new();
    writer.u16(value.textdraw_id);
    writer.u16(value.text.len() as u16);
    writer.bytes(&value.text);
    Ok(writer.finish())
}

pub(super) fn decode_vector2(event: &mut Event<'_>) -> Result<Vector2, EventError> {
    Ok(Vector2 {
        x: event.read_f32()?,
        y: event.read_f32()?,
    })
}

pub(super) fn encode_vector2(writer: &mut PayloadWriter, value: Vector2) {
    writer.f32(value.x);
    writer.f32(value.y);
}

fn decode_gang_zone(event: &mut Event<'_>) -> Result<GangZone, EventError> {
    Ok(GangZone {
        zone_id: event.read_u16()?,
        square_start: decode_vector2(event)?,
        square_end: decode_vector2(event)?,
        color: decode_i32(event)?,
    })
}

fn encode_gang_zone(value: GangZone) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.zone_id);
    encode_vector2(&mut writer, value.square_start);
    encode_vector2(&mut writer, value.square_end);
    writer.u32(value.color as u32);
    Ok(writer.finish())
}

fn decode_u16_i32(event: &mut Event<'_>) -> Result<(u16, i32), EventError> {
    Ok((event.read_u16()?, decode_i32(event)?))
}

fn encode_u16_i32(value: (u16, i32)) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.0);
    writer.u32(value.1 as u32);
    Ok(writer.finish())
}

fn decode_vehicle_number_plate(event: &mut Event<'_>) -> Result<VehicleNumberPlate, EventError> {
    Ok(VehicleNumberPlate {
        vehicle_id: event.read_u16()?,
        text: event.read_string8()?,
    })
}

fn encode_vehicle_number_plate(value: VehicleNumberPlate) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.string8(&value.text)?;
    Ok(writer.finish())
}

fn decode_spectate(event: &mut Event<'_>) -> Result<Spectate, EventError> {
    Ok(Spectate {
        target_id: event.read_u16()?,
        camera_type: event.read_u8()?,
    })
}

fn encode_spectate(value: Spectate) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.target_id);
    writer.u8(value.camera_type);
    Ok(writer.finish())
}

fn decode_weapon_ammo(event: &mut Event<'_>) -> Result<WeaponAmmo, EventError> {
    Ok(WeaponAmmo {
        weapon_id: event.read_u8()?,
        ammo: event.read_u16()?,
    })
}

fn encode_weapon_ammo(value: WeaponAmmo) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(value.weapon_id);
    writer.u16(value.ammo);
    Ok(writer.finish())
}

fn decode_trailer_attachment(event: &mut Event<'_>) -> Result<TrailerAttachment, EventError> {
    Ok(TrailerAttachment {
        trailer_id: event.read_u16()?,
        vehicle_id: event.read_u16()?,
    })
}

fn encode_trailer_attachment(value: TrailerAttachment) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.trailer_id);
    writer.u16(value.vehicle_id);
    Ok(writer.finish())
}

fn decode_camera_look_at(event: &mut Event<'_>) -> Result<CameraLookAt, EventError> {
    Ok(CameraLookAt {
        position: decode_vector3(event)?,
        cut_type: event.read_u8()?,
    })
}

fn encode_camera_look_at(value: CameraLookAt) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.vector3(value.position);
    writer.u8(value.cut_type);
    Ok(writer.finish())
}

fn decode_vehicle_params(event: &mut Event<'_>) -> Result<VehicleParams, EventError> {
    Ok(VehicleParams {
        vehicle_id: event.read_u16()?,
        objective: decode_bool8(event)?,
        doors_locked: decode_bool8(event)?,
    })
}

fn encode_vehicle_params(value: VehicleParams) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u8(u8::from(value.objective));
    writer.u8(u8::from(value.doors_locked));
    Ok(writer.finish())
}

fn decode_player_enter_vehicle(event: &mut Event<'_>) -> Result<PlayerEnterVehicle, EventError> {
    Ok(PlayerEnterVehicle {
        player_id: event.read_u16()?,
        vehicle_id: event.read_u16()?,
        passenger: decode_bool8(event)?,
    })
}

fn encode_player_enter_vehicle(value: PlayerEnterVehicle) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u16(value.vehicle_id);
    writer.u8(u8::from(value.passenger));
    Ok(writer.finish())
}

fn decode_player_exit_vehicle(event: &mut Event<'_>) -> Result<PlayerExitVehicle, EventError> {
    Ok(PlayerExitVehicle {
        player_id: event.read_u16()?,
        vehicle_id: event.read_u16()?,
    })
}

fn encode_player_exit_vehicle(value: PlayerExitVehicle) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u16(value.vehicle_id);
    Ok(writer.finish())
}

fn decode_client_check(event: &mut Event<'_>) -> Result<ClientCheck, EventError> {
    Ok(ClientCheck {
        request_type: event.read_u8()?,
        subject: decode_i32(event)?,
        offset: event.read_u16()?,
        length: event.read_u16()?,
    })
}

fn encode_client_check(value: ClientCheck) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(value.request_type);
    writer.u32(value.subject as u32);
    writer.u16(value.offset);
    writer.u16(value.length);
    Ok(writer.finish())
}

pub(super) fn read_array<const N: usize>(event: &mut Event<'_>) -> Result<[u8; N], EventError> {
    event
        .read_bytes(N)?
        .try_into()
        .map_err(|_| EventError::Host(SampClientSdkResult::NativeCallFailed))
}

fn decode_vehicle_params_ex(event: &mut Event<'_>) -> Result<VehicleParamsEx, EventError> {
    Ok(VehicleParamsEx {
        vehicle_id: event.read_u16()?,
        params: read_array(event)?,
        doors: read_array(event)?,
        windows: read_array(event)?,
    })
}

fn encode_vehicle_params_ex(value: VehicleParamsEx) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.bytes(&value.params);
    writer.bytes(&value.doors);
    writer.bytes(&value.windows);
    Ok(writer.finish())
}

fn decode_vehicle_tuning_notification(
    event: &mut Event<'_>,
) -> Result<VehicleTuningNotification, EventError> {
    Ok(VehicleTuningNotification {
        player_id: event.read_u16()?,
        event: decode_i32(event)?,
        vehicle_id: decode_i32(event)?,
        param1: decode_i32(event)?,
        param2: decode_i32(event)?,
    })
}

fn encode_vehicle_tuning_notification(
    value: VehicleTuningNotification,
) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u32(value.event as u32);
    writer.u32(value.vehicle_id as u32);
    writer.u32(value.param1 as u32);
    writer.u32(value.param2 as u32);
    Ok(writer.finish())
}

fn decode_u16_u8(event: &mut Event<'_>) -> Result<(u16, u8), EventError> {
    Ok((event.read_u16()?, event.read_u8()?))
}

fn encode_u16_u8(value: (u16, u8)) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.0);
    writer.u8(value.1);
    Ok(writer.finish())
}

fn decode_vehicle_damage_status(event: &mut Event<'_>) -> Result<VehicleDamageStatus, EventError> {
    Ok(VehicleDamageStatus {
        vehicle_id: event.read_u16()?,
        panel_damage: decode_i32(event)?,
        door_damage: decode_i32(event)?,
        lights: event.read_u8()?,
        tires: event.read_u8()?,
    })
}

fn encode_vehicle_damage_status(value: VehicleDamageStatus) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u32(value.panel_damage as u32);
    writer.u32(value.door_damage as u32);
    writer.u8(value.lights);
    writer.u8(value.tires);
    Ok(writer.finish())
}

fn decode_actor(event: &mut Event<'_>) -> Result<Actor, EventError> {
    Ok(Actor {
        actor_id: event.read_u16()?,
        skin_id: decode_i32(event)?,
        position: decode_vector3(event)?,
        rotation: event.read_f32()?,
        health: event.read_f32()?,
    })
}

fn encode_actor(value: Actor) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.actor_id);
    writer.u32(value.skin_id as u32);
    writer.vector3(value.position);
    writer.f32(value.rotation);
    writer.f32(value.health);
    Ok(writer.finish())
}

fn decode_actor_angle(event: &mut Event<'_>) -> Result<ActorAngle, EventError> {
    Ok(ActorAngle {
        actor_id: event.read_u16()?,
        angle: event.read_f32()?,
    })
}

fn encode_actor_angle(value: ActorAngle) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.actor_id);
    writer.f32(value.angle);
    Ok(writer.finish())
}

fn decode_actor_position(event: &mut Event<'_>) -> Result<ActorPosition, EventError> {
    Ok(ActorPosition {
        actor_id: event.read_u16()?,
        position: decode_vector3(event)?,
    })
}

fn encode_actor_position(value: ActorPosition) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.actor_id);
    writer.vector3(value.position);
    Ok(writer.finish())
}

fn decode_actor_health(event: &mut Event<'_>) -> Result<ActorHealth, EventError> {
    Ok(ActorHealth {
        actor_id: event.read_u16()?,
        health: event.read_f32()?,
    })
}

fn encode_actor_health(value: ActorHealth) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.actor_id);
    writer.f32(value.health);
    Ok(writer.finish())
}

pub(super) fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
    Ok(event.read_u32()? as i32)
}

fn encode_i32(value: i32) -> Result<Vec<u8>, EventError> {
    Ok(value.to_le_bytes().to_vec())
}

fn decode_u8(event: &mut Event<'_>) -> Result<u8, EventError> {
    event.read_u8()
}

fn encode_u8(value: u8) -> Result<Vec<u8>, EventError> {
    Ok(vec![value])
}

pub(super) fn decode_u16(event: &mut Event<'_>) -> Result<u16, EventError> {
    event.read_u16()
}

pub(super) fn encode_u16(value: u16) -> Result<Vec<u8>, EventError> {
    Ok(value.to_le_bytes().to_vec())
}

fn decode_vehicle_position(event: &mut Event<'_>) -> Result<VehiclePosition, EventError> {
    Ok(VehiclePosition {
        vehicle_id: event.read_u16()?,
        position: decode_vector3(event)?,
    })
}

fn encode_vehicle_position(value: VehiclePosition) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.vector3(value.position);
    Ok(writer.finish())
}

fn decode_vehicle_angle(event: &mut Event<'_>) -> Result<VehicleAngle, EventError> {
    Ok(VehicleAngle {
        vehicle_id: event.read_u16()?,
        angle: event.read_f32()?,
    })
}

fn encode_vehicle_angle(value: VehicleAngle) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.f32(value.angle);
    Ok(writer.finish())
}

fn decode_vehicle_health(event: &mut Event<'_>) -> Result<VehicleHealth, EventError> {
    Ok(VehicleHealth {
        vehicle_id: event.read_u16()?,
        health: event.read_f32()?,
    })
}

fn encode_vehicle_health(value: VehicleHealth) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.f32(value.health);
    Ok(writer.finish())
}

fn decode_empty(_event: &mut Event<'_>) -> Result<(), EventError> {
    Ok(())
}

fn encode_empty(_value: ()) -> Result<Vec<u8>, EventError> {
    Ok(Vec::new())
}
