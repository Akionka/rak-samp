use samp_protocol::packet::r1::{
    MARKERS_SYNC, MAX_MARKERS, Marker, MarkerCoordinates, MarkersSync, MarkersSyncPacket,
    PLAYER_SYNC, RemotePlayerAnimation, RemotePlayerSurfing, RemotePlayerSync,
    RemotePlayerSyncCodec, RemoteVehicleSync, VEHICLE_SYNC,
};
use samp_protocol::{
    BitRead, BitStream, BitWrite, DecodeError, EncodeError, EncodedBits, WireCodec, WireDescriptor,
    types::Vector3,
};

fn vector3(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3 { x, y, z }
}

fn assert_vector<D>(descriptor: D, value: D::Value, expected: &[u8], bit_len: usize)
where
    D: WireDescriptor,
    D::Value: Clone + core::fmt::Debug + PartialEq,
{
    let _ = descriptor;
    let bits = D::encode_bits(&value).expect("the packet value must encode");

    assert_eq!(bits.as_bytes(), expected);
    assert_eq!(bits.len_bits(), bit_len);
    assert_eq!(D::decode_bits(&bits), Ok(value));
}

#[test]
fn r1_sync_packet_inventory_has_three_incoming_descriptors() {
    fn id<D: WireDescriptor>(_: D) -> u8 {
        D::ID
    }

    assert_eq!(
        [id(PLAYER_SYNC), id(VEHICLE_SYNC), id(MARKERS_SYNC)],
        [207, 200, 208]
    );
}

#[test]
fn r1_sync_packets_preserve_exact_vectors() {
    assert_vector(
        PLAYER_SYNC,
        RemotePlayerSync {
            player_id: 1,
            left_right_keys: Some(2),
            up_down_keys: None,
            key_data: 3,
            position: vector3(1.0, 2.0, 3.0),
            quaternion: [-1.0, 0.0, 0.0, 0.0],
            health: 100,
            armour: 98,
            weapon: 24,
            special_action: 0,
            move_speed: vector3(0.0, 0.0, 0.0),
            surfing: Some(RemotePlayerSurfing {
                vehicle_id: 4,
                offsets: vector3(4.0, 5.0, 6.0),
            }),
            animation: Some(RemotePlayerAnimation { id: 7, flags: 8 }),
        },
        &[
            1, 0, 129, 0, 0, 192, 0, 0, 32, 15, 192, 0, 0, 16, 0, 0, 16, 16, 32, 0, 0, 0, 0, 0, 3,
            248, 96, 0, 0, 0, 0, 2, 8, 0, 0, 1, 0, 128, 0, 1, 64, 128, 0, 1, 128, 129, 7, 0, 8, 0,
        ],
        400,
    );
    assert_vector(
        VEHICLE_SYNC,
        RemoteVehicleSync {
            player_id: 1,
            vehicle_id: 2,
            left_right_keys: 3,
            up_down_keys: 4,
            key_data: 5,
            quaternion: [1.0, 0.0, 0.0, 0.0],
            position: vector3(1.0, 2.0, 3.0),
            // R1's compressed-vector zero components decode to -1 / 65536 after the
            // writer's integer conversion; use the exact representable values here.
            move_speed: vector3(1.0, -1.0 / 65_536.0, -1.0 / 65_536.0),
            vehicle_health: 900,
            player_health: 98,
            armour: 0,
            current_weapon: 24,
            siren: true,
            landing_gear: false,
            train_speed: Some(-7),
            trailer_id: Some(6),
        },
        &[
            1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 3, 240, 0, 0, 4, 0, 0, 4, 4,
            0, 0, 8, 3, 255, 255, 255, 247, 255, 247, 248, 64, 62, 1, 139, 243, 255, 255, 255, 6,
            0,
        ],
        392,
    );
    assert_vector(
        MARKERS_SYNC,
        MarkersSync {
            markers: vec![
                Marker {
                    player_id: 1,
                    coordinates: None,
                },
                Marker {
                    player_id: 2,
                    coordinates: Some(MarkerCoordinates { x: -1, y: -2, z: 3 }),
                },
            ],
        },
        &[
            2, 0, 0, 0, 1, 0, 1, 0, 0x7F, 0xFF, 0xFF, 0xBF, 0xC0, 0xC0, 0,
        ],
        114,
    );
}

#[test]
fn r1_sync_packet_codec_preserves_values_from_an_unaligned_cursor() {
    let value = RemotePlayerSync {
        player_id: 1,
        left_right_keys: Some(2),
        up_down_keys: None,
        key_data: 3,
        position: vector3(1.0, 2.0, 3.0),
        quaternion: [-1.0, 0.0, 0.0, 0.0],
        health: 100,
        armour: 98,
        weapon: 24,
        special_action: 0,
        move_speed: vector3(0.0, 0.0, 0.0),
        surfing: None,
        animation: None,
    };
    let mut writer = BitStream::new();
    BitWrite::write_left_aligned_bits(&mut writer, &[0b1010_0000], 3).unwrap();
    RemotePlayerSyncCodec::encode(&mut writer, &value).unwrap();

    let mut reader = BitStream::from_bits(writer.as_bytes(), writer.len_bits()).unwrap();
    assert_eq!(
        BitRead::read_left_aligned_bits(&mut reader, 3),
        Ok(vec![0b1010_0000])
    );
    assert_eq!(RemotePlayerSyncCodec::decode(&mut reader), Ok(value));
    assert_eq!(reader.remaining_bits(), 0);
}

#[test]
fn marker_sync_accepts_terminal_alignment_padding_and_rejects_a_full_extra_byte() {
    let value = MarkersSync {
        markers: vec![Marker {
            player_id: 1,
            coordinates: None,
        }],
    };
    let canonical = MarkersSyncPacket::encode_bits(&value).expect("marker payload must encode");
    assert_eq!(canonical.len_bits(), 49);
    assert_eq!(
        canonical
            .as_bytes()
            .last()
            .expect("marker payload has a final byte")
            & 0x7F,
        0
    );

    let mut bytes = canonical.as_bytes().to_vec();
    *bytes.last_mut().expect("marker payload has a final byte") |= 0x40;
    let padded = EncodedBits::from_bits(bytes, 56).expect("the padded payload fits its buffer");
    assert_eq!(MarkersSyncPacket::decode_bits(&padded), Ok(value));

    let mut bytes = canonical.as_bytes().to_vec();
    bytes.push(0);
    let malformed = EncodedBits::from_bits(bytes, 57).expect("the malformed suffix fits");
    assert!(matches!(
        MarkersSyncPacket::decode_bits(&malformed),
        Err(DecodeError::InvalidTerminalPaddingLength {
            remaining_bits: 8,
            required_bits: 7,
        })
    ));
}

#[test]
fn marker_sync_accepts_zero_through_seven_terminal_alignment_bits() {
    for count in 0..=7 {
        let value = MarkersSync {
            markers: vec![
                Marker {
                    player_id: 1,
                    coordinates: None,
                };
                count
            ],
        };
        let canonical = MarkersSyncPacket::encode_bits(&value).expect("marker payload must encode");
        let padded = EncodedBits::from_bits(
            canonical.as_bytes().to_vec(),
            canonical.len_bits().next_multiple_of(8),
        )
        .expect("the padded marker payload fits its buffer");

        assert_eq!(MarkersSyncPacket::decode_bits(&padded), Ok(value));
    }
}

#[test]
fn marker_sync_enforces_the_protocol_player_slot_bound() {
    let marker = Marker {
        player_id: 1,
        coordinates: None,
    };
    let oversized = MarkersSync {
        markers: vec![marker; MAX_MARKERS + 1],
    };
    assert!(matches!(
        MarkersSyncPacket::encode_bits(&oversized),
        Err(EncodeError::LengthExceedsLimit { length, limit })
            if length == MAX_MARKERS + 1 && limit == MAX_MARKERS
    ));

    let count = EncodedBits::from_bits((MAX_MARKERS as i32 + 1).to_le_bytes(), 32)
        .expect("the marker count fits its buffer");
    assert!(matches!(
        MarkersSyncPacket::decode_bits(&count),
        Err(DecodeError::LengthExceedsLimit { length, limit })
            if length == MAX_MARKERS + 1 && limit == MAX_MARKERS
    ));
}
