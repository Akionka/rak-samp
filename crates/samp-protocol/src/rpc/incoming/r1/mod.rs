//! R1 incoming RPC codecs.

mod wire;

macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ident, $value:ty, $policy:ident) => {
        $crate::wire::nominal_descriptor!(
            incoming rpc,
            $name,
            $constant,
            $id,
            $codec,
            $value,
            $crate::$policy
        );
    };
}

macro_rules! r1_codec {
    ($codec:ident, $value:ty, $decode:ident, $encode:ident) => {
        impl $crate::WireCodec for $codec {
            type Value = $value;
            fn decode<R: $crate::BitRead>(
                reader: &mut R,
            ) -> Result<Self::Value, $crate::DecodeError<R::Error>> {
                $decode(reader)
            }

            fn encode<W: $crate::BitWrite>(
                writer: &mut W,
                value: &Self::Value,
            ) -> Result<(), $crate::EncodeError<W::Error>> {
                $encode(writer, value)
            }
        }
    };
}

macro_rules! encoded_string_rpc_descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ty, $value:ty) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct $name;

        pub const $constant: $name = $name;

        impl $crate::encoded_string::sealed::EncodedStringWireDescriptor<$value> for $name {
            fn decode<R: $crate::EncodedStringRead>(
                reader: &mut R,
            ) -> Result<$value, $crate::DecodeError<R::Error>> {
                <$codec as $crate::EncodedStringWireCodec>::decode(reader)
            }

            fn encode<W: $crate::EncodedStringWrite>(
                writer: &mut W,
                value: &$value,
            ) -> Result<(), $crate::EncodeError<W::Error>> {
                <$codec as $crate::EncodedStringWireCodec>::encode(writer, value)
            }
        }

        impl $crate::EncodedStringWireDescriptor for $name {
            type Value = $value;

            const ID: u8 = $id;
            const KIND: $crate::WireKind = $crate::WireKind::Rpc;
            const TRAILING_POLICY: $crate::TrailingPolicy = $crate::TrailingPolicy::ExactBits;
        }

        impl $crate::wire::sealed::IncomingRpcDescriptor for $name {}

        impl $crate::IncomingRpcDescriptor for $name {
            type Value = $value;
            type Capability = $crate::EncodedStringWire;

            const ID: u8 = $id;
        }
    };
}

mod text_labels;

pub use text_labels::{CREATE_3D_TEXT, Create3DTextRpc, TextLabel3D};
mod object;

pub use object::{
    CREATE_OBJECT, CreateObjectRpc, ENTER_EDIT_OBJECT, EnterEditObject, EnterEditObjectRpc,
    MAX_OBJECT_MATERIAL_TEXT_BYTES, MAX_OBJECT_MATERIALS, Object, ObjectAttachment, ObjectMaterial,
    ObjectMaterialUpdate, SET_OBJECT_MATERIAL, SetObjectMaterialRpc, TextMaterial, TextureMaterial,
};
mod camera;

pub use camera::{
    INTERPOLATE_CAMERA, InterpolateCamera, InterpolateCameraRpc, TOGGLE_CAMERA_TARGET_NOTIFYING,
    ToggleCameraTargetNotifyingRpc,
};
mod ui;

pub use ui::{
    INIT_MENU, InitMenu, InitMenuRpc, MAX_MENU_COLUMNS, MAX_MENU_ROWS, MenuColumn, SHOW_TEXT_DRAW,
    ShowTextDraw, ShowTextDrawRpc, TEXT_DRAW_HIDE, TOGGLE_SELECT_TEXT_DRAW, TextDraw,
    TextDrawHideRpc, ToggleSelectTextDraw, ToggleSelectTextDrawRpc,
};
mod vehicle;

pub use vehicle::{
    DISABLE_VEHICLE_COLLISIONS, DisableVehicleCollisionsRpc, StreamedVehicle, VEHICLE_STREAM_IN,
    VehicleStreamIn, VehicleStreamInRpc,
};
mod session;

pub use session::{
    ENABLE_STUNT_BONUS, EnableStuntBonusRpc, GameSettings, INIT_GAME, InitGame, InitGameRpc,
    MAX_SCORE_PING_ENTRIES, REQUEST_CLASS_RESPONSE, RequestClassResponse, RequestClassResponseRpc,
    SET_SPAWN_INFO, ScorePing, ScoresAndPings, ScoresAndPingsRpc, SpawnInfo, SpawnInfoRpc,
    UPDATE_SCORES_AND_PINGS,
};
mod actor;

pub use actor::{APPLY_ACTOR_ANIMATION, ActorAnimation, ApplyActorAnimationRpc};
mod player;

pub use player::{
    APPLY_PLAYER_ANIMATION, Animation, AttachedObject, CrimeReport, CrimeReportRpc,
    PLAY_CRIME_REPORT, PLAYER_STREAM_IN, PlayerAnimation, PlayerAnimationRpc, PlayerAttachedObject,
    PlayerAttachedObjectRpc, PlayerStreamIn, PlayerStreamInRpc, SET_PLAYER_ATTACHED_OBJECT,
    TOGGLE_PLAYER_SPECTATING, TogglePlayerSpectatingRpc,
};
