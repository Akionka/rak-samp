use super::wire::{read_bool8, write_bool8};
use crate::encoded_string::{read_encoded_string, write_encoded_string};
use crate::limits::MAX_ENCODED_STRING_BYTES;
use crate::types::Vector3;
use crate::{
    DecodeError, EncodeError, EncodedStringRead, EncodedStringWireCodec, EncodedStringWrite,
    WireReadExt, WireWriteExt,
};

/// MoonLoader's `onCreate3DText` payload (RPC 36).
#[derive(Clone, Debug, PartialEq)]
pub struct TextLabel3D {
    pub id: u16,
    pub color: i32,
    pub position: Vector3,
    pub distance: f32,
    pub test_los: bool,
    pub attached_player_id: u16,
    pub attached_vehicle_id: u16,
    pub text: Vec<u8>,
}

struct Create3DTextCodec;

impl EncodedStringWireCodec for Create3DTextCodec {
    type Value = TextLabel3D;

    fn decode<R: EncodedStringRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        Ok(TextLabel3D {
            id: reader.read_u16_le()?,
            color: reader.read_i32_le()?,
            position: reader.read_vector3_le()?,
            distance: reader.read_f32_le()?,
            test_los: read_bool8(reader)?,
            attached_player_id: reader.read_u16_le()?,
            attached_vehicle_id: reader.read_u16_le()?,
            text: read_encoded_string(reader, MAX_ENCODED_STRING_BYTES)?,
        })
    }

    fn encode<W: EncodedStringWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        writer.write_u16_le(value.id)?;
        writer.write_i32_le(value.color)?;
        writer.write_vector3_le(&value.position)?;
        writer.write_f32_le(value.distance)?;
        write_bool8(writer, value.test_los)?;
        writer.write_u16_le(value.attached_player_id)?;
        writer.write_u16_le(value.attached_vehicle_id)?;
        write_encoded_string(writer, &value.text, MAX_ENCODED_STRING_BYTES)
    }
}

encoded_string_rpc_descriptor!(
    Create3DTextRpc,
    CREATE_3D_TEXT,
    36,
    Create3DTextCodec,
    TextLabel3D
);
