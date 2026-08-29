use super::player::{Animation, decode_animation, encode_animation};
use crate::{BitRead, BitWrite, DecodeError, EncodeError, WireReadExt, WireWriteExt};

/// R1's `onApplyActorAnimation` payload (RPC 173).
#[derive(Clone, Debug, PartialEq)]
pub struct ActorAnimation {
    pub actor_id: u16,
    pub animation: Animation,
}

struct ActorAnimationCodec;

descriptor!(
    ApplyActorAnimationRpc,
    APPLY_ACTOR_ANIMATION,
    173,
    ActorAnimationCodec,
    ActorAnimation,
    ExactBitsPolicy
);

r1_codec!(
    ActorAnimationCodec,
    ActorAnimation,
    decode_actor_animation,
    encode_actor_animation
);

fn decode_actor_animation<R: BitRead>(
    reader: &mut R,
) -> Result<ActorAnimation, DecodeError<R::Error>> {
    Ok(ActorAnimation {
        actor_id: reader.read_u16_le()?,
        animation: decode_animation(reader)?,
    })
}

fn encode_actor_animation<W: BitWrite>(
    writer: &mut W,
    value: &ActorAnimation,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.actor_id)?;
    encode_animation(writer, &value.animation)
}
