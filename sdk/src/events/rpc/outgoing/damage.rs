//! Exact-bit outgoing player and actor damage OutgoingRpc codecs.

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{EncodedPayload, Event, EventError, OutgoingRpc, RpcAction},
};

/// MoonLoader's shared `onSendGiveDamage` / `onSendTakeDamage` payload (OutgoingRpc 115).
///
/// `take` is a one-bit RakNet boolean. `false` identifies give-damage traffic and `true`
/// identifies take-damage traffic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Damage {
    pub player_id: u16,
    pub damage: f32,
    pub weapon: i32,
    pub body_part: i32,
    pub take: bool,
}

/// MoonLoader's `onSendGiveActorDamage` payload (OutgoingRpc 177).
///
/// `unused` is a one-bit RakNet boolean retained for wire compatibility.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorDamage {
    pub unused: bool,
    pub actor_id: u16,
    pub damage: f32,
    pub weapon: i32,
    pub body_part: i32,
}

pub const SEND_DAMAGE: OutgoingRpc<Damage> =
    OutgoingRpc::new_bits(115, decode_damage, encode_damage);
pub const SEND_GIVE_ACTOR_DAMAGE: OutgoingRpc<ActorDamage> =
    OutgoingRpc::new_bits(177, decode_actor_damage, encode_actor_damage);

#[allow(dead_code)]
pub(crate) unsafe fn on_send_give_actor_damage(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(ActorDamage) -> RpcAction<ActorDamage>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_GIVE_ACTOR_DAMAGE, handler) }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_give_damage(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(Damage) -> RpcAction<Damage>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe {
        handle(api, raw, SEND_DAMAGE, |value| {
            if value.take {
                RpcAction::Continue
            } else {
                handler(value)
            }
        })
    }
}

#[allow(dead_code)]
pub(crate) unsafe fn on_send_take_damage(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(Damage) -> RpcAction<Damage>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe {
        handle(api, raw, SEND_DAMAGE, |value| {
            if value.take {
                handler(value)
            } else {
                RpcAction::Continue
            }
        })
    }
}

fn decode_damage(event: &mut Event<'_>) -> Result<Damage, EventError> {
    Ok(Damage {
        take: event.read_bits(1)?[0] & 0x80 != 0,
        player_id: event.read_u16()?,
        damage: event.read_f32()?,
        weapon: event.read_u32()? as i32,
        body_part: event.read_u32()? as i32,
    })
}

fn encode_damage(_api: HostApi, value: Damage) -> Result<EncodedPayload, EventError> {
    encode_damage_payload(value)
}

pub fn encode_damage_payload(value: Damage) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.bit(value.take);
    writer.u16(value.player_id);
    writer.f32(value.damage);
    writer.u32(value.weapon as u32);
    writer.u32(value.body_part as u32);
    Ok(writer.finish_bits())
}

fn decode_actor_damage(event: &mut Event<'_>) -> Result<ActorDamage, EventError> {
    Ok(ActorDamage {
        unused: event.read_bits(1)?[0] & 0x80 != 0,
        actor_id: event.read_u16()?,
        damage: event.read_f32()?,
        weapon: event.read_u32()? as i32,
        body_part: event.read_u32()? as i32,
    })
}

fn encode_actor_damage(_api: HostApi, value: ActorDamage) -> Result<EncodedPayload, EventError> {
    let mut writer = PayloadWriter::new();
    writer.bit(value.unused);
    writer.u16(value.actor_id);
    writer.f32(value.damage);
    writer.u32(value.weapon as u32);
    writer.u32(value.body_part as u32);
    Ok(writer.finish_bits())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::test_support::test_api;

    #[test]
    fn outgoing_damage_keeps_its_one_bit_boolean_and_exact_payload_length() {
        let payload = SEND_DAMAGE
            .encode(
                test_api(),
                Damage {
                    player_id: 0x1234,
                    damage: 1.0,
                    weapon: 24,
                    body_part: 9,
                    take: true,
                },
            )
            .expect("damage payload must encode");

        assert_eq!(payload.len_bits(), 113);
        assert_eq!(
            payload.as_bytes(),
            [
                0x9A, 0x09, 0x00, 0x00, 0x40, 0x1F, 0x8C, 0x00, 0x00, 0x00, 0x04, 0x80, 0x00, 0x00,
                0x00,
            ]
        );
    }
}
