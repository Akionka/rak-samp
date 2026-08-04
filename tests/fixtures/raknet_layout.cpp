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

struct FixtureR1TrailerData {
    unsigned short id;
    FixtureVector3 position;
    float quaternion[4];
    FixtureVector3 speed;
    FixtureVector3 turn_speed;
};

struct FixtureR1PassengerData {
    unsigned short vehicle_id;
    unsigned char seat_id;
    unsigned char weapon;
    unsigned char health;
    unsigned char armour;
    FixtureControllerState controller;
    FixtureVector3 position;
};

struct FixtureR1AimData {
    unsigned char camera_mode;
    FixtureVector3 aim_first;
    FixtureVector3 aim_position;
    float aim_z;
    unsigned char camera_zoom_and_weapon_state;
    char aspect_ratio;
};

// Independent packed reconstruction of the fixed R1 CRemotePlayer prefix.
// This fixture deliberately stops before its remaining marker state because
// the future safe snapshot needs only the fields below; no native pointer is
// exported by this fixture or the eventual ABI.
struct FixtureR1RemotePlayerPrefix {
    void* ped;
    void* vehicle;
    unsigned char team;
    unsigned char state;
    unsigned char seat_id;
    int unknown_b;
    int passenger_drive_by;
    unsigned char pad_13[64];
    FixtureVector3 position_difference;
    struct {
        float real;
        FixtureVector3 imag;
    } incar_target_rotation;
    int pad_6f[3];
    FixtureVector3 onfoot_target_position;
    FixtureVector3 onfoot_target_speed;
    FixtureVector3 incar_target_position;
    FixtureVector3 incar_target_speed;
    unsigned short id;
    unsigned short vehicle_id;
    int unknown_af;
    int draw_labels;
    int has_jetpack;
    unsigned char special_action;
    int pad_bc[3];
    FixtureR1OnfootData onfoot;
    FixtureR1IncarData incar;
    FixtureR1TrailerData trailer;
    FixtureR1PassengerData passenger;
    FixtureR1AimData aim;
    float reported_armour;
    float reported_health;
    unsigned int animation;
    unsigned char update_type;
    unsigned int last_update;
    unsigned int last_timestamp;
    int performing_custom_animation;
    int status;
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

struct FixtureR1VehicleInfo {
    unsigned short id;
    int type;
    float position[3];
    float rotation;
    unsigned char primary_colour;
    unsigned char secondary_colour;
    float health;
    char interior;
    int door_damage;
    int panel_damage;
    char light_damage;
    bool doors_locked;
    bool has_siren;
};

struct FixtureR1VehiclePoolExistsPrefix {
    int count;
    struct {
        FixtureR1VehicleInfo entry[100];
        int not_empty[100];
    } waiting;
    void* objects[2000];
    int not_empty[2000];
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
    unsigned int last_connect_attempt;
    void* settings;
    void* rak_client;
    void* pools;
};

struct FixtureR1NetGamePools {
    void* actor;
    void* object;
    void* gang_zone;
    void* label;
    void* text_draw;
    void* menu;
    void* player;
    void* vehicle;
    void* pickup;
};

struct FixtureR1TextLabel {
    char* text;
    unsigned int colour;
    FixtureVector3 position;
    float draw_distance;
    bool behind_walls;
    unsigned short attached_player;
    unsigned short attached_vehicle;
};

struct FixtureR1LabelPoolExistsPrefix {
    FixtureR1TextLabel objects[2048];
    int not_empty[2048];
};

struct FixtureR1TextDrawPoolExistsPrefix {
    int not_empty[2048 + 256];
    void* objects[2048 + 256];
};

struct FixtureR1TextDrawData {
    float letter_width;
    float letter_height;
    unsigned int letter_colour;
    unsigned char unknown;
    unsigned char align_center;
    unsigned char box_enabled;
    float box_width;
    float box_height;
    unsigned int box_colour;
    unsigned char proportional;
    unsigned int background_colour;
    unsigned char shadow;
    unsigned char outline;
    unsigned char align_left;
    unsigned char align_right;
    int style;
    float x;
    float y;
    unsigned char pad[8];
    unsigned int field_99b;
    unsigned int field_99f;
    unsigned int index;
    unsigned char field_9a7;
    unsigned short model_id;
    FixtureVector3 rotation;
    float zoom;
    unsigned short model_colour1;
    unsigned short model_colour2;
    unsigned char field_9be;
    unsigned char field_9bf;
    unsigned char field_9c0;
    unsigned int field_9c1;
    unsigned int field_9c5;
    unsigned int field_9c9;
    unsigned int field_9cd;
    unsigned char field_9d1;
    unsigned int field_9d2;
};

struct FixtureR1TextDraw {
    char text[801];
    char string[1602];
    FixtureR1TextDrawData data;
};

struct FixtureR1ObjectPoolExistsPrefix {
    int largest_id;
    int not_empty[1000];
    void* objects[1000];
};

struct FixtureR1Gangzone {
    float left;
    float bottom;
    float right;
    float top;
    unsigned int colour;
    unsigned int alternate_colour;
};

struct FixtureR1GangzonePoolPrefix {
    void* objects[1024];
    int not_empty[1024];
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

struct FixtureR1ChatEntry {
    int timestamp;
    char prefix[28];
    char text[144];
    unsigned char unused[64];
    int type;
    unsigned int text_colour;
    unsigned int prefix_colour;
};

struct FixtureR1ChatPrefix {
    unsigned char pad[0x132];
    FixtureR1ChatEntry entries[100];
};

struct FixtureR1DialogSnapshot {
    void* device;
    unsigned int position[2];
    unsigned int size[2];
    unsigned int button_offset[2];
    void* dialog;
    void* listbox;
    void* editbox;
    int is_active;
    int type;
    int id;
    char* text;
    int text_size[2];
    char caption[65];
    int server_side;
};

#pragma pack(push, 1)
struct FixtureDxutListBoxSelection {
    unsigned char pad[0x143];
    int selected;
    unsigned char pad_0[0x09];
    int item_count;
};
#pragma pack(pop)

struct FixtureR1InputPrefix {
    unsigned char pad_0[0x14E0];
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
static_assert(sizeof(FixtureR1TrailerData) == 54);
static_assert(sizeof(FixtureR1PassengerData) == 24);
static_assert(sizeof(FixtureR1AimData) == 31);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, special_action) == 0xBB);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, onfoot) == 0xC8);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, reported_armour) == 0x1B8);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, reported_health) == 0x1BC);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, animation) == 0x1C0);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, status) == 0x1D1);
static_assert(sizeof(FixtureR1LocalPlayerPrefix) == 92);
static_assert(offsetof(FixtureR1Ped, game_ped) == 0x2A4);
static_assert(offsetof(FixtureR1PlayerPoolPrefix, local_id) == 0x04);
static_assert(sizeof(FixtureR1VehicleInfo) == 40);
static_assert(offsetof(FixtureR1VehiclePoolExistsPrefix, not_empty) == 0x3074);
static_assert(offsetof(FixtureR1NetGamePrefix, host_address) == 0x20);
static_assert(offsetof(FixtureR1NetGamePrefix, hostname) == 0x121);
static_assert(offsetof(FixtureR1NetGamePrefix, port) == 0x225);
static_assert(offsetof(FixtureR1NetGamePrefix, game_state) == 0x3BD);
static_assert(offsetof(FixtureR1NetGamePrefix, settings) == 0x3C5);
static_assert(offsetof(FixtureR1NetGamePrefix, pools) == 0x3CD);
static_assert(offsetof(FixtureR1NetGamePools, label) == 0x0C);
static_assert(offsetof(FixtureR1NetGamePools, text_draw) == 0x10);
static_assert(offsetof(FixtureR1NetGamePools, object) == 0x04);
static_assert(offsetof(FixtureR1NetGamePools, gang_zone) == 0x08);
static_assert(offsetof(FixtureR1NetGamePools, pickup) == 0x20);
static_assert(sizeof(FixtureR1TextLabel) == 29);
static_assert(offsetof(FixtureR1TextLabel, text) == 0x00);
static_assert(offsetof(FixtureR1TextLabel, colour) == 0x04);
static_assert(offsetof(FixtureR1TextLabel, position) == 0x08);
static_assert(offsetof(FixtureR1TextLabel, draw_distance) == 0x14);
static_assert(offsetof(FixtureR1TextLabel, behind_walls) == 0x18);
static_assert(offsetof(FixtureR1TextLabel, attached_player) == 0x19);
static_assert(offsetof(FixtureR1TextLabel, attached_vehicle) == 0x1B);
static_assert(offsetof(FixtureR1LabelPoolExistsPrefix, not_empty) == 0xE800);
static_assert(offsetof(FixtureR1TextDrawPoolExistsPrefix, not_empty) == 0);
static_assert(sizeof(FixtureR1TextDrawPoolExistsPrefix) == 0x4800);
static_assert(offsetof(FixtureR1TextDrawPoolExistsPrefix, objects) == 0x2400);
static_assert(offsetof(FixtureR1TextDraw, data) == 0x963);
static_assert(offsetof(FixtureR1TextDraw, string) == 801);
static_assert(offsetof(FixtureR1TextDrawData, letter_width) == 0x00);
static_assert(offsetof(FixtureR1TextDrawData, letter_height) == 0x04);
static_assert(offsetof(FixtureR1TextDrawData, letter_colour) == 0x08);
static_assert(offsetof(FixtureR1TextDrawData, align_center) == 0x0D);
static_assert(offsetof(FixtureR1TextDrawData, box_enabled) == 0x0E);
static_assert(offsetof(FixtureR1TextDrawData, box_width) == 0x0F);
static_assert(offsetof(FixtureR1TextDrawData, box_height) == 0x13);
static_assert(offsetof(FixtureR1TextDrawData, box_colour) == 0x17);
static_assert(offsetof(FixtureR1TextDrawData, proportional) == 0x1B);
static_assert(offsetof(FixtureR1TextDrawData, background_colour) == 0x1C);
static_assert(offsetof(FixtureR1TextDrawData, shadow) == 0x20);
static_assert(offsetof(FixtureR1TextDrawData, outline) == 0x21);
static_assert(offsetof(FixtureR1TextDrawData, align_left) == 0x22);
static_assert(offsetof(FixtureR1TextDrawData, align_right) == 0x23);
static_assert(offsetof(FixtureR1TextDrawData, style) == 0x24);
static_assert(offsetof(FixtureR1TextDrawData, x) == 0x28);
static_assert(offsetof(FixtureR1TextDrawData, y) == 0x2C);
static_assert(offsetof(FixtureR1TextDrawData, model_id) == 0x45);
static_assert(offsetof(FixtureR1TextDrawData, rotation) == 0x47);
static_assert(offsetof(FixtureR1TextDrawData, zoom) == 0x53);
static_assert(offsetof(FixtureR1TextDrawData, model_colour1) == 0x57);
static_assert(offsetof(FixtureR1TextDrawData, model_colour2) == 0x59);
static_assert(offsetof(FixtureR1ObjectPoolExistsPrefix, not_empty) == 0x04);
static_assert(sizeof(FixtureR1Gangzone) == 0x18);
static_assert(offsetof(FixtureR1GangzonePoolPrefix, not_empty) == 0x1000);
static_assert(offsetof(FixtureR1GamePrefix, cursor_mode) == 0x55);
static_assert(offsetof(FixtureR1ScoreboardPrefix, is_enabled) == 0x00);
static_assert(sizeof(FixtureR1ChatEntry) == 0xFC);
static_assert(offsetof(FixtureR1ChatPrefix, entries) == 0x132);
static_assert(offsetof(FixtureR1DialogSnapshot, is_active) == 0x28);
static_assert(offsetof(FixtureR1DialogSnapshot, listbox) == 0x20);
static_assert(offsetof(FixtureDxutListBoxSelection, selected) == 0x143);
static_assert(offsetof(FixtureDxutListBoxSelection, item_count) == 0x150);
static_assert(offsetof(FixtureR1DialogSnapshot, type) == 0x2C);
static_assert(offsetof(FixtureR1DialogSnapshot, id) == 0x30);
static_assert(offsetof(FixtureR1DialogSnapshot, caption) == 0x40);
static_assert(offsetof(FixtureR1DialogSnapshot, server_side) == 0x81);
static_assert(offsetof(FixtureR1InputPrefix, is_enabled) == 0x14E0);

extern "C" {

std::size_t samp_client_sdk_fixture_player_id_size() {
    return sizeof(FixturePlayerId);
}

std::size_t samp_client_sdk_fixture_player_id_alignment() {
    return alignof(FixturePlayerId);
}

std::size_t samp_client_sdk_fixture_packet_size() {
    return sizeof(FixturePacket);
}

std::size_t samp_client_sdk_fixture_packet_alignment() {
    return alignof(FixturePacket);
}

std::size_t samp_client_sdk_fixture_packet_player_index_offset() {
    return offsetof(FixturePacket, player_index);
}

std::size_t samp_client_sdk_fixture_packet_player_id_offset() {
    return offsetof(FixturePacket, player_id);
}

std::size_t samp_client_sdk_fixture_packet_length_offset() {
    return offsetof(FixturePacket, length);
}

std::size_t samp_client_sdk_fixture_packet_bit_size_offset() {
    return offsetof(FixturePacket, bit_size);
}

std::size_t samp_client_sdk_fixture_packet_data_offset() {
    return offsetof(FixturePacket, data);
}

std::size_t samp_client_sdk_fixture_packet_delete_data_offset() {
    return offsetof(FixturePacket, delete_data);
}

void samp_client_sdk_fixture_initialize_packet(void* memory, unsigned char* data) {
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

std::size_t samp_client_sdk_fixture_r1_onfoot_size() {
    return sizeof(FixtureR1OnfootData);
}

std::size_t samp_client_sdk_fixture_r1_incar_size() {
    return sizeof(FixtureR1IncarData);
}

std::size_t samp_client_sdk_fixture_r1_local_player_prefix_size() {
    return sizeof(FixtureR1LocalPlayerPrefix);
}

std::size_t samp_client_sdk_fixture_r1_local_active_offset() {
    return offsetof(FixtureR1LocalPlayerPrefix, active);
}

std::size_t samp_client_sdk_fixture_r1_local_current_vehicle_offset() {
    return offsetof(FixtureR1LocalPlayerPrefix, current_vehicle);
}

std::size_t samp_client_sdk_fixture_r1_local_onfoot_offset() {
    return offsetof(FixtureR1LocalPlayerPrefix, onfoot);
}

std::size_t samp_client_sdk_fixture_r1_onfoot_position_offset() {
    return offsetof(FixtureR1OnfootData, position);
}

std::size_t samp_client_sdk_fixture_r1_onfoot_speed_offset() {
    return offsetof(FixtureR1OnfootData, speed);
}

std::size_t samp_client_sdk_fixture_r1_onfoot_special_action_offset() {
    return offsetof(FixtureR1OnfootData, special_action);
}

std::size_t samp_client_sdk_fixture_r1_onfoot_animation_offset() {
    return offsetof(FixtureR1OnfootData, animation);
}

std::size_t samp_client_sdk_fixture_r1_incar_position_offset() {
    return offsetof(FixtureR1IncarData, position);
}

std::size_t samp_client_sdk_fixture_r1_incar_speed_offset() {
    return offsetof(FixtureR1IncarData, speed);
}

std::size_t samp_client_sdk_fixture_r1_ped_game_ped_offset() {
    return offsetof(FixtureR1Ped, game_ped);
}

std::size_t samp_client_sdk_fixture_r1_player_pool_local_id_offset() {
    return offsetof(FixtureR1PlayerPoolPrefix, local_id);
}

std::size_t samp_client_sdk_fixture_r1_player_pool_largest_id_offset() {
    return offsetof(FixtureR1PlayerPoolPrefix, largest_id);
}

std::size_t samp_client_sdk_fixture_r1_vehicle_pool_not_empty_offset() {
    return offsetof(FixtureR1VehiclePoolExistsPrefix, not_empty);
}

std::size_t samp_client_sdk_fixture_r1_net_game_host_address_offset() {
    return offsetof(FixtureR1NetGamePrefix, host_address);
}

std::size_t samp_client_sdk_fixture_r1_net_game_hostname_offset() {
    return offsetof(FixtureR1NetGamePrefix, hostname);
}

std::size_t samp_client_sdk_fixture_r1_net_game_port_offset() {
    return offsetof(FixtureR1NetGamePrefix, port);
}

std::size_t samp_client_sdk_fixture_r1_net_game_game_state_offset() {
    return offsetof(FixtureR1NetGamePrefix, game_state);
}

std::size_t samp_client_sdk_fixture_r1_net_game_server_settings_offset() {
    return offsetof(FixtureR1NetGamePrefix, settings);
}

std::size_t samp_client_sdk_fixture_r1_net_game_pools_offset() {
    return offsetof(FixtureR1NetGamePrefix, pools);
}

std::size_t samp_client_sdk_fixture_r1_net_game_pools_label_offset() {
    return offsetof(FixtureR1NetGamePools, label);
}

std::size_t samp_client_sdk_fixture_r1_net_game_pools_text_draw_offset() {
    return offsetof(FixtureR1NetGamePools, text_draw);
}

std::size_t samp_client_sdk_fixture_r1_net_game_pools_object_offset() {
    return offsetof(FixtureR1NetGamePools, object);
}

std::size_t samp_client_sdk_fixture_r1_net_game_pools_gang_zone_offset() {
    return offsetof(FixtureR1NetGamePools, gang_zone);
}

std::size_t samp_client_sdk_fixture_r1_net_game_pools_pickup_offset() {
    return offsetof(FixtureR1NetGamePools, pickup);
}

std::size_t samp_client_sdk_fixture_r1_label_pool_not_empty_offset() {
    return offsetof(FixtureR1LabelPoolExistsPrefix, not_empty);
}

std::size_t samp_client_sdk_fixture_r1_text_label_size() {
    return sizeof(FixtureR1TextLabel);
}

std::size_t samp_client_sdk_fixture_r1_text_label_text_offset() {
    return offsetof(FixtureR1TextLabel, text);
}

std::size_t samp_client_sdk_fixture_r1_text_label_colour_offset() {
    return offsetof(FixtureR1TextLabel, colour);
}

std::size_t samp_client_sdk_fixture_r1_text_label_position_offset() {
    return offsetof(FixtureR1TextLabel, position);
}

std::size_t samp_client_sdk_fixture_r1_text_label_draw_distance_offset() {
    return offsetof(FixtureR1TextLabel, draw_distance);
}

std::size_t samp_client_sdk_fixture_r1_text_label_behind_walls_offset() {
    return offsetof(FixtureR1TextLabel, behind_walls);
}

std::size_t samp_client_sdk_fixture_r1_text_label_attached_player_offset() {
    return offsetof(FixtureR1TextLabel, attached_player);
}

std::size_t samp_client_sdk_fixture_r1_text_label_attached_vehicle_offset() {
    return offsetof(FixtureR1TextLabel, attached_vehicle);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_pool_not_empty_offset() {
    return offsetof(FixtureR1TextDrawPoolExistsPrefix, not_empty);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_pool_objects_offset() {
    return offsetof(FixtureR1TextDrawPoolExistsPrefix, objects);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_data_offset() {
    return offsetof(FixtureR1TextDraw, data);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_string_offset() {
    return offsetof(FixtureR1TextDraw, string);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_letter_width_offset() {
    return offsetof(FixtureR1TextDrawData, letter_width);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_letter_height_offset() {
    return offsetof(FixtureR1TextDrawData, letter_height);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_letter_colour_offset() {
    return offsetof(FixtureR1TextDrawData, letter_colour);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_align_center_offset() {
    return offsetof(FixtureR1TextDrawData, align_center);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_box_enabled_offset() {
    return offsetof(FixtureR1TextDrawData, box_enabled);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_box_width_offset() {
    return offsetof(FixtureR1TextDrawData, box_width);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_box_height_offset() {
    return offsetof(FixtureR1TextDrawData, box_height);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_box_colour_offset() {
    return offsetof(FixtureR1TextDrawData, box_colour);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_proportional_offset() {
    return offsetof(FixtureR1TextDrawData, proportional);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_background_colour_offset() {
    return offsetof(FixtureR1TextDrawData, background_colour);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_shadow_offset() {
    return offsetof(FixtureR1TextDrawData, shadow);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_outline_offset() {
    return offsetof(FixtureR1TextDrawData, outline);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_align_left_offset() {
    return offsetof(FixtureR1TextDrawData, align_left);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_align_right_offset() {
    return offsetof(FixtureR1TextDrawData, align_right);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_style_offset() {
    return offsetof(FixtureR1TextDrawData, style);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_x_offset() {
    return offsetof(FixtureR1TextDrawData, x);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_y_offset() {
    return offsetof(FixtureR1TextDrawData, y);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_model_id_offset() {
    return offsetof(FixtureR1TextDrawData, model_id);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_rotation_offset() {
    return offsetof(FixtureR1TextDrawData, rotation);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_zoom_offset() {
    return offsetof(FixtureR1TextDrawData, zoom);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_model_colour1_offset() {
    return offsetof(FixtureR1TextDrawData, model_colour1);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_model_colour2_offset() {
    return offsetof(FixtureR1TextDrawData, model_colour2);
}

std::size_t samp_client_sdk_fixture_r1_object_pool_not_empty_offset() {
    return offsetof(FixtureR1ObjectPoolExistsPrefix, not_empty);
}

std::size_t samp_client_sdk_fixture_r1_gangzone_pool_not_empty_offset() {
    return offsetof(FixtureR1GangzonePoolPrefix, not_empty);
}

std::size_t samp_client_sdk_fixture_r1_gangzone_size() {
    return sizeof(FixtureR1Gangzone);
}

std::size_t samp_client_sdk_fixture_r1_game_cursor_mode_offset() {
    return offsetof(FixtureR1GamePrefix, cursor_mode);
}

std::size_t samp_client_sdk_fixture_r1_scoreboard_enabled_offset() {
    return offsetof(FixtureR1ScoreboardPrefix, is_enabled);
}

std::size_t samp_client_sdk_fixture_r1_chat_entries_offset() {
    return offsetof(FixtureR1ChatPrefix, entries);
}

std::size_t samp_client_sdk_fixture_r1_chat_entry_size() {
    return sizeof(FixtureR1ChatEntry);
}

std::size_t samp_client_sdk_fixture_r1_dialog_active_offset() {
    return offsetof(FixtureR1DialogSnapshot, is_active);
}

std::size_t samp_client_sdk_fixture_r1_dialog_listbox_offset() {
    return offsetof(FixtureR1DialogSnapshot, listbox);
}

std::size_t samp_client_sdk_fixture_dxut_listbox_selected_offset() {
    return offsetof(FixtureDxutListBoxSelection, selected);
}

std::size_t samp_client_sdk_fixture_dxut_listbox_item_count_offset() {
    return offsetof(FixtureDxutListBoxSelection, item_count);
}

std::size_t samp_client_sdk_fixture_r1_dialog_type_offset() {
    return offsetof(FixtureR1DialogSnapshot, type);
}

std::size_t samp_client_sdk_fixture_r1_dialog_id_offset() {
    return offsetof(FixtureR1DialogSnapshot, id);
}

std::size_t samp_client_sdk_fixture_r1_dialog_caption_offset() {
    return offsetof(FixtureR1DialogSnapshot, caption);
}

std::size_t samp_client_sdk_fixture_r1_dialog_server_side_offset() {
    return offsetof(FixtureR1DialogSnapshot, server_side);
}

std::size_t samp_client_sdk_fixture_r1_input_enabled_offset() {
    return offsetof(FixtureR1InputPrefix, is_enabled);
}

}
