use super::*;

/// MoonLoader's `onCreateActor` payload (RPC 171).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Actor {
    pub actor_id: u16,
    pub skin_id: i32,
    pub position: Vector3,
    pub rotation: f32,
    pub health: f32,
}

/// MoonLoader's `onSetActorFacingAngle` payload (RPC 175).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorAngle {
    pub actor_id: u16,
    pub angle: f32,
}

/// MoonLoader's `onSetActorPos` payload (RPC 176).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorPosition {
    pub actor_id: u16,
    pub position: Vector3,
}

/// MoonLoader's `onSetActorHealth` payload (RPC 178).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorHealth {
    pub actor_id: u16,
    pub health: f32,
}

struct ActorCodec;

struct ActorAngleCodec;

struct ActorPositionCodec;

struct ActorHealthCodec;

descriptor!(DestroyActor, DESTROY_ACTOR, 172, U16, u16);

descriptor!(CreateActor, CREATE_ACTOR, 171, ActorCodec, Actor);

descriptor!(ClearActorAnimation, CLEAR_ACTOR_ANIMATION, 174, U16, u16);

descriptor!(
    SetActorFacingAngle,
    SET_ACTOR_FACING_ANGLE,
    175,
    ActorAngleCodec,
    ActorAngle
);

descriptor!(
    SetActorPosition,
    SET_ACTOR_POSITION,
    176,
    ActorPositionCodec,
    ActorPosition
);

descriptor!(
    SetActorHealth,
    SET_ACTOR_HEALTH,
    178,
    ActorHealthCodec,
    ActorHealth
);

wire_codec!(ActorCodec, Actor, read_actor, write_actor);

wire_codec!(
    ActorAngleCodec,
    ActorAngle,
    read_actor_angle,
    write_actor_angle
);

wire_codec!(
    ActorPositionCodec,
    ActorPosition,
    read_actor_position,
    write_actor_position
);

wire_codec!(
    ActorHealthCodec,
    ActorHealth,
    read_actor_health,
    write_actor_health
);

fn read_actor<R: BitRead>(reader: &mut R) -> Result<Actor, DecodeError<R::Error>> {
    Ok(Actor {
        actor_id: reader.read_u16_le()?,
        skin_id: reader.read_i32_le()?,
        position: reader.read_vector3_le()?,
        rotation: reader.read_f32_le()?,
        health: reader.read_f32_le()?,
    })
}

fn write_actor<W: BitWrite>(writer: &mut W, value: &Actor) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.actor_id)?;
    writer.write_i32_le(value.skin_id)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_f32_le(value.rotation)?;
    writer.write_f32_le(value.health)
}

fn read_actor_angle<R: BitRead>(reader: &mut R) -> Result<ActorAngle, DecodeError<R::Error>> {
    Ok(ActorAngle {
        actor_id: reader.read_u16_le()?,
        angle: reader.read_f32_le()?,
    })
}

fn write_actor_angle<W: BitWrite>(
    writer: &mut W,
    value: &ActorAngle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.actor_id)?;
    writer.write_f32_le(value.angle)
}

fn read_actor_position<R: BitRead>(reader: &mut R) -> Result<ActorPosition, DecodeError<R::Error>> {
    Ok(ActorPosition {
        actor_id: reader.read_u16_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_actor_position<W: BitWrite>(
    writer: &mut W,
    value: &ActorPosition,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.actor_id)?;
    writer.write_vector3_le(&value.position)
}

fn read_actor_health<R: BitRead>(reader: &mut R) -> Result<ActorHealth, DecodeError<R::Error>> {
    Ok(ActorHealth {
        actor_id: reader.read_u16_le()?,
        health: reader.read_f32_le()?,
    })
}

fn write_actor_health<W: BitWrite>(
    writer: &mut W,
    value: &ActorHealth,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.actor_id)?;
    writer.write_f32_le(value.health)
}
