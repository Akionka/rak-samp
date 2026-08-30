use super::{PacketPlayerId, RawPacket};
use modkit_abi::{
    CommandCompletionV1, CoreServiceV1, HostStatusV1, LegacySampServiceV1, ModHostApiV1,
    ServiceHeader,
};
use std::mem::{MaybeUninit, align_of, offset_of, size_of};
use std::ptr;

unsafe extern "C" {
    fn samp_client_sdk_fixture_player_id_size() -> usize;
    fn samp_client_sdk_fixture_player_id_alignment() -> usize;
    fn samp_client_sdk_fixture_packet_size() -> usize;
    fn samp_client_sdk_fixture_packet_alignment() -> usize;
    fn samp_client_sdk_fixture_packet_player_index_offset() -> usize;
    fn samp_client_sdk_fixture_packet_player_id_offset() -> usize;
    fn samp_client_sdk_fixture_packet_length_offset() -> usize;
    fn samp_client_sdk_fixture_packet_bit_size_offset() -> usize;
    fn samp_client_sdk_fixture_packet_data_offset() -> usize;
    fn samp_client_sdk_fixture_packet_delete_data_offset() -> usize;
    fn samp_client_sdk_fixture_initialize_packet(memory: *mut RawPacket, data: *mut u8);
    fn modkit_fixture_service_header_size() -> usize;
    fn modkit_fixture_bootstrap_size() -> usize;
    fn modkit_fixture_core_size() -> usize;
    fn modkit_fixture_legacy_size() -> usize;
    fn modkit_fixture_host_status_size() -> usize;
    fn modkit_fixture_command_completion_size() -> usize;
    fn modkit_fixture_command_completion_alignment() -> usize;
}

#[test]
fn modkit_v1_layouts_match_the_independent_cpp_fixture() {
    unsafe {
        assert_eq!(
            size_of::<ServiceHeader>(),
            modkit_fixture_service_header_size()
        );
        assert_eq!(size_of::<ModHostApiV1>(), modkit_fixture_bootstrap_size());
        assert_eq!(size_of::<CoreServiceV1>(), modkit_fixture_core_size());
        assert_eq!(
            size_of::<LegacySampServiceV1>(),
            modkit_fixture_legacy_size()
        );
        assert_eq!(size_of::<HostStatusV1>(), modkit_fixture_host_status_size());
        assert_eq!(
            size_of::<CommandCompletionV1>(),
            modkit_fixture_command_completion_size()
        );
        assert_eq!(
            align_of::<CommandCompletionV1>(),
            modkit_fixture_command_completion_alignment()
        );
    }
}

#[test]
fn raknet_packet_layout_matches_the_cpp_x86_abi() {
    unsafe {
        assert_eq!(
            size_of::<PacketPlayerId>(),
            samp_client_sdk_fixture_player_id_size()
        );
        assert_eq!(
            align_of::<PacketPlayerId>(),
            samp_client_sdk_fixture_player_id_alignment()
        );

        assert_eq!(
            size_of::<RawPacket>(),
            samp_client_sdk_fixture_packet_size()
        );
        assert_eq!(
            align_of::<RawPacket>(),
            samp_client_sdk_fixture_packet_alignment()
        );
        assert_eq!(
            offset_of!(RawPacket, player_index),
            samp_client_sdk_fixture_packet_player_index_offset()
        );
        assert_eq!(
            offset_of!(RawPacket, player_id),
            samp_client_sdk_fixture_packet_player_id_offset()
        );
        assert_eq!(
            offset_of!(RawPacket, length),
            samp_client_sdk_fixture_packet_length_offset()
        );
        assert_eq!(
            offset_of!(RawPacket, bit_size),
            samp_client_sdk_fixture_packet_bit_size_offset()
        );
        assert_eq!(
            offset_of!(RawPacket, data),
            samp_client_sdk_fixture_packet_data_offset()
        );
        assert_eq!(
            offset_of!(RawPacket, delete_data),
            samp_client_sdk_fixture_packet_delete_data_offset()
        );
    }
}

#[test]
fn reads_a_packet_initialized_by_cpp() {
    let mut data = [0xAA, 0xBB, 0xCC];
    let mut packet = MaybeUninit::<RawPacket>::uninit();
    unsafe {
        samp_client_sdk_fixture_initialize_packet(packet.as_mut_ptr(), data.as_mut_ptr());
        let packet = packet.assume_init();
        assert_eq!(ptr::addr_of!(packet.player_index).read_unaligned(), 0x1234);
        assert_eq!(
            ptr::addr_of!(packet.player_id.binary_address).read_unaligned(),
            0x01020304
        );
        assert_eq!(
            ptr::addr_of!(packet.player_id.port).read_unaligned(),
            0x5678
        );
        assert_eq!(ptr::addr_of!(packet.length).read_unaligned(), 3);
        assert_eq!(ptr::addr_of!(packet.bit_size).read_unaligned(), 17);
        assert_eq!(
            ptr::addr_of!(packet.data).read_unaligned(),
            data.as_mut_ptr()
        );
        assert!(ptr::addr_of!(packet.delete_data).read_unaligned());
    }
}
