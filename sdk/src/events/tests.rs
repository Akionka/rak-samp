use super::test_support::{
    TestEvent, assert_encoded_string_protocol_replacement_round_trip,
    assert_protocol_replacement_round_trip, test_api,
};
use super::*;
use crate::{SampClientSdkEventV1, SampClientSdkHookAction};
use samp_protocol::{
    EncodedStringWireDescriptor, WireDescriptor,
    packet::r1 as protocol_packet,
    rpc::incoming::{common as protocol_common, r1 as protocol_r1},
    types::{Vector2 as ProtocolVector2, Vector3 as ProtocolVector3},
};

fn test_protocol_vector3(x: f32, y: f32, z: f32) -> ProtocolVector3 {
    ProtocolVector3 { x, y, z }
}

fn test_protocol_vector2(x: f32, y: f32) -> ProtocolVector2 {
    ProtocolVector2 { x, y }
}

fn test_spawn_info() -> protocol_r1::SpawnInfo {
    protocol_r1::SpawnInfo {
        team: 7,
        skin: 411,
        unused: 0xA5,
        position: test_protocol_vector3(1.0, 2.0, 3.0),
        rotation: 4.0,
        weapons: [22, 24, 31],
        ammo: [100, 200, 300],
    }
}

fn test_protocol_animation() -> protocol_r1::Animation {
    protocol_r1::Animation {
        animation_library: b"PED".to_vec(),
        animation_name: b"WALK".to_vec(),
        frame_delta: 4.0,
        looped: true,
        lock_x: false,
        lock_y: true,
        freeze: false,
        time: -1,
    }
}

#[test]
fn vehicle_position_through_player_name_tag_decode_and_atomically_replace() {
    use samp_protocol::types::Vector3;

    fn assert_replacement_round_trip<D>(descriptor: D, value: D::Value)
    where
        D: samp_protocol::WireDescriptor,
        D::Value: Clone + ::core::fmt::Debug + PartialEq,
    {
        assert_protocol_replacement_round_trip(descriptor, value);
    }

    assert_encoded_string_protocol_replacement_round_trip(
        protocol_common::SHOW_DIALOG,
        protocol_common::ShowDialog {
            dialog_id: 1,
            style: 2,
            title: b"title".to_vec(),
            button1: b"ok".to_vec(),
            button2: b"cancel".to_vec(),
            text: b"encoded dialog text".to_vec(),
        },
    );

    assert_replacement_round_trip(
        protocol_common::SET_VEHICLE_POSITION,
        protocol_common::VehiclePosition {
            vehicle_id: 1,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_VEHICLE_ANGLE,
        protocol_common::VehicleAngle {
            vehicle_id: 2,
            angle: 90.0,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_VEHICLE_HEALTH,
        protocol_common::VehicleHealth {
            vehicle_id: 3,
            health: 750.0,
        },
    );
    assert_replacement_round_trip(protocol_common::RESET_PLAYER_MONEY, ());
    assert_replacement_round_trip(protocol_common::RESET_PLAYER_WEAPONS, ());
    assert_replacement_round_trip(protocol_common::CANCEL_EDIT, ());
    assert_replacement_round_trip(protocol_common::SET_TOGGLE_CLOCK, true);
    assert_replacement_round_trip(protocol_common::SET_PLAYER_DRUNK, -2);
    assert_replacement_round_trip(
        protocol_common::SET_RACE_CHECKPOINT,
        protocol_common::RaceCheckpoint {
            checkpoint_type: 1,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            next_position: Vector3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
            size: 7.0,
        },
    );
    assert_replacement_round_trip(
        protocol_common::PLAY_AUDIO_STREAM,
        protocol_common::AudioStream {
            url: b"https://example.invalid/audio".to_vec(),
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            radius: 50.0,
            use_position: true,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_OBJECT_POSITION,
        protocol_common::ObjectPosition {
            object_id: 4,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_OBJECT_ROTATION,
        protocol_common::ObjectRotation {
            object_id: 5,
            rotation: Vector3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        },
    );
    assert_replacement_round_trip(protocol_common::DESTROY_OBJECT, 6);
    assert_replacement_round_trip(
        protocol_common::PLAYER_DEATH_NOTIFICATION,
        protocol_common::PlayerDeathNotification {
            killer_id: 7,
            killed_id: 8,
            reason: 9,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_MAP_ICON,
        protocol_common::MapIcon {
            icon_id: 1,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            icon_type: 2,
            color: -1,
            style: 3,
        },
    );
    assert_replacement_round_trip(
        protocol_common::REMOVE_VEHICLE_COMPONENT,
        protocol_common::VehicleComponent {
            vehicle_id: 10,
            component_id: 11,
        },
    );
    assert_replacement_round_trip(protocol_common::REMOVE_3D_TEXT_LABEL, 12);
    assert_replacement_round_trip(protocol_common::UPDATE_GLOBAL_TIMER, 13);
    assert_replacement_round_trip(protocol_common::DESTROY_PICKUP, -14);
    assert_replacement_round_trip(
        protocol_common::LINK_VEHICLE_TO_INTERIOR,
        protocol_common::VehicleInterior {
            vehicle_id: 15,
            interior_id: 16,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_PLAYER_COLOR,
        protocol_common::PlayerColor {
            player_id: 17,
            color: -1,
        },
    );
    assert_replacement_round_trip(protocol_common::REQUEST_SPAWN_RESPONSE, false);
    assert_replacement_round_trip(protocol_common::SET_SHOP_NAME, [b'S'; 32]);
    assert_replacement_round_trip(
        protocol_common::SET_PLAYER_SKILL_LEVEL,
        protocol_common::PlayerSkill {
            player_id: 18,
            skill: 19,
            level: 20,
        },
    );
    assert_replacement_round_trip(
        protocol_common::REMOVE_BUILDING,
        protocol_common::RemoveBuilding {
            model_id: 21,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            radius: 4.0,
        },
    );
    assert_replacement_round_trip(
        protocol_common::ATTACH_OBJECT_TO_PLAYER,
        protocol_common::AttachObjectToPlayer {
            object_id: 22,
            player_id: 23,
            offsets: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            rotation: Vector3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        },
    );
    assert_replacement_round_trip(protocol_common::SHOW_MENU, 24);
    assert_replacement_round_trip(protocol_common::HIDE_MENU, 25);
    assert_replacement_round_trip(
        protocol_common::CREATE_EXPLOSION,
        protocol_common::Explosion {
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            style: 26,
            radius: 27.0,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SHOW_PLAYER_NAME_TAG,
        protocol_common::PlayerNameTag {
            player_id: 28,
            show: true,
        },
    );
}

#[test]
fn client_check_through_set_camera_behind_decode_and_atomically_replace() {
    use samp_protocol::types::Vector3;

    fn assert_replacement_round_trip<D>(descriptor: D, value: D::Value)
    where
        D: samp_protocol::WireDescriptor,
        D::Value: Clone + ::core::fmt::Debug + PartialEq,
    {
        assert_protocol_replacement_round_trip(descriptor, value);
    }

    assert_replacement_round_trip(
        protocol_common::CLIENT_CHECK,
        protocol_common::ClientCheck {
            request_type: 1,
            subject: -2,
            offset: 3,
            length: 4,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_VEHICLE_PARAMS_EX,
        protocol_common::VehicleParamsEx {
            vehicle_id: 5,
            params: [6; 8],
            doors: [7; 4],
            windows: [8; 4],
        },
    );
    assert_replacement_round_trip(
        protocol_common::VEHICLE_TUNING_NOTIFICATION,
        protocol_common::VehicleTuningNotification {
            player_id: 9,
            event: 10,
            vehicle_id: 11,
            param1: 12,
            param2: 13,
        },
    );
    assert_replacement_round_trip(protocol_common::SET_VEHICLE_TIRES, (14, 15));
    assert_replacement_round_trip(
        protocol_common::VEHICLE_DAMAGE_STATUS_UPDATE,
        protocol_common::VehicleDamageStatus {
            vehicle_id: 16,
            panel_damage: 17,
            door_damage: 18,
            lights: 19,
            tires: 20,
        },
    );
    assert_replacement_round_trip(protocol_common::TOGGLE_WIDESCREEN, true);
    assert_replacement_round_trip(protocol_common::DESTROY_ACTOR, 21);
    assert_replacement_round_trip(protocol_common::DESTROY_WEAPON_PICKUP, 22);
    assert_replacement_round_trip(protocol_common::EDIT_ATTACHED_OBJECT, -23);
    assert_replacement_round_trip(protocol_common::ENTER_SELECT_OBJECT, ());
    assert_replacement_round_trip(protocol_common::SERVER_STATISTICS_RESPONSE, ());
    assert_replacement_round_trip(protocol_common::SET_PLAYER_DRUNK_VISUALS, -24);
    assert_replacement_round_trip(protocol_common::SET_PLAYER_DRUNK_HANDLING, 25);
    assert_replacement_round_trip(
        protocol_common::CREATE_ACTOR,
        protocol_common::Actor {
            actor_id: 26,
            skin_id: 27,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            rotation: 4.0,
            health: 5.0,
        },
    );
    assert_replacement_round_trip(protocol_common::CLEAR_ACTOR_ANIMATION, 28);
    assert_replacement_round_trip(
        protocol_common::SET_ACTOR_FACING_ANGLE,
        protocol_common::ActorAngle {
            actor_id: 29,
            angle: 30.0,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_ACTOR_POSITION,
        protocol_common::ActorPosition {
            actor_id: 31,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_ACTOR_HEALTH,
        protocol_common::ActorHealth {
            actor_id: 32,
            health: 33.0,
        },
    );
    assert_replacement_round_trip(protocol_common::SET_PLAYER_OBJECT_NO_CAMERA_COL, 34);
    assert_replacement_round_trip(protocol_common::DISABLE_CHECKPOINT, ());
    assert_replacement_round_trip(protocol_common::DISABLE_RACE_CHECKPOINT, ());
    assert_replacement_round_trip(protocol_common::GAMEMODE_RESTART, ());
    assert_replacement_round_trip(protocol_common::STOP_AUDIO_STREAM, ());
    assert_replacement_round_trip(protocol_common::REMOVE_PLAYER_FROM_VEHICLE, ());
    assert_replacement_round_trip(protocol_common::FORCE_CLASS_SELECTION, ());
    assert_replacement_round_trip(protocol_common::SET_CAMERA_BEHIND, ());
}

#[test]
fn common_world_incoming_rpcs_decode_and_atomically_replace() {
    use samp_protocol::types::{Vector2, Vector3};

    fn assert_replacement_round_trip<D>(descriptor: D, value: D::Value)
    where
        D: samp_protocol::WireDescriptor,
        D::Value: Clone + ::core::fmt::Debug + PartialEq,
    {
        assert_protocol_replacement_round_trip(descriptor, value);
    }

    assert_replacement_round_trip(protocol_common::ATTACH_CAMERA_TO_OBJECT, 1);
    assert_replacement_round_trip(protocol_common::GANG_ZONE_STOP_FLASH, 2);
    assert_replacement_round_trip(protocol_common::CLEAR_PLAYER_ANIMATION, 3);
    assert_replacement_round_trip(protocol_common::SET_PLAYER_SPECIAL_ACTION, 4);
    assert_replacement_round_trip(
        protocol_common::SET_PLAYER_FIGHTING_STYLE,
        protocol_common::PlayerFightingStyle {
            player_id: 5,
            style_id: 6,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_PLAYER_VELOCITY,
        Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_VEHICLE_VELOCITY,
        protocol_common::VehicleVelocity {
            turn: true,
            velocity: Vector3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        },
    );
    assert_replacement_round_trip(
        protocol_common::CREATE_PICKUP,
        protocol_common::Pickup {
            id: 7,
            model: 8,
            pickup_type: 9,
            position: Vector3 {
                x: 10.0,
                y: 11.0,
                z: 12.0,
            },
        },
    );
    assert_replacement_round_trip(
        protocol_common::MOVE_OBJECT,
        protocol_common::MoveObject {
            object_id: 13,
            from_position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            destination: Vector3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
            speed: 7.0,
            rotation: Vector3 {
                x: 8.0,
                y: 9.0,
                z: 10.0,
            },
        },
    );
    assert_replacement_round_trip(
        protocol_common::TEXT_DRAW_SET_STRING,
        protocol_common::TextDrawString {
            textdraw_id: 14,
            text: b"text".to_vec(),
        },
    );
    assert_replacement_round_trip(
        protocol_common::CREATE_GANG_ZONE,
        protocol_common::GangZone {
            zone_id: 15,
            square_start: Vector2 { x: 1.0, y: 2.0 },
            square_end: Vector2 { x: 3.0, y: 4.0 },
            color: 16,
        },
    );
    assert_replacement_round_trip(protocol_common::GANG_ZONE_DESTROY, 17);
    assert_replacement_round_trip(protocol_common::GANG_ZONE_FLASH, (18, 19));
    assert_replacement_round_trip(protocol_common::STOP_OBJECT, 20);
    assert_replacement_round_trip(
        protocol_common::SET_VEHICLE_NUMBER_PLATE,
        protocol_common::VehicleNumberPlate {
            vehicle_id: 21,
            text: b"plate".to_vec(),
        },
    );
    assert_replacement_round_trip(
        protocol_common::SPECTATE_PLAYER,
        protocol_common::Spectate {
            target_id: 22,
            camera_type: 23,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SPECTATE_VEHICLE,
        protocol_common::Spectate {
            target_id: 24,
            camera_type: 25,
        },
    );
    assert_replacement_round_trip(protocol_common::CONNECTION_REJECTED, 26);
    assert_replacement_round_trip(protocol_common::REMOVE_MAP_ICON, 27);
    assert_replacement_round_trip(
        protocol_common::SET_WEAPON_AMMO,
        protocol_common::WeaponAmmo {
            weapon_id: 28,
            ammo: 29,
        },
    );
    assert_replacement_round_trip(protocol_common::SET_GRAVITY, 30.0);
    assert_replacement_round_trip(
        protocol_common::ATTACH_TRAILER_TO_VEHICLE,
        protocol_common::TrailerAttachment {
            trailer_id: 31,
            vehicle_id: 32,
        },
    );
    assert_replacement_round_trip(protocol_common::DETACH_TRAILER_FROM_VEHICLE, 33);
    assert_replacement_round_trip(
        protocol_common::SET_CAMERA_POSITION,
        Vector3 {
            x: 34.0,
            y: 35.0,
            z: 36.0,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_CAMERA_LOOK_AT,
        protocol_common::CameraLookAt {
            position: Vector3 {
                x: 37.0,
                y: 38.0,
                z: 39.0,
            },
            cut_type: 40,
        },
    );
    assert_replacement_round_trip(
        protocol_common::SET_VEHICLE_PARAMS,
        protocol_common::VehicleParams {
            vehicle_id: 41,
            objective: true,
            doors_locked: false,
        },
    );
    assert_replacement_round_trip(protocol_common::PLAYER_DEATH, 42);
    assert_replacement_round_trip(
        protocol_common::PLAYER_ENTER_VEHICLE,
        protocol_common::PlayerEnterVehicle {
            player_id: 43,
            vehicle_id: 44,
            passenger: true,
        },
    );
    assert_replacement_round_trip(
        protocol_common::PLAYER_EXIT_VEHICLE,
        protocol_common::PlayerExitVehicle {
            player_id: 45,
            vehicle_id: 46,
        },
    );
}

#[test]
fn r1_player_stream_in_includes_all_eleven_weapon_skill_levels() {
    use samp_protocol::WireDescriptor;

    let value = protocol_r1::PlayerStreamIn {
        player_id: 42,
        team: 3,
        model: 411,
        position: test_protocol_vector3(1.0, 2.0, 3.0),
        rotation: 90.0,
        color: -1,
        fighting_style: 4,
        weapon_skill_levels: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    };
    let encoded = protocol_r1::PlayerStreamInRpc::encode_bits(&value)
        .expect("R1 player stream-in payload must encode");

    assert_eq!(encoded.len_bits(), 400);
    assert_eq!(
        encoded.as_bytes(),
        &[
            0x2A, 0x00, 0x03, 0x9B, 0x01, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00,
            0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0xB4, 0x42, 0xFF, 0xFF, 0xFF, 0xFF, 0x04,
            0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06, 0x00,
            0x07, 0x00, 0x08, 0x00, 0x09, 0x00, 0x0A, 0x00,
        ]
    );
    assert_protocol_replacement_round_trip(protocol_r1::PLAYER_STREAM_IN, value);
}

#[test]
fn r1_complex_incoming_rpc_helpers_decode_and_atomically_replace() {
    let settings = protocol_r1::GameSettings {
        zone_names: true,
        use_cj_walk: false,
        allow_weapons: true,
        limit_global_chat_radius: false,
        global_chat_radius: 100.0,
        stunt_bonus: true,
        nametag_draw_distance: 70.0,
        disable_enter_exits: false,
        nametag_los: true,
        tire_popping: false,
        classes_available: 5,
        show_player_tags: true,
        player_markers_mode: 1,
        world_time: 12,
        world_weather: 7,
        gravity: 0.008,
        lan_mode: false,
        death_money_drop: 500,
        instagib: false,
        normal_onfoot_send_rate: 30,
        normal_incar_send_rate: 30,
        normal_firing_send_rate: 30,
        send_multiplier: 2,
        lag_compensation_mode: 1,
        vehicle_friendly_fire: true,
    };
    assert_protocol_replacement_round_trip(
        protocol_r1::INIT_GAME,
        protocol_r1::InitGame {
            player_id: 42,
            host_name: b"R1 host".to_vec(),
            settings,
            vehicle_models: [1; 212],
        },
    );
    assert_protocol_replacement_round_trip(
        protocol_r1::REQUEST_CLASS_RESPONSE,
        protocol_r1::RequestClassResponse {
            can_spawn: true,
            spawn: test_spawn_info(),
        },
    );
    assert_protocol_replacement_round_trip(
        protocol_r1::PLAYER_STREAM_IN,
        protocol_r1::PlayerStreamIn {
            player_id: 42,
            team: 3,
            model: 411,
            position: test_protocol_vector3(1.0, 2.0, 3.0),
            rotation: 90.0,
            color: -1,
            fighting_style: 4,
            weapon_skill_levels: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        },
    );
    assert_encoded_string_protocol_replacement_round_trip(
        protocol_r1::CREATE_3D_TEXT,
        protocol_r1::TextLabel3D {
            id: 4,
            color: -1,
            position: test_protocol_vector3(1.0, 2.0, 3.0),
            distance: 50.0,
            test_los: true,
            attached_player_id: u16::MAX,
            attached_vehicle_id: u16::MAX,
            text: b"encoded 3D text".to_vec(),
        },
    );
    assert_encoded_string_protocol_replacement_round_trip(
        protocol_r1::CREATE_OBJECT,
        protocol_r1::Object {
            object_id: 9,
            model_id: 1337,
            position: test_protocol_vector3(1.0, 2.0, 3.0),
            rotation: test_protocol_vector3(4.0, 5.0, 6.0),
            draw_distance: 300.0,
            no_camera_collision: true,
            attach_to_vehicle_id: u16::MAX,
            attach_to_object_id: u16::MAX,
            attachment: None,
            textures_count: 2,
            materials: vec![
                protocol_r1::ObjectMaterial::Texture(protocol_r1::TextureMaterial {
                    material_id: 0,
                    model_id: 18646,
                    library_name: b"matcolours".to_vec(),
                    texture_name: b"grey-10-percent".to_vec(),
                    color: -1,
                }),
                protocol_r1::ObjectMaterial::Text(protocol_r1::TextMaterial {
                    material_id: 1,
                    material_size: 90,
                    font_name: b"Arial".to_vec(),
                    font_size: 20,
                    bold: 1,
                    font_color: -1,
                    background_color: 0,
                    align: 2,
                    text: b"material text".to_vec(),
                }),
            ],
        },
    );
    assert_protocol_replacement_round_trip(protocol_r1::SET_SPAWN_INFO, test_spawn_info());
    assert_protocol_replacement_round_trip(
        protocol_r1::INIT_MENU,
        protocol_r1::InitMenu {
            menu_id: 1,
            two_columns: true,
            title: *b"R1 menu                         ",
            position: test_protocol_vector2(10.0, 20.0),
            columns: vec![
                protocol_r1::MenuColumn {
                    width: 100.0,
                    title: *b"first                           ",
                    rows: vec![*b"one                             "],
                },
                protocol_r1::MenuColumn {
                    width: 200.0,
                    title: *b"second                          ",
                    rows: vec![*b"two                             "],
                },
            ],
            rows: [-1; protocol_r1::MAX_MENU_ROWS],
            menu: false,
        },
    );
    assert_protocol_replacement_round_trip(
        protocol_r1::INTERPOLATE_CAMERA,
        protocol_r1::InterpolateCamera {
            set_position: true,
            from_position: test_protocol_vector3(1.0, 2.0, 3.0),
            destination: test_protocol_vector3(4.0, 5.0, 6.0),
            time_ms: 500,
            mode: 2,
        },
    );
    assert_protocol_replacement_round_trip(
        protocol_r1::TOGGLE_SELECT_TEXT_DRAW,
        protocol_r1::ToggleSelectTextDraw {
            enabled: true,
            hover_color: -1,
        },
    );
    assert_encoded_string_protocol_replacement_round_trip(
        protocol_r1::SET_OBJECT_MATERIAL,
        protocol_r1::ObjectMaterialUpdate {
            object_id: 9,
            material: protocol_r1::ObjectMaterial::Texture(protocol_r1::TextureMaterial {
                material_id: 1,
                model_id: 123,
                library_name: b"lib".to_vec(),
                texture_name: b"texture".to_vec(),
                color: 0x1122_3344,
            }),
        },
    );
    assert_encoded_string_protocol_replacement_round_trip(
        protocol_r1::SET_OBJECT_MATERIAL,
        protocol_r1::ObjectMaterialUpdate {
            object_id: 9,
            material: protocol_r1::ObjectMaterial::Text(protocol_r1::TextMaterial {
                material_id: 2,
                material_size: 90,
                font_name: b"Arial".to_vec(),
                font_size: 20,
                bold: 0,
                font_color: -1,
                background_color: 0,
                align: 1,
                text: b"encoded material update".to_vec(),
            }),
        },
    );
    assert_protocol_replacement_round_trip(
        protocol_r1::APPLY_PLAYER_ANIMATION,
        protocol_r1::PlayerAnimation {
            player_id: 7,
            animation: test_protocol_animation(),
        },
    );
    assert_protocol_replacement_round_trip(protocol_r1::ENABLE_STUNT_BONUS, true);
    assert_protocol_replacement_round_trip(
        protocol_r1::PLAY_CRIME_REPORT,
        protocol_r1::CrimeReport {
            suspect_id: 7,
            in_vehicle: true,
            vehicle_model: 411,
            vehicle_color: 4,
            crime: 9,
            coordinates: test_protocol_vector3(1.0, 2.0, 3.0),
        },
    );
    assert_protocol_replacement_round_trip(
        protocol_r1::SET_PLAYER_ATTACHED_OBJECT,
        protocol_r1::PlayerAttachedObject {
            player_id: 7,
            index: 3,
            object: Some(protocol_r1::AttachedObject {
                model_id: 19327,
                bone: 1,
                offset: test_protocol_vector3(1.0, 2.0, 3.0),
                rotation: test_protocol_vector3(4.0, 5.0, 6.0),
                scale: test_protocol_vector3(1.0, 1.0, 1.0),
                color1: -1,
                color2: 0,
            }),
        },
    );
    assert_protocol_replacement_round_trip(
        protocol_r1::ENTER_EDIT_OBJECT,
        protocol_r1::EnterEditObject {
            player_object: true,
            object_id: 5,
        },
    );
    assert_protocol_replacement_round_trip(protocol_r1::TOGGLE_PLAYER_SPECTATING, false);
    assert_protocol_replacement_round_trip(
        protocol_r1::SHOW_TEXT_DRAW,
        protocol_r1::ShowTextDraw {
            textdraw_id: 99,
            textdraw: protocol_r1::TextDraw {
                flags: 1,
                letter_width: 0.5,
                letter_height: 1.0,
                letter_color: -1,
                line_width: 2.0,
                line_height: 3.0,
                box_color: 0,
                shadow: 1,
                outline: 2,
                background_color: 0,
                style: 4,
                selectable: 1,
                position: test_protocol_vector2(100.0, 200.0),
                model_id: 1234,
                rotation: test_protocol_vector3(0.0, 0.0, 1.0),
                zoom: 1.5,
                color1: -1,
                color2: 2,
                text: b"textdraw".to_vec(),
            },
        },
    );
    assert_protocol_replacement_round_trip(protocol_r1::TEXT_DRAW_HIDE, 99);
    assert_protocol_replacement_round_trip(
        protocol_r1::UPDATE_SCORES_AND_PINGS,
        protocol_r1::ScoresAndPings {
            entries: vec![protocol_r1::ScorePing {
                player_id: 7,
                score: -100,
                ping: 42,
            }],
        },
    );
    assert_protocol_replacement_round_trip(
        protocol_r1::VEHICLE_STREAM_IN,
        protocol_r1::VehicleStreamIn {
            vehicle_id: 9,
            vehicle: protocol_r1::StreamedVehicle {
                model: 411,
                position: test_protocol_vector3(1.0, 2.0, 3.0),
                rotation: 45.0,
                body_color1: 1,
                body_color2: 2,
                health: 900.0,
                interior_id: 3,
                door_damage_status: 4,
                panel_damage_status: 5,
                light_damage_status: 6,
                tire_damage_status: 7,
                add_siren: 8,
                mod_slots: [9; 14],
                paint_job: 10,
                interior_color1: 11,
                interior_color2: 12,
            },
        },
    );
    assert_protocol_replacement_round_trip(protocol_r1::DISABLE_VEHICLE_COLLISIONS, true);
    assert_protocol_replacement_round_trip(protocol_r1::TOGGLE_CAMERA_TARGET_NOTIFYING, false);
    assert_protocol_replacement_round_trip(
        protocol_r1::APPLY_ACTOR_ANIMATION,
        protocol_r1::ActorAnimation {
            actor_id: 8,
            animation: test_protocol_animation(),
        },
    );
}

#[test]
fn r1_remote_sync_and_markers_decode_and_atomically_replace() {
    assert_protocol_replacement_round_trip(
        protocol_packet::PLAYER_SYNC,
        protocol_packet::RemotePlayerSync {
            player_id: 1,
            left_right_keys: Some(2),
            up_down_keys: None,
            key_data: 3,
            position: test_protocol_vector3(1.0, 2.0, 3.0),
            quaternion: [-1.0, 0.0, 0.0, 0.0],
            health: 100,
            armour: 98,
            weapon: 24,
            special_action: 0,
            move_speed: test_protocol_vector3(0.0, 0.0, 0.0),
            surfing: Some(protocol_packet::RemotePlayerSurfing {
                vehicle_id: 4,
                offsets: test_protocol_vector3(4.0, 5.0, 6.0),
            }),
            animation: Some(protocol_packet::RemotePlayerAnimation { id: 7, flags: 8 }),
        },
    );
    assert_protocol_replacement_round_trip(
        protocol_packet::VEHICLE_SYNC,
        protocol_packet::RemoteVehicleSync {
            player_id: 1,
            vehicle_id: 2,
            left_right_keys: 3,
            up_down_keys: 4,
            key_data: 5,
            quaternion: [1.0, 0.0, 0.0, 0.0],
            position: test_protocol_vector3(1.0, 2.0, 3.0),
            // R1's compressed-vector zero components decode to -1 / 65536 after the
            // writer's integer conversion; use the exact representable values here.
            move_speed: test_protocol_vector3(1.0, -1.0 / 65_536.0, -1.0 / 65_536.0),
            vehicle_health: 900,
            player_health: 98,
            armour: 0,
            current_weapon: 24,
            siren: true,
            landing_gear: false,
            train_speed: Some(-7),
            trailer_id: Some(6),
        },
    );
    assert_protocol_replacement_round_trip(
        protocol_packet::MARKERS_SYNC,
        protocol_packet::MarkersSync {
            markers: vec![
                protocol_packet::Marker {
                    player_id: 1,
                    coordinates: None,
                },
                protocol_packet::Marker {
                    player_id: 2,
                    coordinates: Some(protocol_packet::MarkerCoordinates { x: -1, y: -2, z: 3 }),
                },
            ],
        },
    );
}

#[test]
fn typed_helpers_reject_trailing_bits_before_invoking_the_callback() {
    use samp_protocol::WireDescriptor;

    let api = test_api();
    let mut raw = TestEvent::new(
        protocol_r1::EnableStuntBonusRpc::ID,
        samp_protocol::EncodedBits::from_bits(vec![0b1000_0000], 2).unwrap(),
    );
    let mut event = unsafe {
        Event::from_callback(
            api,
            (&mut raw as *mut TestEvent).cast::<SampClientSdkEventV1>(),
        )
    }
    .unwrap();
    assert!(matches!(
        super::handle_protocol::<protocol_r1::EnableStuntBonusRpc>(&mut event, |_| panic!(
            "must not dispatch"
        ),),
        Err(super::core::ProtocolEventError::DecodeMalformed(
            samp_protocol::DecodeError::UnexpectedTrailingBits {
                remaining_bits: 1,
                allowed_bits: 0,
            }
        ))
    ));
}

#[test]
fn protocol_replacement_validates_canonical_framing_before_host_mutation() {
    use samp_protocol::{
        BitRead, BitWrite, DecodeError, EncodeError, ExactBytesPolicy, IncomingRpc, WireCodec,
    };

    struct NonByteAlignedCodec;

    impl WireCodec for NonByteAlignedCodec {
        type Value = bool;

        fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
            reader
                .read_left_aligned_bits(8)
                .map(|bits| bits[0] != 0)
                .map_err(DecodeError::Source)
        }

        fn encode<W: BitWrite>(
            writer: &mut W,
            value: &Self::Value,
        ) -> Result<(), EncodeError<W::Error>> {
            writer
                .write_left_aligned_bits(&[u8::from(*value) << 7], 1)
                .map_err(EncodeError::Source)
        }
    }

    type Descriptor = IncomingRpc<201, NonByteAlignedCodec, ExactBytesPolicy>;

    let original = samp_protocol::EncodedBits::from_bits(vec![0x80], 8).unwrap();
    let mut raw = TestEvent::new(201, original);
    let mut event = unsafe {
        Event::from_callback(
            test_api(),
            (&mut raw as *mut TestEvent).cast::<SampClientSdkEventV1>(),
        )
    }
    .unwrap();

    assert!(matches!(
        super::handle_protocol::<Descriptor>(&mut event, ProtocolAction::Replace),
        Err(super::core::ProtocolEventError::ReplacementEncode(
            samp_protocol::EncodeError::NonByteAlignedPayload { bit_len: 1 }
        ))
    ));
    assert_eq!(raw.bytes, [0x80]);
    assert_eq!(raw.bit_len, 8);
}

#[test]
fn marker_sync_keeps_negative_r1_coordinates_as_signed_i16_values() {
    let payload = protocol_packet::MarkersSyncPacket::encode_bits(&protocol_packet::MarkersSync {
        markers: vec![
            protocol_packet::Marker {
                player_id: 1,
                coordinates: None,
            },
            protocol_packet::Marker {
                player_id: 2,
                coordinates: Some(protocol_packet::MarkerCoordinates { x: -1, y: -2, z: 3 }),
            },
        ],
    })
    .unwrap();
    assert_eq!(payload.len_bits(), 114);
    assert_eq!(
        payload.as_bytes(),
        &[
            2, 0, 0, 0, 1, 0, 1, 0, 0x7F, 0xFF, 0xFF, 0xBF, 0xC0, 0xC0, 0
        ]
    );
}

#[test]
fn marker_sync_accepts_terminal_byte_alignment_padding() {
    let api = test_api();
    let value = protocol_packet::MarkersSync {
        markers: vec![protocol_packet::Marker {
            player_id: 1,
            coordinates: None,
        }],
    };
    let canonical = protocol_packet::MarkersSyncPacket::encode_bits(&value)
        .expect("marker payload must encode");
    assert_eq!(canonical.len_bits(), 49);

    let mut bytes = canonical.as_bytes().to_vec();
    // The packet transport can leave its terminal byte's unused bits unspecified.
    *bytes.last_mut().expect("marker payload has a final byte") |= 0x40;
    let padded = samp_protocol::EncodedBits::from_bits(bytes, 56)
        .expect("the rounded marker payload remains in its buffer");
    let mut raw = TestEvent::new(protocol_packet::MarkersSyncPacket::ID, padded);
    let mut event = unsafe {
        Event::from_callback(
            api,
            (&mut raw as *mut TestEvent).cast::<SampClientSdkEventV1>(),
        )
    }
    .expect("test event is not null");
    assert_eq!(
        super::handle_protocol::<protocol_packet::MarkersSyncPacket>(&mut event, |decoded| {
            assert_eq!(decoded, value);
            ProtocolAction::Replace(decoded)
        })
        .expect("terminal alignment padding must be accepted"),
        SampClientSdkHookAction::Continue
    );
    assert_eq!(raw.bit_len, canonical.len_bits());
    assert_eq!(raw.bytes, canonical.as_bytes());

    let mut bytes = canonical.as_bytes().to_vec();
    bytes.push(0);
    let mut raw = TestEvent::new(
        protocol_packet::MarkersSyncPacket::ID,
        samp_protocol::EncodedBits::from_bits(bytes, 57).expect("the malformed suffix fits"),
    );
    let mut event = unsafe {
        Event::from_callback(
            api,
            (&mut raw as *mut TestEvent).cast::<SampClientSdkEventV1>(),
        )
    }
    .expect("test event is not null");
    assert!(matches!(
        super::handle_protocol::<protocol_packet::MarkersSyncPacket>(&mut event, |_| panic!(
            "a full trailing byte must not dispatch"
        )),
        Err(super::core::ProtocolEventError::DecodeMalformed(
            samp_protocol::DecodeError::InvalidTerminalPaddingLength {
                remaining_bits: 8,
                required_bits: 7,
            }
        ))
    ));
}

#[test]
fn set_player_skin_uses_rpc_153_and_two_i32_values() {
    use samp_protocol::{
        WireDescriptor,
        rpc::incoming::common::{PlayerSkin, SetPlayerSkin},
    };

    assert_eq!(SetPlayerSkin::ID, 153);
    let bits = SetPlayerSkin::encode_bits(&PlayerSkin {
        player_id: 0,
        skin_id: 411,
    })
    .expect("valid i32 skin payload");

    assert_eq!(bits.as_bytes(), [0, 0, 0, 0, 0x9B, 0x01, 0, 0]);
    assert_eq!(bits.len_bits(), 64);
}

#[test]
fn r1_complex_incoming_rpc_helpers_use_their_protocol_ids() {
    use samp_protocol::WireDescriptor;

    let descriptors = [
        (protocol_r1::InitGameRpc::ID, 139),
        (protocol_r1::RequestClassResponseRpc::ID, 128),
        (protocol_r1::PlayerStreamInRpc::ID, 32),
        (protocol_r1::Create3DTextRpc::ID, 36),
        (protocol_r1::CreateObjectRpc::ID, 44),
        (protocol_r1::SpawnInfoRpc::ID, 68),
        (protocol_r1::InitMenuRpc::ID, 76),
        (protocol_r1::InterpolateCameraRpc::ID, 82),
        (protocol_r1::ToggleSelectTextDrawRpc::ID, 83),
        (protocol_r1::SetObjectMaterialRpc::ID, 84),
        (protocol_r1::PlayerAnimationRpc::ID, 86),
        (protocol_r1::EnableStuntBonusRpc::ID, 104),
        (protocol_r1::CrimeReportRpc::ID, 112),
        (protocol_r1::PlayerAttachedObjectRpc::ID, 113),
        (protocol_r1::EnterEditObjectRpc::ID, 117),
        (protocol_r1::TogglePlayerSpectatingRpc::ID, 124),
        (protocol_r1::ShowTextDrawRpc::ID, 134),
        (protocol_r1::TextDrawHideRpc::ID, 135),
        (protocol_r1::ScoresAndPingsRpc::ID, 155),
        (protocol_r1::VehicleStreamInRpc::ID, 164),
        (protocol_r1::DisableVehicleCollisionsRpc::ID, 167),
        (protocol_r1::ToggleCameraTargetNotifyingRpc::ID, 170),
        (protocol_r1::ApplyActorAnimationRpc::ID, 173),
    ];
    for (actual, expected) in descriptors {
        assert_eq!(actual, expected);
    }
}
