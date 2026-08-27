mod fixed;
mod r1;
mod types;

pub use fixed::*;
pub use r1::*;
pub use types::*;

use crate::events::core::{ProtocolEventError, handle, handle_protocol};
use crate::events::{Event, EventError, ProtocolAction};
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

macro_rules! protocol_rpc_helper {
    ($name:ident, $descriptor:path, $value:ty, $event_name:literal) => {
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
        ) -> Result<SampClientSdkHookAction, ProtocolEventError> {
            let mut event = unsafe { Event::from_callback(api, raw) }
                .map_err(|error| ProtocolEventError::DecodeSource(error))?;
            handle_protocol::<$descriptor>(&mut event, handler)
        }
    };
}

rpc_helper!(on_show_dialog, ShowDialog, SHOW_DIALOG, "onShowDialog");
protocol_rpc_helper!(
    on_init_game,
    samp_protocol::rpc::incoming::r1::InitGameRpc,
    samp_protocol::rpc::incoming::r1::InitGame,
    "onInitGame"
);
protocol_rpc_helper!(
    on_request_class_response,
    samp_protocol::rpc::incoming::r1::RequestClassResponseRpc,
    samp_protocol::rpc::incoming::r1::RequestClassResponse,
    "onRequestClassResponse"
);
protocol_rpc_helper!(
    on_player_stream_in,
    samp_protocol::rpc::incoming::r1::PlayerStreamInRpc,
    samp_protocol::rpc::incoming::r1::PlayerStreamIn,
    "onPlayerStreamIn"
);
rpc_helper!(
    on_create_3d_text,
    TextLabel3D,
    CREATE_3D_TEXT,
    "onCreate3DText"
);
rpc_helper!(on_create_object, Object, CREATE_OBJECT, "onCreateObject");
protocol_rpc_helper!(
    on_set_spawn_info,
    samp_protocol::rpc::incoming::r1::SpawnInfoRpc,
    samp_protocol::rpc::incoming::r1::SpawnInfo,
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
protocol_rpc_helper!(
    on_apply_player_animation,
    samp_protocol::rpc::incoming::r1::PlayerAnimationRpc,
    samp_protocol::rpc::incoming::r1::PlayerAnimation,
    "onApplyPlayerAnimation"
);
protocol_rpc_helper!(
    on_enable_stunt_bonus,
    samp_protocol::rpc::incoming::r1::EnableStuntBonusRpc,
    bool,
    "onEnableStuntBonus"
);
protocol_rpc_helper!(
    on_play_crime_report,
    samp_protocol::rpc::incoming::r1::CrimeReportRpc,
    samp_protocol::rpc::incoming::r1::CrimeReport,
    "onPlayCrimeReport"
);
protocol_rpc_helper!(
    on_set_player_attached_object,
    samp_protocol::rpc::incoming::r1::PlayerAttachedObjectRpc,
    samp_protocol::rpc::incoming::r1::PlayerAttachedObject,
    "onSetPlayerAttachedObject"
);
rpc_helper!(
    on_enter_edit_object,
    EnterEditObject,
    ENTER_EDIT_OBJECT,
    "onEnterEditObject"
);
protocol_rpc_helper!(
    on_toggle_player_spectating,
    samp_protocol::rpc::incoming::r1::TogglePlayerSpectatingRpc,
    bool,
    "onTogglePlayerSpectating"
);
rpc_helper!(
    on_show_text_draw,
    ShowTextDraw,
    SHOW_TEXT_DRAW,
    "onShowTextDraw"
);
rpc_helper!(on_text_draw_hide, u16, TEXT_DRAW_HIDE, "onTextDrawHide");
protocol_rpc_helper!(
    on_update_scores_and_pings,
    samp_protocol::rpc::incoming::r1::ScoresAndPingsRpc,
    samp_protocol::rpc::incoming::r1::ScoresAndPings,
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
