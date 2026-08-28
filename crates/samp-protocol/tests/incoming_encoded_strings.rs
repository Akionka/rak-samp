use samp_protocol::{
    BitRead, BitStream, BitStreamError, BitWrite, DecodeError, EncodeError, EncodedBits,
    EncodedBitsError, EncodedStringRead, EncodedStringWireDescriptor, EncodedStringWrite,
    limits::MAX_ENCODED_STRING_BYTES,
    rpc::incoming::common::{SHOW_DIALOG, ShowDialog, ShowDialogRpc},
    rpc::incoming::r1::{
        CREATE_3D_TEXT, CREATE_OBJECT, Create3DTextRpc, CreateObjectRpc, MAX_OBJECT_MATERIALS,
        Object, ObjectAttachment, ObjectMaterial, ObjectMaterialUpdate, SET_OBJECT_MATERIAL,
        SetObjectMaterialRpc, TextLabel3D, TextMaterial, TextureMaterial,
    },
    types::Vector3,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum TestError {
    Bits(BitStreamError),
    EncodedStringUnavailable,
}

struct TestStream {
    bits: BitStream,
    encoded_strings_available: bool,
}

impl TestStream {
    fn new() -> Self {
        Self {
            bits: BitStream::new(),
            encoded_strings_available: true,
        }
    }

    fn from_bits(bits: &EncodedBits) -> Self {
        Self {
            bits: BitStream::from_bits(bits.as_bytes().to_vec(), bits.len_bits()).unwrap(),
            encoded_strings_available: true,
        }
    }

    fn finish(self) -> EncodedBits {
        EncodedBits::from_bits(self.bits.as_bytes().to_vec(), self.bits.len_bits()).unwrap()
    }
}

impl BitRead for TestStream {
    type Error = TestError;

    fn remaining_bits(&self) -> usize {
        self.bits.remaining_bits()
    }

    fn read_left_aligned_bits(&mut self, bit_len: usize) -> Result<Vec<u8>, Self::Error> {
        BitRead::read_left_aligned_bits(&mut self.bits, bit_len).map_err(TestError::Bits)
    }
}

impl BitWrite for TestStream {
    type Error = TestError;

    fn write_left_aligned_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), Self::Error> {
        BitWrite::write_left_aligned_bits(&mut self.bits, bytes, bit_len).map_err(TestError::Bits)
    }
}

impl EncodedStringRead for TestStream {
    fn read_encoded_string(&mut self, _max_len: usize) -> Result<Vec<u8>, Self::Error> {
        if !self.encoded_strings_available {
            return Err(TestError::EncodedStringUnavailable);
        }
        let length = self.read_left_aligned_bits(16)?;
        let length = usize::from(u16::from_le_bytes([length[0], length[1]]));
        self.read_left_aligned_bits(length * 8)
    }
}

impl EncodedStringWrite for TestStream {
    fn write_encoded_string(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        if !self.encoded_strings_available {
            return Err(TestError::EncodedStringUnavailable);
        }
        let length = u16::try_from(value.len()).map_err(|_| TestError::EncodedStringUnavailable)?;
        self.write_left_aligned_bits(&length.to_le_bytes(), 16)?;
        self.write_left_aligned_bits(value, value.len() * 8)
    }

    fn finish_encoded_bits(self) -> Result<EncodedBits, EncodedBitsError> {
        EncodedBits::from_bits(self.bits.as_bytes().to_vec(), self.bits.len_bits())
    }
}

fn dialog() -> ShowDialog {
    ShowDialog {
        dialog_id: 0x1234,
        style: 2,
        title: b"T".to_vec(),
        button1: b"OK".to_vec(),
        button2: Vec::new(),
        text: b"hi".to_vec(),
    }
}

#[test]
fn show_dialog_uses_injected_encoded_string_exactly() {
    let encoded = ShowDialogRpc::encode_bits(TestStream::new(), &dialog()).unwrap();

    assert_eq!(SHOW_DIALOG, ShowDialogRpc);
    assert_eq!(encoded.len_bits(), 104);
    assert_eq!(
        encoded.as_bytes(),
        &[0x34, 0x12, 2, 1, b'T', 2, b'O', b'K', 0, 2, 0, b'h', b'i']
    );

    let mut reader = TestStream::from_bits(&encoded);
    assert_eq!(ShowDialogRpc::decode_from(&mut reader), Ok(dialog()));
    assert_eq!(reader.remaining_bits(), 0);
}

#[test]
fn show_dialog_preserves_encoded_string_source_errors() {
    let mut writer = TestStream::new();
    writer.encoded_strings_available = false;

    assert_eq!(
        ShowDialogRpc::encode_to(&mut writer, &dialog()),
        Err(EncodeError::Source(TestError::EncodedStringUnavailable))
    );
}

#[test]
fn show_dialog_preserves_encoded_string_read_errors() {
    let mut writer = TestStream::new();
    ShowDialogRpc::encode_to(&mut writer, &dialog()).unwrap();
    let encoded = writer.finish();
    let mut reader = TestStream::from_bits(&encoded);
    reader.encoded_strings_available = false;

    assert_eq!(
        ShowDialogRpc::decode_from(&mut reader),
        Err(DecodeError::Source(TestError::EncodedStringUnavailable))
    );
}

#[test]
fn show_dialog_rejects_embedded_nul_and_logical_limit() {
    let mut embedded_nul = dialog();
    embedded_nul.text = b"a\0b".to_vec();
    assert_eq!(
        ShowDialogRpc::encode_to(&mut TestStream::new(), &embedded_nul),
        Err(EncodeError::EmbeddedNul)
    );

    let mut oversized = dialog();
    oversized.text = vec![b'x'; MAX_ENCODED_STRING_BYTES + 1];
    assert_eq!(
        ShowDialogRpc::encode_to(&mut TestStream::new(), &oversized),
        Err(EncodeError::LengthExceedsLimit {
            length: MAX_ENCODED_STRING_BYTES + 1,
            limit: MAX_ENCODED_STRING_BYTES,
        })
    );
}

#[test]
fn show_dialog_rejects_decoded_embedded_nul() {
    let bytes = [
        &[0x34, 0x12, 2, 1, b'T', 2, b'O', b'K', 0, 3, 0][..],
        b"a\0b",
    ]
    .concat();
    let malformed = EncodedBits::from_bits(bytes.clone(), bytes.len() * 8).unwrap();

    assert_eq!(
        ShowDialogRpc::decode_from(&mut TestStream::from_bits(&malformed)),
        Err(DecodeError::EmbeddedNul)
    );
}

fn vector3(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3 { x, y, z }
}

fn text_material(text: &[u8]) -> TextMaterial {
    TextMaterial {
        material_id: 1,
        material_size: 90,
        font_name: b"Arial".to_vec(),
        font_size: 24,
        bold: 1,
        font_color: -1,
        background_color: 0,
        align: 1,
        text: text.to_vec(),
    }
}

fn texture_material() -> TextureMaterial {
    TextureMaterial {
        material_id: 0,
        model_id: 1_337,
        library_name: b"lib".to_vec(),
        texture_name: b"tex".to_vec(),
        color: -1,
    }
}

#[test]
fn create_3d_text_preserves_the_exact_vector() {
    let value = TextLabel3D {
        id: 4,
        color: -1,
        position: vector3(1.0, 2.0, 3.0),
        distance: 50.0,
        test_los: true,
        attached_player_id: u16::MAX,
        attached_vehicle_id: u16::MAX,
        text: b"label".to_vec(),
    };
    let expected = [
        4, 0, 255, 255, 255, 255, 0, 0, 128, 63, 0, 0, 0, 64, 0, 0, 64, 64, 0, 0, 72, 66, 1, 255,
        255, 255, 255, 5, 0, b'l', b'a', b'b', b'e', b'l',
    ];
    let mut writer = TestStream::new();
    Create3DTextRpc::encode_to(&mut writer, &value).unwrap();
    let encoded = writer.finish();

    assert_eq!(CREATE_3D_TEXT, Create3DTextRpc);
    assert_eq!(encoded.as_bytes(), expected);
    assert_eq!(encoded.len_bits(), expected.len() * 8);
    assert_eq!(
        Create3DTextRpc::decode_from(&mut TestStream::from_bits(&encoded)),
        Ok(value)
    );
}

#[test]
fn set_object_material_preserves_texture_and_text_variants() {
    for value in [
        ObjectMaterialUpdate {
            object_id: 9,
            material: ObjectMaterial::Texture(texture_material()),
        },
        ObjectMaterialUpdate {
            object_id: 9,
            material: ObjectMaterial::Text(text_material(b"hello")),
        },
    ] {
        let mut writer = TestStream::new();
        SetObjectMaterialRpc::encode_to(&mut writer, &value).unwrap();
        let encoded = writer.finish();
        assert_eq!(
            SetObjectMaterialRpc::decode_from(&mut TestStream::from_bits(&encoded)),
            Ok(value)
        );
    }

    assert_eq!(SET_OBJECT_MATERIAL, SetObjectMaterialRpc);
}

#[test]
fn create_object_preserves_nested_texture_and_text_materials() {
    let value = Object {
        object_id: 9,
        model_id: 1_337,
        position: vector3(1.0, 2.0, 3.0),
        rotation: vector3(4.0, 5.0, 6.0),
        draw_distance: 100.0,
        no_camera_collision: true,
        attach_to_vehicle_id: 7,
        attach_to_object_id: u16::MAX,
        attachment: Some(ObjectAttachment {
            offsets: vector3(0.5, 1.5, 2.5),
            rotation: vector3(10.0, 20.0, 30.0),
            sync_rotation: false,
        }),
        textures_count: 2,
        materials: vec![
            ObjectMaterial::Texture(texture_material()),
            ObjectMaterial::Text(text_material(b"nested")),
        ],
    };
    let mut writer = TestStream::new();
    CreateObjectRpc::encode_to(&mut writer, &value).unwrap();
    let encoded = writer.finish();

    assert_eq!(CREATE_OBJECT, CreateObjectRpc);
    assert_eq!(encoded.len_bits(), 872);
    assert_eq!(
        CreateObjectRpc::decode_from(&mut TestStream::from_bits(&encoded)),
        Ok(value)
    );
}

#[test]
fn object_material_rejects_malformed_input_and_collection_overflow() {
    let malformed = EncodedBits::from_bits([9, 0, 3], 24).unwrap();
    assert_eq!(
        SetObjectMaterialRpc::decode_from(&mut TestStream::from_bits(&malformed)),
        Err(DecodeError::InvalidDiscriminant { value: 3 })
    );

    let object = Object {
        object_id: 9,
        model_id: 1_337,
        position: vector3(0.0, 0.0, 0.0),
        rotation: vector3(0.0, 0.0, 0.0),
        draw_distance: 100.0,
        no_camera_collision: false,
        attach_to_vehicle_id: u16::MAX,
        attach_to_object_id: u16::MAX,
        attachment: None,
        textures_count: 0,
        materials: vec![ObjectMaterial::Texture(texture_material()); MAX_OBJECT_MATERIALS + 1],
    };
    assert_eq!(
        CreateObjectRpc::encode_to(&mut TestStream::new(), &object),
        Err(EncodeError::LengthExceedsLimit {
            length: MAX_OBJECT_MATERIALS + 1,
            limit: MAX_OBJECT_MATERIALS,
        })
    );
}
