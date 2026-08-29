//! R1 incoming RPC codecs.

mod wire;

use wire::{
    decode_bit_bool, decode_bool32, encode_bit_bool, encode_bool32, read_bool8, read_bool32,
    read_fixed, write_bool8, write_bool32,
};

use crate::limits::{MAX_ENCODED_STRING_BYTES, MAX_STRING32_BYTES};
use crate::types::{Vector2, Vector3};
use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, EncodedStringRead, EncodedStringWireCodec,
    EncodedStringWireDescriptor, EncodedStringWrite, ExactBitsPolicy, ExactBytesPolicy,
    TrailingPolicy, WireCodec, WireKind, WireReadExt, WireWriteExt,
    encoded_string::{read_encoded_string, write_encoded_string},
};

macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ident, $value:ty, $policy:ident) => {
        crate::wire::nominal_descriptor!(
            incoming rpc,
            $name,
            $constant,
            $id,
            $codec,
            $value,
            $policy
        );
    };
}

macro_rules! r1_codec {
    ($codec:ident, $value:ty, $decode:ident, $encode:ident) => {
        impl WireCodec for $codec {
            type Value = $value;
            fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
                $decode(reader)
            }

            fn encode<W: BitWrite>(
                writer: &mut W,
                value: &Self::Value,
            ) -> Result<(), EncodeError<W::Error>> {
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

        impl crate::encoded_string::sealed::EncodedStringWireDescriptor<$value> for $name {
            fn decode<R: EncodedStringRead>(
                reader: &mut R,
            ) -> Result<$value, DecodeError<R::Error>> {
                <$codec as EncodedStringWireCodec>::decode(reader)
            }

            fn encode<W: EncodedStringWrite>(
                writer: &mut W,
                value: &$value,
            ) -> Result<(), EncodeError<W::Error>> {
                <$codec as EncodedStringWireCodec>::encode(writer, value)
            }
        }

        impl EncodedStringWireDescriptor for $name {
            type Value = $value;

            const ID: u8 = $id;
            const KIND: WireKind = WireKind::Rpc;
            const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBits;
        }

        impl crate::wire::sealed::IncomingRpcDescriptor for $name {}

        impl crate::IncomingRpcDescriptor for $name {
            type Value = $value;
            type Capability = crate::EncodedStringWire;

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

use player::{decode_animation, encode_animation};
