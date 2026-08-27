mod fixed;
mod r1;
mod types;

pub use fixed::*;
pub use r1::*;
pub use types::*;

use crate::events::core::handle;
use crate::events::{EventError, ProtocolAction, Vector3};
use crate::{HostApi, SampClientSdkEventV1, SampClientSdkHookAction};

macro_rules! rpc_helper {
    ($name:ident, $value:ty, $rpc:ident, $event_name:literal) => {
        #[doc = concat!("Handles MoonLoader's `", $event_name, "` from an incoming raw RPC callback.")]
        ///
        /// # Safety
        ///
        /// See [`crate::events::handle`].
        #[allow(dead_code)]
        pub(crate) unsafe fn $name(
            api: HostApi,
            raw: *mut SampClientSdkEventV1,
            handler: impl FnOnce($value) -> ProtocolAction<$value>,
        ) -> Result<SampClientSdkHookAction, EventError> {
            unsafe { handle(api, raw, $rpc, handler) }
        }
    };
}

rpc_helper!(on_show_dialog, ShowDialog, SHOW_DIALOG, "onShowDialog");
rpc_helper!(
    on_attach_camera_to_object,
    u16,
    ATTACH_CAMERA_TO_OBJECT,
    "onAttachCameraToObject"
);
rpc_helper!(
    on_gang_zone_stop_flash,
    u16,
    GANG_ZONE_STOP_FLASH,
    "onGangZoneStopFlash"
);
rpc_helper!(
    on_clear_player_animation,
    u16,
    CLEAR_PLAYER_ANIMATION,
    "onClearPlayerAnimation"
);
rpc_helper!(
    on_set_player_special_action,
    u8,
    SET_PLAYER_SPECIAL_ACTION,
    "onSetPlayerSpecialAction"
);
rpc_helper!(
    on_set_player_fighting_style,
    PlayerFightingStyle,
    SET_PLAYER_FIGHTING_STYLE,
    "onSetPlayerFightingStyle"
);
rpc_helper!(
    on_set_player_velocity,
    Vector3,
    SET_PLAYER_VELOCITY,
    "onSetPlayerVelocity"
);
rpc_helper!(
    on_set_vehicle_velocity,
    VehicleVelocity,
    SET_VEHICLE_VELOCITY,
    "onSetVehicleVelocity"
);
rpc_helper!(on_create_pickup, Pickup, CREATE_PICKUP, "onCreatePickup");
rpc_helper!(on_move_object, MoveObject, MOVE_OBJECT, "onMoveObject");
rpc_helper!(
    on_text_draw_set_string,
    TextDrawString,
    TEXT_DRAW_SET_STRING,
    "onTextDrawSetString"
);
rpc_helper!(
    on_create_gang_zone,
    GangZone,
    CREATE_GANG_ZONE,
    "onCreateGangZone"
);
rpc_helper!(
    on_gang_zone_destroy,
    u16,
    GANG_ZONE_DESTROY,
    "onGangZoneDestroy"
);
rpc_helper!(
    on_gang_zone_flash,
    (u16, i32),
    GANG_ZONE_FLASH,
    "onGangZoneFlash"
);
rpc_helper!(on_stop_object, u16, STOP_OBJECT, "onStopObject");
rpc_helper!(
    on_set_vehicle_number_plate,
    VehicleNumberPlate,
    SET_VEHICLE_NUMBER_PLATE,
    "onSetVehicleNumberPlate"
);
rpc_helper!(
    on_spectate_player,
    Spectate,
    SPECTATE_PLAYER,
    "onSpectatePlayer"
);
rpc_helper!(
    on_spectate_vehicle,
    Spectate,
    SPECTATE_VEHICLE,
    "onSpectateVehicle"
);
rpc_helper!(
    on_connection_rejected,
    u8,
    CONNECTION_REJECTED,
    "onConnectionRejected"
);
rpc_helper!(on_remove_map_icon, u8, REMOVE_MAP_ICON, "onRemoveMapIcon");
rpc_helper!(
    on_set_weapon_ammo,
    WeaponAmmo,
    SET_WEAPON_AMMO,
    "onSetWeaponAmmo"
);
rpc_helper!(on_set_gravity, f32, SET_GRAVITY, "onSetGravity");
rpc_helper!(
    on_attach_trailer_to_vehicle,
    TrailerAttachment,
    ATTACH_TRAILER_TO_VEHICLE,
    "onAttachTrailerToVehicle"
);
rpc_helper!(
    on_detach_trailer_from_vehicle,
    u16,
    DETACH_TRAILER_FROM_VEHICLE,
    "onDetachTrailerFromVehicle"
);
rpc_helper!(
    on_set_camera_position,
    Vector3,
    SET_CAMERA_POSITION,
    "onSetCameraPosition"
);
rpc_helper!(
    on_set_camera_look_at,
    CameraLookAt,
    SET_CAMERA_LOOK_AT,
    "onSetCameraLookAt"
);
rpc_helper!(
    on_set_vehicle_params,
    VehicleParams,
    SET_VEHICLE_PARAMS,
    "onSetVehicleParams"
);
rpc_helper!(on_player_death, u16, PLAYER_DEATH, "onPlayerDeath");
rpc_helper!(
    on_player_enter_vehicle,
    PlayerEnterVehicle,
    PLAYER_ENTER_VEHICLE,
    "onPlayerEnterVehicle"
);
rpc_helper!(
    on_player_exit_vehicle,
    PlayerExitVehicle,
    PLAYER_EXIT_VEHICLE,
    "onPlayerExitVehicle"
);
rpc_helper!(on_init_game, InitGame, INIT_GAME, "onInitGame");
rpc_helper!(
    on_request_class_response,
    RequestClassResponse,
    REQUEST_CLASS_RESPONSE,
    "onRequestClassResponse"
);
rpc_helper!(
    on_player_stream_in,
    PlayerStreamIn,
    PLAYER_STREAM_IN,
    "onPlayerStreamIn"
);
rpc_helper!(
    on_create_3d_text,
    TextLabel3D,
    CREATE_3D_TEXT,
    "onCreate3DText"
);
rpc_helper!(on_create_object, Object, CREATE_OBJECT, "onCreateObject");
rpc_helper!(
    on_set_spawn_info,
    SpawnInfo,
    SET_SPAWN_INFO,
    "onSetSpawnInfo"
);
rpc_helper!(on_init_menu, InitMenu, INIT_MENU, "onInitMenu");
rpc_helper!(
    on_interpolate_camera,
    InterpolateCamera,
    INTERPOLATE_CAMERA,
    "onInterpolateCamera"
);
rpc_helper!(
    on_toggle_select_text_draw,
    ToggleSelectTextDraw,
    TOGGLE_SELECT_TEXT_DRAW,
    "onToggleSelectTextDraw"
);
rpc_helper!(
    on_set_object_material,
    ObjectMaterialUpdate,
    SET_OBJECT_MATERIAL,
    "onSetObjectMaterial/onSetObjectMaterialText"
);
rpc_helper!(
    on_apply_player_animation,
    PlayerAnimation,
    APPLY_PLAYER_ANIMATION,
    "onApplyPlayerAnimation"
);
rpc_helper!(
    on_enable_stunt_bonus,
    bool,
    ENABLE_STUNT_BONUS,
    "onEnableStuntBonus"
);
rpc_helper!(
    on_play_crime_report,
    CrimeReport,
    PLAY_CRIME_REPORT,
    "onPlayCrimeReport"
);
rpc_helper!(
    on_set_player_attached_object,
    PlayerAttachedObject,
    SET_PLAYER_ATTACHED_OBJECT,
    "onSetPlayerAttachedObject"
);
rpc_helper!(
    on_enter_edit_object,
    EnterEditObject,
    ENTER_EDIT_OBJECT,
    "onEnterEditObject"
);
rpc_helper!(
    on_toggle_player_spectating,
    bool,
    TOGGLE_PLAYER_SPECTATING,
    "onTogglePlayerSpectating"
);
rpc_helper!(
    on_show_text_draw,
    ShowTextDraw,
    SHOW_TEXT_DRAW,
    "onShowTextDraw"
);
rpc_helper!(on_text_draw_hide, u16, TEXT_DRAW_HIDE, "onTextDrawHide");
rpc_helper!(
    on_update_scores_and_pings,
    ScoresAndPings,
    UPDATE_SCORES_AND_PINGS,
    "onUpdateScoresAndPings"
);
rpc_helper!(
    on_vehicle_stream_in,
    VehicleStreamIn,
    VEHICLE_STREAM_IN,
    "onVehicleStreamIn"
);
rpc_helper!(
    on_disable_vehicle_collisions,
    bool,
    DISABLE_VEHICLE_COLLISIONS,
    "onDisableVehicleCollisions"
);
rpc_helper!(
    on_toggle_camera_target_notifying,
    bool,
    TOGGLE_CAMERA_TARGET_NOTIFYING,
    "onToggleCameraTargetNotifying"
);
rpc_helper!(
    on_apply_actor_animation,
    ActorAnimation,
    APPLY_ACTOR_ANIMATION,
    "onApplyActorAnimation"
);
