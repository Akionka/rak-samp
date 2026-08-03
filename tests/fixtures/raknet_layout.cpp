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

// Independently recorded prefixes of the SA-MP 0.3.7 R1 synchronization
// layouts used by the direct local-player profile. These are deliberately not
// generated from the Rust implementation.
struct FixtureVector3 {
    float x;
    float y;
    float z;
};

struct FixtureControllerState {
    short left_stick_x;
    short left_stick_y;
    short buttons;
};

struct FixtureR1OnfootData {
    FixtureControllerState controller;
    FixtureVector3 position;
    float quaternion[4];
    unsigned char health;
    unsigned char armour;
    unsigned char weapon;
    unsigned char special_action;
    FixtureVector3 speed;
    FixtureVector3 surfing_offset;
    unsigned short surfing_vehicle;
    unsigned int animation;
};

struct FixtureR1IncarData {
    unsigned short vehicle;
    FixtureControllerState controller;
    float quaternion[4];
    FixtureVector3 position;
    FixtureVector3 speed;
    float health;
    unsigned char driver_health;
    unsigned char driver_armour;
    unsigned char weapon;
    bool siren;
    bool landing_gear;
    unsigned short trailer;
    unsigned int hydra_or_train;
};

struct FixtureR1LocalPlayerPrefix {
    void* ped;
    unsigned int animation;
    int field_8;
    int active;
    int wasted;
    unsigned short current_vehicle;
    unsigned short last_vehicle;
    FixtureR1OnfootData onfoot;
};

// Independent packed equivalents of the R1 `CEntity` / `CPed` prefix. The
// snapshot validates CPed's dedicated GTA-ped pointer before calling native
// health and armour getters.
struct FixtureR1Entity {
    void* vtable;
    unsigned char pad_4[60];
    void* game_entity;
    unsigned int handle;
};

struct FixtureR1Accessory {
    int model;
    int bone;
    FixtureVector3 offset;
    FixtureVector3 rotation;
    FixtureVector3 scale;
    unsigned int first_material_colour;
    unsigned int second_material_colour;
};

struct FixtureR1Ped {
    FixtureR1Entity entity;
    unsigned int using_cellphone;
    unsigned int accessory_not_empty[10];
    FixtureR1Accessory accessories[10];
    void* accessory_objects[10];
    void* game_ped;
};

struct FixtureR1PlayerPoolPrefix {
    int largest_id;
    unsigned short local_id;
};

struct FixtureR1NetGamePrefix {
    unsigned char pad_0[32];
    char host_address[257];
    char hostname[257];
    bool disable_collision;
    bool update_camera_target;
    bool nametag_status;
    int port;
    int lan_mode;
    int map_icons[100];
    int game_state;
};

// Independently recorded R1 UI-state prefixes. These are checked separately
// from the Rust profile before it copies the two scalar reads on the game
// thread.
struct FixtureR1GamePrefix {
    unsigned char pad_0[0x55];
    int cursor_mode;
};

struct FixtureR1ScoreboardPrefix {
    int is_enabled;
};
#pragma pack(pop)

static_assert(sizeof(void*) == 4, "the RakNet layout fixture must be compiled for x86");
static_assert(sizeof(FixturePlayerId) == 6);
static_assert(alignof(FixturePlayerId) == 1);
static_assert(sizeof(FixturePacket) == 21);
static_assert(alignof(FixturePacket) == 1);
static_assert(sizeof(FixtureVector3) == 12);
static_assert(sizeof(FixtureR1OnfootData) == 68);
static_assert(sizeof(FixtureR1IncarData) == 63);
static_assert(sizeof(FixtureR1LocalPlayerPrefix) == 92);
static_assert(offsetof(FixtureR1Ped, game_ped) == 0x2A4);
static_assert(offsetof(FixtureR1PlayerPoolPrefix, local_id) == 0x04);
static_assert(offsetof(FixtureR1NetGamePrefix, host_address) == 0x20);
static_assert(offsetof(FixtureR1NetGamePrefix, hostname) == 0x121);
static_assert(offsetof(FixtureR1NetGamePrefix, port) == 0x225);
static_assert(offsetof(FixtureR1NetGamePrefix, game_state) == 0x3BD);
static_assert(offsetof(FixtureR1GamePrefix, cursor_mode) == 0x55);
static_assert(offsetof(FixtureR1ScoreboardPrefix, is_enabled) == 0x00);

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

std::size_t rak_samp_fixture_r1_onfoot_size() {
    return sizeof(FixtureR1OnfootData);
}

std::size_t rak_samp_fixture_r1_incar_size() {
    return sizeof(FixtureR1IncarData);
}

std::size_t rak_samp_fixture_r1_local_player_prefix_size() {
    return sizeof(FixtureR1LocalPlayerPrefix);
}

std::size_t rak_samp_fixture_r1_local_active_offset() {
    return offsetof(FixtureR1LocalPlayerPrefix, active);
}

std::size_t rak_samp_fixture_r1_local_current_vehicle_offset() {
    return offsetof(FixtureR1LocalPlayerPrefix, current_vehicle);
}

std::size_t rak_samp_fixture_r1_local_onfoot_offset() {
    return offsetof(FixtureR1LocalPlayerPrefix, onfoot);
}

std::size_t rak_samp_fixture_r1_onfoot_position_offset() {
    return offsetof(FixtureR1OnfootData, position);
}

std::size_t rak_samp_fixture_r1_onfoot_speed_offset() {
    return offsetof(FixtureR1OnfootData, speed);
}

std::size_t rak_samp_fixture_r1_onfoot_special_action_offset() {
    return offsetof(FixtureR1OnfootData, special_action);
}

std::size_t rak_samp_fixture_r1_onfoot_animation_offset() {
    return offsetof(FixtureR1OnfootData, animation);
}

std::size_t rak_samp_fixture_r1_incar_position_offset() {
    return offsetof(FixtureR1IncarData, position);
}

std::size_t rak_samp_fixture_r1_incar_speed_offset() {
    return offsetof(FixtureR1IncarData, speed);
}

std::size_t rak_samp_fixture_r1_ped_game_ped_offset() {
    return offsetof(FixtureR1Ped, game_ped);
}

std::size_t rak_samp_fixture_r1_player_pool_local_id_offset() {
    return offsetof(FixtureR1PlayerPoolPrefix, local_id);
}

std::size_t rak_samp_fixture_r1_net_game_host_address_offset() {
    return offsetof(FixtureR1NetGamePrefix, host_address);
}

std::size_t rak_samp_fixture_r1_net_game_hostname_offset() {
    return offsetof(FixtureR1NetGamePrefix, hostname);
}

std::size_t rak_samp_fixture_r1_net_game_port_offset() {
    return offsetof(FixtureR1NetGamePrefix, port);
}

std::size_t rak_samp_fixture_r1_net_game_game_state_offset() {
    return offsetof(FixtureR1NetGamePrefix, game_state);
}

std::size_t rak_samp_fixture_r1_game_cursor_mode_offset() {
    return offsetof(FixtureR1GamePrefix, cursor_mode);
}

std::size_t rak_samp_fixture_r1_scoreboard_enabled_offset() {
    return offsetof(FixtureR1ScoreboardPrefix, is_enabled);
}

}
