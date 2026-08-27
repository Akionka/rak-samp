mod fixed;
mod r1;
mod types;

pub use fixed::*;
pub use r1::*;
pub use types::*;

use crate::events::core::handle;
use crate::events::{EventError, ProtocolAction};
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
