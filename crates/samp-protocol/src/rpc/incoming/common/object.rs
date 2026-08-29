use super::*;

/// MoonLoader's `onSetObjectPosition` payload (RPC 45).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectPosition {
    pub object_id: u16,
    pub position: Vector3,
}

/// MoonLoader's `onSetObjectRotation` payload (RPC 46).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ObjectRotation {
    pub object_id: u16,
    pub rotation: Vector3,
}

/// MoonLoader's `onAttachObjectToPlayer` payload (RPC 75).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttachObjectToPlayer {
    pub object_id: u16,
    pub player_id: u16,
    pub offsets: Vector3,
    pub rotation: Vector3,
}

/// MoonLoader's `onMoveObject` payload (RPC 99).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveObject {
    pub object_id: u16,
    pub from_position: Vector3,
    pub destination: Vector3,
    pub speed: f32,
    pub rotation: Vector3,
}

struct ObjectPositionCodec;

struct ObjectRotationCodec;

struct AttachObjectToPlayerCodec;

struct MoveObjectCodec;

descriptor!(CancelEdit, CANCEL_EDIT, 28, Empty, ());

descriptor!(
    SetObjectPosition,
    SET_OBJECT_POSITION,
    45,
    ObjectPositionCodec,
    ObjectPosition
);

descriptor!(
    SetObjectRotation,
    SET_OBJECT_ROTATION,
    46,
    ObjectRotationCodec,
    ObjectRotation
);

descriptor!(DestroyObject, DESTROY_OBJECT, 47, U16, u16);

descriptor!(
    AttachObjectToPlayerRpc,
    ATTACH_OBJECT_TO_PLAYER,
    75,
    AttachObjectToPlayerCodec,
    AttachObjectToPlayer
);

descriptor!(EditAttachedObject, EDIT_ATTACHED_OBJECT, 116, I32, i32);

descriptor!(EnterSelectObject, ENTER_SELECT_OBJECT, 27, Empty, ());

descriptor!(
    SetPlayerObjectNoCameraCol,
    SET_PLAYER_OBJECT_NO_CAMERA_COL,
    169,
    U16,
    u16
);

descriptor!(MoveObjectRpc, MOVE_OBJECT, 99, MoveObjectCodec, MoveObject);

descriptor!(StopObject, STOP_OBJECT, 122, U16, u16);

wire_codec!(
    ObjectPositionCodec,
    ObjectPosition,
    read_object_position,
    write_object_position
);

wire_codec!(
    ObjectRotationCodec,
    ObjectRotation,
    read_object_rotation,
    write_object_rotation
);

wire_codec!(
    AttachObjectToPlayerCodec,
    AttachObjectToPlayer,
    read_attach_object_to_player,
    write_attach_object_to_player
);

wire_codec!(
    MoveObjectCodec,
    MoveObject,
    read_move_object,
    write_move_object
);

fn read_object_position<R: BitRead>(
    reader: &mut R,
) -> Result<ObjectPosition, DecodeError<R::Error>> {
    Ok(ObjectPosition {
        object_id: reader.read_u16_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_object_position<W: BitWrite>(
    writer: &mut W,
    value: &ObjectPosition,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.object_id)?;
    writer.write_vector3_le(&value.position)
}

fn read_object_rotation<R: BitRead>(
    reader: &mut R,
) -> Result<ObjectRotation, DecodeError<R::Error>> {
    Ok(ObjectRotation {
        object_id: reader.read_u16_le()?,
        rotation: reader.read_vector3_le()?,
    })
}

fn write_object_rotation<W: BitWrite>(
    writer: &mut W,
    value: &ObjectRotation,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.object_id)?;
    writer.write_vector3_le(&value.rotation)
}

fn read_attach_object_to_player<R: BitRead>(
    reader: &mut R,
) -> Result<AttachObjectToPlayer, DecodeError<R::Error>> {
    Ok(AttachObjectToPlayer {
        object_id: reader.read_u16_le()?,
        player_id: reader.read_u16_le()?,
        offsets: reader.read_vector3_le()?,
        rotation: reader.read_vector3_le()?,
    })
}

fn write_attach_object_to_player<W: BitWrite>(
    writer: &mut W,
    value: &AttachObjectToPlayer,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.object_id)?;
    writer.write_u16_le(value.player_id)?;
    writer.write_vector3_le(&value.offsets)?;
    writer.write_vector3_le(&value.rotation)
}

fn read_move_object<R: BitRead>(reader: &mut R) -> Result<MoveObject, DecodeError<R::Error>> {
    Ok(MoveObject {
        object_id: reader.read_u16_le()?,
        from_position: reader.read_vector3_le()?,
        destination: reader.read_vector3_le()?,
        speed: reader.read_f32_le()?,
        rotation: reader.read_vector3_le()?,
    })
}

fn write_move_object<W: BitWrite>(
    writer: &mut W,
    value: &MoveObject,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.object_id)?;
    writer.write_vector3_le(&value.from_position)?;
    writer.write_vector3_le(&value.destination)?;
    writer.write_f32_le(value.speed)?;
    writer.write_vector3_le(&value.rotation)
}
