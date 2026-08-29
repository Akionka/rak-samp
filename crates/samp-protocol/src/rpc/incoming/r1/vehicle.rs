use super::*;

/// R1's `onVehicleStreamIn` vehicle data (RPC 164).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StreamedVehicle {
    pub model: i32,
    pub position: Vector3,
    pub rotation: f32,
    pub body_color1: u8,
    pub body_color2: u8,
    pub health: f32,
    pub interior_id: u8,
    pub door_damage_status: i32,
    pub panel_damage_status: i32,
    pub light_damage_status: u8,
    pub tire_damage_status: u8,
    pub add_siren: u8,
    pub mod_slots: [u8; 14],
    pub paint_job: u8,
    pub interior_color1: i32,
    pub interior_color2: i32,
}

/// R1's `onVehicleStreamIn` payload (RPC 164).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleStreamIn {
    pub vehicle_id: u16,
    pub vehicle: StreamedVehicle,
}

struct VehicleStreamInCodec;

struct DisableVehicleCollisionsCodec;

descriptor!(
    VehicleStreamInRpc,
    VEHICLE_STREAM_IN,
    164,
    VehicleStreamInCodec,
    VehicleStreamIn,
    ExactBytesPolicy
);

descriptor!(
    DisableVehicleCollisionsRpc,
    DISABLE_VEHICLE_COLLISIONS,
    167,
    DisableVehicleCollisionsCodec,
    bool,
    ExactBitsPolicy
);

r1_codec!(
    VehicleStreamInCodec,
    VehicleStreamIn,
    decode_vehicle_stream_in,
    encode_vehicle_stream_in
);

r1_codec!(
    DisableVehicleCollisionsCodec,
    bool,
    decode_bit_bool,
    encode_bit_bool
);

fn decode_vehicle_stream_in<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleStreamIn, DecodeError<R::Error>> {
    Ok(VehicleStreamIn {
        vehicle_id: reader.read_u16_le()?,
        vehicle: StreamedVehicle {
            model: reader.read_i32_le()?,
            position: reader.read_vector3_le()?,
            rotation: reader.read_f32_le()?,
            body_color1: reader.read_u8()?,
            body_color2: reader.read_u8()?,
            health: reader.read_f32_le()?,
            interior_id: reader.read_u8()?,
            door_damage_status: reader.read_i32_le()?,
            panel_damage_status: reader.read_i32_le()?,
            light_damage_status: reader.read_u8()?,
            tire_damage_status: reader.read_u8()?,
            add_siren: reader.read_u8()?,
            mod_slots: read_fixed(reader)?,
            paint_job: reader.read_u8()?,
            interior_color1: reader.read_i32_le()?,
            interior_color2: reader.read_i32_le()?,
        },
    })
}

fn encode_vehicle_stream_in<W: BitWrite>(
    writer: &mut W,
    value: &VehicleStreamIn,
) -> Result<(), EncodeError<W::Error>> {
    let vehicle = &value.vehicle;
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_i32_le(vehicle.model)?;
    writer.write_vector3_le(&vehicle.position)?;
    writer.write_f32_le(vehicle.rotation)?;
    writer.write_u8(vehicle.body_color1)?;
    writer.write_u8(vehicle.body_color2)?;
    writer.write_f32_le(vehicle.health)?;
    writer.write_u8(vehicle.interior_id)?;
    writer.write_i32_le(vehicle.door_damage_status)?;
    writer.write_i32_le(vehicle.panel_damage_status)?;
    writer.write_u8(vehicle.light_damage_status)?;
    writer.write_u8(vehicle.tire_damage_status)?;
    writer.write_u8(vehicle.add_siren)?;
    writer.write_bytes(&vehicle.mod_slots)?;
    writer.write_u8(vehicle.paint_job)?;
    writer.write_i32_le(vehicle.interior_color1)?;
    writer.write_i32_le(vehicle.interior_color2)
}
