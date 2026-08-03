#include <cstddef>

// Minimal independent equivalents of RakNet 2.52's PlayerID and Packet declarations:
// https://github.com/openmultiplayer/RakNet/blob/master/Include/raknet/NetworkTypes.h
using FixturePlayerIndex = unsigned short;

#pragma pack(push, 1)
struct FixturePlayerId {
    unsigned int binary_address;
    unsigned short port;
};

struct FixturePacket {
    FixturePlayerIndex player_index;
    FixturePlayerId player_id;
    unsigned int length;
    unsigned int bit_size;
    unsigned char* data;
    bool delete_data;
};
#pragma pack(pop)

static_assert(sizeof(void*) == 4, "the RakNet layout fixture must be compiled for x86");
static_assert(sizeof(FixturePlayerId) == 6);
static_assert(alignof(FixturePlayerId) == 1);
static_assert(sizeof(FixturePacket) == 21);
static_assert(alignof(FixturePacket) == 1);

extern "C" {

std::size_t rak_samp_fixture_player_id_size() {
    return sizeof(FixturePlayerId);
}

std::size_t rak_samp_fixture_player_id_alignment() {
    return alignof(FixturePlayerId);
}

std::size_t rak_samp_fixture_packet_size() {
    return sizeof(FixturePacket);
}

std::size_t rak_samp_fixture_packet_alignment() {
    return alignof(FixturePacket);
}

std::size_t rak_samp_fixture_packet_player_index_offset() {
    return offsetof(FixturePacket, player_index);
}

std::size_t rak_samp_fixture_packet_player_id_offset() {
    return offsetof(FixturePacket, player_id);
}

std::size_t rak_samp_fixture_packet_length_offset() {
    return offsetof(FixturePacket, length);
}

std::size_t rak_samp_fixture_packet_bit_size_offset() {
    return offsetof(FixturePacket, bit_size);
}

std::size_t rak_samp_fixture_packet_data_offset() {
    return offsetof(FixturePacket, data);
}

std::size_t rak_samp_fixture_packet_delete_data_offset() {
    return offsetof(FixturePacket, delete_data);
}

void rak_samp_fixture_initialize_packet(void* memory, unsigned char* data) {
    auto* packet = static_cast<FixturePacket*>(memory);
    *packet = {};
    packet->player_index = 0x1234;
    packet->player_id.binary_address = 0x01020304;
    packet->player_id.port = 0x5678;
    packet->length = 3;
    packet->bit_size = 17;
    packet->data = data;
    packet->delete_data = true;
}

}
