//! Outgoing vehicle, player, and actor damage RPC codecs.

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{EncodedPayload, Event, EventError, Rpc, RpcAction},
};

/// MoonLoader's `onSendVehicleDamaged` payload (RPC 106).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleDamage {
    pub vehicle_id: u16,
    pub panel_damage: i32,
    pub door_damage: i32,
    pub lights: u8,
    pub tires: u8,
}

/// MoonLoader's shared `onSendGiveDamage` / `onSendTakeDamage` payload (RPC 115).
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

/// MoonLoader's `onSendGiveActorDamage` payload (RPC 177).
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

pub const SEND_VEHICLE_DAMAGED: Rpc<VehicleDamage> =
    Rpc::new(106, decode_vehicle_damage, encode_vehicle_damage);
pub const SEND_DAMAGE: Rpc<Damage> = Rpc::new_bits(115, decode_damage, encode_damage);
pub const SEND_GIVE_ACTOR_DAMAGE: Rpc<ActorDamage> =
    Rpc::new_bits(177, decode_actor_damage, encode_actor_damage);

#[allow(dead_code)]
pub(crate) unsafe fn on_send_vehicle_damaged(
    api: HostApi,
    raw: *mut SampClientSdkEventV1,
    handler: impl FnOnce(VehicleDamage) -> RpcAction<VehicleDamage>,
) -> Result<SampClientSdkHookAction, EventError> {
    unsafe { handle(api, raw, SEND_VEHICLE_DAMAGED, handler) }
}
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

fn decode_vehicle_damage(event: &mut Event<'_>) -> Result<VehicleDamage, EventError> {
    Ok(VehicleDamage {
        vehicle_id: event.read_u16()?,
        panel_damage: event.read_u32()? as i32,
        door_damage: event.read_u32()? as i32,
        lights: event.read_u8()?,
        tires: event.read_u8()?,
    })
}
fn encode_vehicle_damage(value: VehicleDamage) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u32(value.panel_damage as u32);
    writer.u32(value.door_damage as u32);
    writer.u8(value.lights);
    writer.u8(value.tires);
    Ok(writer.finish())
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
