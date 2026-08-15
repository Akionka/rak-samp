#include <cstddef>
#include <cstdint>
#ifdef _WIN32
#include <windows.h>
#endif

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

struct FixtureR1LocalPlayerSyncPrefix {
    void* ped;
    unsigned int animation;
    int field_8;
    int active;
    int wasted;
    unsigned short current_vehicle;
    unsigned short last_vehicle;
    FixtureR1OnfootData onfoot;
    FixtureR1PassengerData passenger;
    FixtureR1TrailerData trailer;
    FixtureR1IncarData incar;
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
    void* game_objects[2000];
};

struct FixtureR1ObjectPoolExistsPrefix {
    int largest_id;
    int not_empty[1000];
    void* objects[1000];
};

struct FixtureR1PickupPoolHandlesPrefix {
    int count;
    int handles[4096];
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

#pragma pack(push, 1)
struct FixtureR1TextDrawTransmit {
    unsigned char flags;
    float letter_width;
    float letter_height;
    unsigned int letter_colour;
    float box_width;
    float box_height;
    unsigned int box_colour;
    unsigned char shadow;
    unsigned char outline;
    unsigned int background_colour;
    unsigned char style;
    unsigned char unknown;
    float x;
    float y;
    unsigned short model_id;
    FixtureVector3 rotation;
    float zoom;
    unsigned short model_colour1;
    unsigned short model_colour2;
};
#pragma pack(pop)

struct FixtureR1TextDraw {
    char text[801];
    char string[1602];
    FixtureR1TextDrawData data;
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

// Mirrors the SF.lua `DXUTComboBoxItem` declaration at the pinned commit:
// https://github.com/SF-lua/SF.lua/blob/d869b8fb2ac9b527209e05376c19f3c96ee318e5/SFlua/cdef/dxut.lua
// `SCRect` is the windef `RECT` (four `long`s), so the fixture uses the real
// `RECT` from `windows.h` and the struct's default (non-packed) alignment,
// exactly as LuaJIT's `ffi.cdef` lays it out.
#pragma pack(pop)
struct FixtureDxutComboBoxItem {
    char str_text[256];
    void* data;
    RECT active_rect;
    bool visible;
};
#pragma pack(push, 1)

#pragma pack(push, 1)
struct FixtureDxutListBoxSelection {
    unsigned char pad[0x143];
    int selected;
    unsigned char pad_0[0x05];
    void* items;
    int item_count;
};
#pragma pack(pop)

struct FixtureR1InputPrefix {
    unsigned char pad_0[0x14E0];
    int is_enabled;
};
#pragma pack(pop)

// Independently recorded minimal profile gates for the three non-R1 builds.
// These deliberately retype only the CNetGame, command-input, and dialog
// fields that a future profile would need before it can be activated. They do
// not include, or depend on, SAMPFUNCS/SAMP-API headers.
using FixtureCommandProc = void (*)(char*);

#pragma pack(push, 1)
struct FixtureR3_1NetGame {
    std::uint8_t pad_0[44];
    void* rak_client;
    char host_address[257];
    char hostname[257];
    bool disable_collision;
    bool update_camera_target;
    bool nametag_status;
    std::int32_t port;
    std::int32_t lan_mode;
    std::uint32_t map_icons[100];
    std::int32_t game_state;
    std::uint32_t last_connect_attempt;
    void* settings;
    std::uint8_t pad_2[5];
    void* pools;
};

struct FixtureR3_1Pools {
    void* menu;
    void* actor;
    void* player;
    void* vehicle;
    void* pickup;
    void* object;
    void* gang_zone;
    void* label;
    void* textdraw;
};

struct FixtureR3_1TextLabel {
    char* text;
    std::uint32_t colour;
    float position[3];
    float draw_distance;
    std::uint8_t behind_walls;
    std::uint16_t attached_player;
    std::uint16_t attached_vehicle;
};

struct FixtureR3_1LabelPool {
    FixtureR3_1TextLabel labels[2048];
    std::int32_t not_empty[2048];
};

struct FixtureR3_1VehiclePoolPrefix {
    std::uint8_t pad[0x3074];
    std::int32_t not_empty[2000];
    void* game_objects[2000];
};

struct FixtureR3_1ObjectPoolPrefix {
    std::int32_t largest_id;
    std::int32_t not_empty[1000];
    void* objects[1000];
};

struct FixtureR3_1PickupPoolPrefix {
    std::int32_t count;
    std::int32_t handles[4096];
};

struct FixtureR3_1EntityPrefix {
    void* vtable;
    std::uint8_t pad[60];
    void* game_entity;
    std::int32_t handle;
};

struct FixtureR3_1Gangzone {
    float left;
    float bottom;
    float right;
    float top;
    std::uint32_t colour;
    std::uint32_t alternate_colour;
};

struct FixtureR3_1GangzonePool {
    FixtureR3_1Gangzone* gangzones[1024];
    std::int32_t not_empty[1024];
};

struct FixtureR3_1TextdrawData {
    float letter_width;
    float letter_height;
    std::uint32_t letter_colour;
    std::uint8_t unknown;
    std::uint8_t align_center;
    std::uint8_t box_enabled;
    float box_width;
    float box_height;
    std::uint32_t box_colour;
    std::uint8_t proportional;
    std::uint32_t background_colour;
    std::uint8_t shadow;
    std::uint8_t outline;
    std::uint8_t align_left;
    std::uint8_t align_right;
    std::int32_t style;
    float x;
    float y;
    std::uint8_t pad[8];
    std::uint32_t unknown_38;
    std::uint32_t unknown_3c;
    std::uint32_t unknown_40;
    std::uint8_t unknown_44;
    std::uint16_t model_id;
    float rotation[3];
    float zoom;
    std::uint16_t model_colour1;
    std::uint16_t model_colour2;
    std::uint8_t unknown_5b[3];
    std::uint32_t unknown_5e;
    std::uint32_t unknown_62;
    std::uint32_t unknown_66;
    std::uint32_t unknown_6a;
    std::uint8_t unknown_6e;
    std::uint32_t unknown_6f;
};

struct FixtureR3_1Textdraw {
    char text[801];
    char string[1602];
    FixtureR3_1TextdrawData data;
};

struct FixtureR3_1TextdrawPool {
    std::int32_t not_empty[2304];
    FixtureR3_1Textdraw* objects[2304];
};

struct FixtureR3_1AnimationEntry {
    char name_and_file[36];
};

struct FixtureR3_1String {
    std::uint8_t storage[0x18];
};

struct FixtureR3_1PlayerInfo {
    void* player;
    std::int32_t ping;
    std::int32_t align;
    FixtureR3_1String nickname;
    std::int32_t score;
    std::int32_t is_npc;
};

struct FixtureR3_1PlayerPool {
    std::int32_t largest_id;
    FixtureR3_1PlayerInfo* objects[1004];
    std::int32_t not_empty[1004];
    std::int32_t previous_collision[1004];
    struct {
        std::int32_t ping;
        std::int32_t score;
        std::uint16_t id;
        std::int32_t align;
        FixtureR3_1String name;
        void* player;
    } local_info;
};

struct FixtureR3_1LocalPlayerPrefix {
    void* ped;
    FixtureR1IncarData incar;
    FixtureR1AimData aim;
    FixtureR1TrailerData trailer;
    FixtureR1OnfootData onfoot;
    FixtureR1PassengerData passenger;
    std::int32_t active;
    std::int32_t wasted;
    std::uint16_t current_vehicle;
    std::uint16_t last_vehicle;
    std::uint32_t animation;
    std::int32_t field_1;
    std::int32_t does_spectating;
    std::uint8_t team;
    std::uint16_t field_10d;
    std::uint32_t last_update;
    std::uint32_t last_spec_update;
    std::uint32_t last_aim_update;
    std::uint32_t last_stats_update;
    std::uint8_t camera_target[8];
    std::uint32_t last_camera_target_update;
    std::uint8_t head[20];
    std::uint32_t last_any_update;
};

#pragma pack(push, 1)
struct FixtureR3_1RemotePlayerPrefix {
    void* ped;
    void* vehicle;
    std::uint16_t id;
    std::uint16_t vehicle_id;
    std::int32_t field_1;
    std::int32_t draw_labels;
    std::int32_t has_jetpack;
    std::uint8_t special_action;
    FixtureR1IncarData incar;
    FixtureR1TrailerData trailer;
    FixtureR1AimData aim;
    FixtureR1PassengerData passenger;
    FixtureR1OnfootData onfoot;
    std::uint8_t team;
    std::uint8_t state;
    std::uint8_t seat;
    std::int32_t field_3;
    std::int32_t passenger_drive_by;
    FixtureVector3 onfoot_target_position;
    FixtureVector3 onfoot_target_speed;
    FixtureVector3 incar_target_position;
    FixtureVector3 incar_target_speed;
    std::uint8_t pad_1[76];
    FixtureVector3 position_difference;
    struct {
        float real;
        FixtureVector3 imag;
    } incar_target_rotation;
    float reported_armour;
    float reported_health;
    std::uint8_t pad_2[12];
    std::uint32_t animation;
    std::uint8_t update_type;
    std::uint32_t last_update;
    std::uint32_t last_timestamp;
    std::int32_t performing_custom_animation;
    std::int32_t status;
};
#pragma pack(pop)

struct FixtureR3_1PedPrefix {
    std::uint8_t pad[0x2A4];
    void* game_ped;
};

struct FixtureR3_1Input {
    void* device;
    void* game_ui;
    void* editbox;
    FixtureCommandProc command_procedures[144];
    char command_names[144][33];
    std::int32_t command_count;
    std::int32_t enabled;
    char input_buffer[129];
    char recall_buffer[10][129];
    char current_buffer[129];
    std::int32_t current_recall;
    std::int32_t total_recalls;
    FixtureCommandProc default_command;
};

struct FixtureR3_1Dialog {
    void* device;
    std::uint32_t position[2];
    std::uint32_t size[2];
    std::uint32_t button_offset[2];
    void* dialog;
    void* listbox;
    void* editbox;
    std::int32_t active;
    std::int32_t type;
    std::uint32_t id;
    char* text;
    std::uint32_t text_size[2];
    char caption[65];
    std::int32_t server_side;
    std::uint8_t pad[536];
};

struct FixtureR3_1ListboxPrefix {
    std::uint8_t pad_0[0x143];
    std::int32_t selected;
    std::uint8_t pad_1[5];
    void* items;
    std::int32_t item_count;
};

struct FixtureR3_1ListboxItemPrefix {
    char text[256];
};

struct FixtureR3_1Scoreboard {
    std::int32_t is_enabled;
    std::int32_t player_count;
    float position[2];
    float scalar;
    float size[2];
    float pad[5];
    void* device;
    void* dialog;
    void* listbox;
    std::int32_t current_offset;
    std::int32_t is_sorted;
};

struct FixtureR3_1Game {
    void* audio;
    void* camera;
    void* player_ped;
    struct {
        FixtureVector3 current_position;
        FixtureVector3 next_position;
        float size;
        char type;
        std::int32_t enabled;
        std::int32_t marker;
        std::int32_t handle;
    } racing_checkpoint;
    struct {
        FixtureVector3 position;
        FixtureVector3 size;
        std::int32_t enabled;
        std::int32_t handle;
    } checkpoint;
    std::int32_t field_55;
    std::int32_t head_move;
    std::int32_t frame_limiter;
    std::int32_t cursor_mode;
    std::uint32_t input_enable_wait_frames;
    std::int32_t clock_enabled;
    char field_6d;
    bool keep_loaded_vehicle_models[212];
};

struct FixtureR5_1NetGame {
    void* rak_client;
    std::uint8_t pad_0[44];
    char host_address[257];
    char hostname[257];
    bool disable_collision;
    bool update_camera_target;
    bool nametag_status;
    std::int32_t port;
    std::int32_t lan_mode;
    std::uint32_t map_icons[100];
    std::int32_t game_state;
    std::uint32_t last_connect_attempt;
    void* settings;
    std::uint8_t pad_2[5];
    void* pools;
};

struct FixtureR5_1Input {
    void* device;
    void* game_ui;
    void* editbox;
    FixtureCommandProc command_procedures[144];
    char command_names[144][33];
    std::int32_t command_count;
    std::int32_t enabled;
    char input_buffer[129];
    char recall_buffer[10][129];
    char current_buffer[129];
    std::int32_t current_recall;
    std::int32_t total_recalls;
    FixtureCommandProc default_command;
};

struct FixtureR5_1Dialog {
    void* device;
    std::uint32_t position[2];
    std::uint32_t size[2];
    std::uint32_t button_offset[2];
    void* dialog;
    void* listbox;
    void* editbox;
    std::int32_t active;
    std::int32_t type;
    std::uint32_t id;
    char* text;
    std::uint32_t text_size[2];
    char caption[65];
    std::int32_t server_side;
    std::uint8_t pad[536];
};

#pragma pack(push, 1)
struct FixtureR5_1Pools {
    void* vehicle;
    void* player;
    void* pickup;
    void* object;
    void* actor;
    void* gangzone;
    void* label;
    void* textdraw;
    void* menu;
};

struct FixtureR5_1PlayerInfo {
    std::uint8_t pad_0[0x08];
    std::int32_t is_npc;
    std::uint8_t pad_0c[0x04];
    void* player;
    std::uint8_t pad_14[0x04];
    FixtureR3_1String nickname;
};

struct FixtureR5_1PlayerPool {
    std::uint8_t pad_0[0x04];
    std::uint16_t local_id;
    std::uint8_t pad_06[0x24];
    std::int32_t not_empty[1004];
    std::int32_t previous_collision[1004];
    FixtureR5_1PlayerInfo* objects[1004];
    std::int32_t largest_id;
};

struct FixtureR5_1RemotePlayer {
    std::uint8_t pad_0[0x0C];
    std::uint8_t special_action;
    std::uint8_t pad_0d[0x0C];
    FixtureR1IncarData incar;
    FixtureR1TrailerData trailer;
    FixtureR1AimData aim;
    FixtureR1PassengerData passenger;
    FixtureR1OnfootData onfoot;
    std::uint8_t pad_109[0xA3];
    float reported_armour;
    float reported_health;
    std::uint32_t animation;
    std::uint8_t pad_1b8[0x25];
    void* ped;
    std::uint8_t pad_1e1[0x1C];
};

struct FixtureR5_1LocalPlayer {
    FixtureR1IncarData incar;
    FixtureR1AimData aim;
    FixtureR1TrailerData trailer;
    FixtureR1OnfootData onfoot;
    FixtureR1PassengerData passenger;
    std::int32_t active;
    std::int32_t wasted;
    std::uint16_t current_vehicle;
    std::uint16_t last_vehicle;
    std::uint32_t animation;
    std::uint8_t pad_100[0x04];
    void* ped;
    std::uint8_t pad_108[0x37];
    std::uint32_t last_any_update;
    std::uint8_t pad_143[0x1E1];
};
#pragma pack(pop)

struct FixtureDlNetGame {
    std::uint8_t pad_0[44];
    void* rak_client;
    char host_address[257];
    char hostname[257];
    bool disable_collision;
    bool update_camera_target;
    bool nametag_status;
    std::uint32_t port;
    std::int32_t lan_mode;
    std::uint32_t map_icons[100];
    std::int32_t game_state;
    std::uint32_t last_connect_attempt;
    void* settings;
    std::uint8_t control_locked;
    std::int32_t unknown;
    void* pools;
};

struct FixtureDlInput {
    void* device;
    void* game_ui;
    void* editbox;
    FixtureCommandProc command_procedures[144];
    char command_names[144][33];
    std::int32_t command_count;
    std::int32_t enabled;
    char input_buffer[129];
    char recall_buffer[10][129];
    char current_buffer[129];
    std::int32_t current_recall;
    std::int32_t total_recalls;
    FixtureCommandProc default_command;
};

struct FixtureDlDialog {
    void* vtable;
    std::int32_t text_position[2];
    std::int32_t text_size[2];
    std::int32_t button_offset[2];
    void* dialog;
    void* listbox;
    void* editbox;
    std::int32_t active;
    std::int32_t type;
    std::uint32_t id;
    char* text;
    std::uint32_t font_size[2];
    char caption[65];
    std::int32_t server_side;
    std::uint8_t pad[536];
};

struct FixtureDlPools {
    void* menu;
    void* actor;
    void* player;
    void* vehicle;
    void* pickup;
    void* object;
    void* gangzone;
    void* label;
    void* textdraw;
};

struct FixtureDlSampString {
    std::uint8_t storage[0x1C];
};

struct FixtureDlPlayerInfo {
    std::int32_t score;
    std::int32_t is_npc;
    void* player;
    std::int32_t ping;
    FixtureDlSampString nickname;
};

struct FixtureDlPlayerPool {
    std::uint16_t local_id;
    FixtureDlSampString local_name;
    void* local_player;
    std::uint16_t largest_id;
    std::uint16_t unknown_24;
    FixtureDlPlayerInfo* players[1004];
    std::int32_t not_empty[1004];
    std::int32_t previous_collision[1004];
    std::int32_t local_ping;
    std::int32_t local_score;
};

struct FixtureDlLocalPlayer {
    void* ped;
    FixtureR1TrailerData trailer;
    FixtureR1OnfootData onfoot;
    FixtureR1PassengerData passenger;
    FixtureR1IncarData incar;
    FixtureR1AimData aim;
    std::int32_t active;
    std::uint8_t pad_f8[0x04];
    std::uint16_t current_vehicle;
    std::uint8_t pad_fe[0x12];
    std::uint32_t last_any_update;
    std::uint8_t pad_114[0x214];
};

struct FixtureDlRemotePlayer {
    std::uint16_t player_id;
    std::uint16_t vehicle_id;
    void* ped;
    void* vehicle;
    std::uint8_t pad_0c[0x0C];
    std::uint8_t special_action;
    std::uint8_t pad_19[0x0B];
    FixtureR1PassengerData passenger;
    FixtureR1OnfootData onfoot;
    FixtureR1IncarData incar;
    FixtureR1TrailerData trailer;
    FixtureR1AimData aim;
    std::uint8_t pad_114[0x98];
    float reported_armour;
    float reported_health;
    std::uint8_t pad_1b4[0x0C];
    std::uint32_t animation;
    std::uint8_t pad_1c4[0x39];
};

struct FixtureDlVehiclePool {
    std::uint8_t pad_0[0x3074];
    std::int32_t not_empty[2000];
    void* game_objects[2000];
    std::uint8_t pad_6ef4[0x109A4];
};

struct FixtureDlObjectPool {
    std::uint16_t largest_id;
    std::uint16_t pad_2;
    std::int32_t not_empty[2100];
    void* objects[2100];
};

struct FixtureDlPickupPool {
    std::int32_t count;
    std::int32_t handles[4096];
    std::uint8_t pad_4004[0x1F000];
};

struct FixtureDlEntity {
    void* vtable;
    std::uint8_t pad_4[0x3C];
    void* game_entity;
    std::int32_t handle;
};

struct FixtureDlPed {
    std::uint8_t pad_0[0x2A4];
    void* game_ped;
    std::uint8_t pad_2a8[0x85];
};

struct FixtureDlGangzone {
    float left;
    float bottom;
    float right;
    float top;
    std::uint32_t colour;
    std::uint32_t alternate_colour;
};

struct FixtureDlGangzonePool {
    FixtureDlGangzone* gangzones[1024];
    std::int32_t not_empty[1024];
};

struct FixtureDlTextLabel {
    char* text;
    std::uint32_t colour;
    float position[3];
    float draw_distance;
    std::uint8_t behind_walls;
    std::uint16_t attached_player;
    std::uint16_t attached_vehicle;
};

struct FixtureDlTextLabelPool {
    FixtureDlTextLabel labels[2048];
    std::int32_t not_empty[2048];
};

struct FixtureDlTextdrawTransmit {
    std::uint8_t pad_0[0x21];
    float x;
    float y;
    std::uint8_t pad_29[0x16];
};

struct FixtureDlTextdrawData {
    float letter_width;
    float letter_height;
    std::uint32_t letter_colour;
    std::uint8_t unknown;
    std::uint8_t align_center;
    std::uint8_t box_enabled;
    float box_width;
    float box_height;
    std::uint32_t box_colour;
    std::uint8_t proportional;
    std::uint32_t background_colour;
    std::uint8_t shadow;
    std::uint8_t outline;
    std::uint8_t align_left;
    std::uint8_t align_right;
    std::int32_t style;
    float x;
    float y;
    std::uint8_t pad_30[0x15];
    std::uint16_t model_id;
    float rotation[3];
    float zoom;
    std::uint16_t model_colour1;
    std::uint16_t model_colour2;
    std::uint8_t pad_5b[0x18];
};

struct FixtureDlTextdraw {
    char text[801];
    char string[1602];
    FixtureDlTextdrawData data;
};

struct FixtureDlTextdrawPool {
    std::int32_t not_empty[2304];
    FixtureDlTextdraw* objects[2304];
};

struct FixtureDlChatEntry {
    std::int32_t type;
    char prefix[28];
    char text[144];
    std::uint8_t pad_b0[0x44];
    std::uint32_t text_colour;
    std::uint32_t prefix_colour;
};

struct FixtureDlChat {
    std::uint8_t pad_0[0x08];
    std::int32_t mode;
    std::uint8_t pad_0c[0x126];
    FixtureDlChatEntry entries[100];
    std::uint8_t pad_63a2[0x48];
};

struct FixtureDlScoreboard {
    std::int32_t enabled;
    std::uint8_t pad_4[0x40];
};

struct FixtureDlGamePrefix {
    std::uint8_t pad_0[0x61];
    std::int32_t cursor_mode;
};

struct FixtureDlListboxPrefix {
    std::uint8_t pad_0[0x143];
    std::int32_t selected;
    std::uint8_t pad_147[0x05];
    void* items;
    std::int32_t item_count;
};

struct FixtureDlListboxItemPrefix {
    char text[256];
};

struct FixtureDlAnimationEntry {
    char name_and_file[36];
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
static_assert(offsetof(FixtureR1RemotePlayerPrefix, ped) == 0x00);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, onfoot) == 0xC8);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, incar) == 0x10C);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, passenger) == 0x181);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, trailer) == 0x14B);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, reported_armour) == 0x1B8);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, reported_health) == 0x1BC);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, animation) == 0x1C0);
static_assert(offsetof(FixtureR1RemotePlayerPrefix, status) == 0x1D1);
static_assert(sizeof(FixtureR1LocalPlayerPrefix) == 92);
static_assert(offsetof(FixtureR1LocalPlayerSyncPrefix, incar) == 0xAA);
static_assert(offsetof(FixtureR1LocalPlayerSyncPrefix, passenger) == 0x5C);
static_assert(offsetof(FixtureR1LocalPlayerSyncPrefix, trailer) == 0x74);
static_assert(offsetof(FixtureR1Ped, game_ped) == 0x2A4);
static_assert(offsetof(FixtureR1PlayerPoolPrefix, local_id) == 0x04);
static_assert(sizeof(FixtureR1VehicleInfo) == 40);
static_assert(offsetof(FixtureR1VehiclePoolExistsPrefix, not_empty) == 0x3074);
static_assert(offsetof(FixtureR1VehiclePoolExistsPrefix, game_objects) == 0x4FB4);
static_assert(offsetof(FixtureR1ObjectPoolExistsPrefix, objects) == 0xFA4);
static_assert(offsetof(FixtureR1PickupPoolHandlesPrefix, handles) == 0x04);
static_assert(offsetof(FixtureR1Entity, handle) == 0x44);
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
static_assert(sizeof(FixtureR1TextDrawTransmit) == 0x3F);
static_assert(offsetof(FixtureR1TextDrawTransmit, x) == 0x21);
static_assert(offsetof(FixtureR1TextDrawTransmit, y) == 0x25);
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
static_assert(offsetof(FixtureR1DialogSnapshot, editbox) == 0x24);
static_assert(offsetof(FixtureR1DialogSnapshot, text) == 0x34);
static_assert(offsetof(FixtureDxutListBoxSelection, selected) == 0x143);
static_assert(offsetof(FixtureDxutListBoxSelection, items) == 0x14C);
static_assert(offsetof(FixtureDxutListBoxSelection, item_count) == 0x150);
static_assert(offsetof(FixtureDxutComboBoxItem, str_text) == 0x00);
static_assert(offsetof(FixtureDxutComboBoxItem, data) == 0x100);
static_assert(offsetof(FixtureDxutComboBoxItem, active_rect) == 0x104);
static_assert(offsetof(FixtureDxutComboBoxItem, visible) == 0x114);
static_assert(sizeof(FixtureDxutComboBoxItem::str_text) == 256);
static_assert(sizeof(FixtureDxutComboBoxItem::active_rect) == 16);
static_assert(sizeof(FixtureDxutComboBoxItem) == 0x118);
static_assert(offsetof(FixtureR1DialogSnapshot, type) == 0x2C);
static_assert(offsetof(FixtureR1DialogSnapshot, id) == 0x30);
static_assert(offsetof(FixtureR1DialogSnapshot, caption) == 0x40);
static_assert(offsetof(FixtureR1DialogSnapshot, server_side) == 0x81);
static_assert(offsetof(FixtureR1InputPrefix, is_enabled) == 0x14E0);
static_assert(sizeof(FixtureR3_1NetGame) == 0x3E2);
static_assert(offsetof(FixtureR3_1NetGame, rak_client) == 0x2C);
static_assert(offsetof(FixtureR3_1NetGame, host_address) == 0x30);
static_assert(offsetof(FixtureR3_1NetGame, hostname) == 0x131);
static_assert(offsetof(FixtureR3_1NetGame, port) == 0x235);
static_assert(offsetof(FixtureR3_1NetGame, game_state) == 0x3CD);
static_assert(offsetof(FixtureR3_1NetGame, pools) == 0x3DE);
static_assert(sizeof(FixtureR3_1Pools) == 0x24);
static_assert(offsetof(FixtureR3_1Pools, label) == 0x1C);
static_assert(offsetof(FixtureR3_1Pools, object) == 0x14);
static_assert(offsetof(FixtureR3_1Pools, gang_zone) == 0x18);
static_assert(offsetof(FixtureR3_1Pools, pickup) == 0x10);
static_assert(sizeof(FixtureR3_1TextLabel) == 0x1D);
static_assert(offsetof(FixtureR3_1LabelPool, not_empty) == 0xE800);
static_assert(offsetof(FixtureR3_1VehiclePoolPrefix, not_empty) == 0x3074);
static_assert(offsetof(FixtureR3_1VehiclePoolPrefix, game_objects) == 0x4FB4);
static_assert(offsetof(FixtureR3_1ObjectPoolPrefix, not_empty) == 0x04);
static_assert(offsetof(FixtureR3_1ObjectPoolPrefix, objects) == 0xFA4);
static_assert(offsetof(FixtureR3_1PickupPoolPrefix, handles) == 0x04);
static_assert(offsetof(FixtureR3_1EntityPrefix, handle) == 0x44);
static_assert(sizeof(FixtureR3_1Gangzone) == 0x18);
static_assert(offsetof(FixtureR3_1GangzonePool, not_empty) == 0x1000);
static_assert(sizeof(FixtureR3_1GangzonePool) == 0x2000);
static_assert(sizeof(FixtureR3_1TextdrawData) == 0x73);
static_assert(offsetof(FixtureR3_1TextdrawData, letter_width) == 0x00);
static_assert(offsetof(FixtureR3_1TextdrawData, proportional) == 0x1B);
static_assert(offsetof(FixtureR3_1TextdrawData, background_colour) == 0x1C);
static_assert(offsetof(FixtureR3_1TextdrawData, shadow) == 0x20);
static_assert(offsetof(FixtureR3_1TextdrawData, outline) == 0x21);
static_assert(offsetof(FixtureR3_1TextdrawData, align_left) == 0x22);
static_assert(offsetof(FixtureR3_1TextdrawData, align_right) == 0x23);
static_assert(offsetof(FixtureR3_1TextdrawData, style) == 0x24);
static_assert(offsetof(FixtureR3_1TextdrawData, x) == 0x28);
static_assert(offsetof(FixtureR3_1TextdrawData, box_enabled) == 0x0E);
static_assert(offsetof(FixtureR3_1TextdrawData, box_width) == 0x0F);
static_assert(offsetof(FixtureR3_1TextdrawData, box_height) == 0x13);
static_assert(offsetof(FixtureR3_1TextdrawData, box_colour) == 0x17);
static_assert(offsetof(FixtureR3_1TextdrawData, model_id) == 0x45);
static_assert(offsetof(FixtureR3_1TextdrawData, rotation) == 0x47);
static_assert(offsetof(FixtureR3_1TextdrawData, zoom) == 0x53);
static_assert(offsetof(FixtureR3_1TextdrawData, model_colour1) == 0x57);
static_assert(offsetof(FixtureR3_1TextdrawData, model_colour2) == 0x59);
static_assert(sizeof(FixtureR3_1Textdraw) == 0x9D6);
static_assert(offsetof(FixtureR3_1Textdraw, string) == 0x321);
static_assert(offsetof(FixtureR3_1Textdraw, data) == 0x963);
static_assert(offsetof(FixtureR3_1TextdrawPool, objects) == 0x2400);
static_assert(sizeof(FixtureR3_1TextdrawPool) == 0x4800);
static_assert(sizeof(FixtureR3_1AnimationEntry) == 0x24);
static_assert(sizeof(FixtureR3_1String) == 0x18);
static_assert(sizeof(FixtureR3_1PlayerInfo) == 0x2C);
static_assert(offsetof(FixtureR3_1PlayerInfo, is_npc) == 0x28);
static_assert(sizeof(FixtureR3_1PlayerPool) == 0x2F3E);
static_assert(offsetof(FixtureR3_1PlayerPool, largest_id) == 0x00);
static_assert(offsetof(FixtureR3_1PlayerPool, objects) == 0x04);
static_assert(offsetof(FixtureR3_1PlayerPool, not_empty) == 0x0FB4);
static_assert(offsetof(FixtureR3_1PlayerPool, previous_collision) == 0x1F64);
static_assert(offsetof(FixtureR3_1PlayerPool, local_info.ping) == 0x2F14);
static_assert(offsetof(FixtureR3_1PlayerPool, local_info.score) == 0x2F18);
static_assert(offsetof(FixtureR3_1PlayerPool, local_info.id) == 0x2F1C);
static_assert(offsetof(FixtureR3_1RemotePlayerPrefix, special_action) == 0x18);
static_assert(offsetof(FixtureR3_1RemotePlayerPrefix, incar) == 0x19);
static_assert(offsetof(FixtureR3_1RemotePlayerPrefix, trailer) == 0x58);
static_assert(offsetof(FixtureR3_1RemotePlayerPrefix, aim) == 0x8E);
static_assert(offsetof(FixtureR3_1RemotePlayerPrefix, passenger) == 0xAD);
static_assert(offsetof(FixtureR3_1RemotePlayerPrefix, onfoot) == 0xC5);
static_assert(offsetof(FixtureR3_1RemotePlayerPrefix, reported_armour) == 0x1AC);
static_assert(offsetof(FixtureR3_1RemotePlayerPrefix, reported_health) == 0x1B0);
static_assert(offsetof(FixtureR3_1RemotePlayerPrefix, animation) == 0x1C0);
static_assert(offsetof(FixtureR3_1RemotePlayerPrefix, status) == 0x1D1);
static_assert(offsetof(FixtureR3_1LocalPlayerPrefix, incar) == 0x04);
static_assert(offsetof(FixtureR3_1LocalPlayerPrefix, onfoot) == 0x98);
static_assert(offsetof(FixtureR3_1LocalPlayerPrefix, active) == 0xF4);
static_assert(offsetof(FixtureR3_1LocalPlayerPrefix, current_vehicle) == 0xFC);
static_assert(offsetof(FixtureR3_1LocalPlayerPrefix, last_any_update) == 0x13F);
static_assert(offsetof(FixtureR3_1PedPrefix, game_ped) == 0x2A4);
static_assert(sizeof(FixtureR3_1Input) == 0x1AFC);
static_assert(offsetof(FixtureR3_1Input, editbox) == 0x08);
static_assert(offsetof(FixtureR3_1Input, command_count) == 0x14DC);
static_assert(offsetof(FixtureR3_1Input, enabled) == 0x14E0);
static_assert(offsetof(FixtureR3_1Input, command_names) == 0x24C);
static_assert(sizeof(FixtureR3_1Input::command_names[0]) == 33);
static_assert(sizeof(FixtureR3_1Dialog) == 0x29D);
static_assert(offsetof(FixtureR3_1Dialog, active) == 0x28);
static_assert(offsetof(FixtureR3_1Dialog, listbox) == 0x20);
static_assert(offsetof(FixtureR3_1Dialog, editbox) == 0x24);
static_assert(offsetof(FixtureR3_1Dialog, type) == 0x2C);
static_assert(offsetof(FixtureR3_1Dialog, id) == 0x30);
static_assert(offsetof(FixtureR3_1Dialog, text) == 0x34);
static_assert(offsetof(FixtureR3_1Dialog, caption) == 0x40);
static_assert(offsetof(FixtureR3_1Dialog, server_side) == 0x81);
static_assert(offsetof(FixtureR3_1ListboxPrefix, selected) == 0x143);
static_assert(offsetof(FixtureR3_1ListboxPrefix, items) == 0x14C);
static_assert(offsetof(FixtureR3_1ListboxPrefix, item_count) == 0x150);
static_assert(sizeof(FixtureR3_1ListboxItemPrefix::text) == 256);
static_assert(sizeof(FixtureR3_1Scoreboard) == 0x44);
static_assert(offsetof(FixtureR3_1Scoreboard, is_enabled) == 0x00);
static_assert(sizeof(FixtureR3_1Game) == 0x142);
static_assert(offsetof(FixtureR3_1Game, cursor_mode) == 0x61);
static_assert(sizeof(FixtureR5_1NetGame) == 0x3E2);
static_assert(offsetof(FixtureR5_1NetGame, rak_client) == 0x00);
static_assert(offsetof(FixtureR5_1NetGame, game_state) == 0x3CD);
static_assert(offsetof(FixtureR5_1NetGame, pools) == 0x3DE);
static_assert(sizeof(FixtureR5_1Input) == 0x1AFC);
static_assert(offsetof(FixtureR5_1Input, command_count) == 0x14DC);
static_assert(offsetof(FixtureR5_1Input, enabled) == 0x14E0);
static_assert(sizeof(FixtureR5_1Dialog) == 0x29D);
static_assert(offsetof(FixtureR5_1Dialog, active) == 0x28);
static_assert(offsetof(FixtureR5_1Dialog, caption) == 0x40);
static_assert(sizeof(FixtureR5_1Pools) == 0x24);
static_assert(offsetof(FixtureR5_1Pools, pickup) == 0x08);
static_assert(offsetof(FixtureR5_1Pools, object) == 0x0C);
static_assert(offsetof(FixtureR5_1Pools, gangzone) == 0x14);
static_assert(offsetof(FixtureR5_1Pools, label) == 0x18);
static_assert(offsetof(FixtureR5_1Pools, textdraw) == 0x1C);
static_assert(sizeof(FixtureR5_1PlayerInfo) == 0x30);
static_assert(offsetof(FixtureR5_1PlayerInfo, is_npc) == 0x08);
static_assert(offsetof(FixtureR5_1PlayerInfo, player) == 0x10);
static_assert(offsetof(FixtureR5_1PlayerInfo, nickname) == 0x18);
static_assert(sizeof(FixtureR5_1PlayerPool) == 0x2F3E);
static_assert(offsetof(FixtureR5_1PlayerPool, local_id) == 0x04);
static_assert(offsetof(FixtureR5_1PlayerPool, not_empty) == 0x2A);
static_assert(offsetof(FixtureR5_1PlayerPool, previous_collision) == 0xFDA);
static_assert(offsetof(FixtureR5_1PlayerPool, objects) == 0x1F8A);
static_assert(offsetof(FixtureR5_1PlayerPool, largest_id) == 0x2F3A);
static_assert(sizeof(FixtureR5_1RemotePlayer) == 0x1FD);
static_assert(offsetof(FixtureR5_1RemotePlayer, special_action) == 0x0C);
static_assert(offsetof(FixtureR5_1RemotePlayer, incar) == 0x19);
static_assert(offsetof(FixtureR5_1RemotePlayer, trailer) == 0x58);
static_assert(offsetof(FixtureR5_1RemotePlayer, aim) == 0x8E);
static_assert(offsetof(FixtureR5_1RemotePlayer, passenger) == 0xAD);
static_assert(offsetof(FixtureR5_1RemotePlayer, onfoot) == 0xC5);
static_assert(offsetof(FixtureR5_1RemotePlayer, reported_armour) == 0x1AC);
static_assert(offsetof(FixtureR5_1RemotePlayer, reported_health) == 0x1B0);
static_assert(offsetof(FixtureR5_1RemotePlayer, animation) == 0x1B4);
static_assert(offsetof(FixtureR5_1RemotePlayer, ped) == 0x1DD);
static_assert(sizeof(FixtureR5_1LocalPlayer) == 0x324);
static_assert(offsetof(FixtureR5_1LocalPlayer, incar) == 0x00);
static_assert(offsetof(FixtureR5_1LocalPlayer, aim) == 0x3F);
static_assert(offsetof(FixtureR5_1LocalPlayer, trailer) == 0x5E);
static_assert(offsetof(FixtureR5_1LocalPlayer, onfoot) == 0x94);
static_assert(offsetof(FixtureR5_1LocalPlayer, passenger) == 0xD8);
static_assert(offsetof(FixtureR5_1LocalPlayer, active) == 0xF0);
static_assert(offsetof(FixtureR5_1LocalPlayer, current_vehicle) == 0xF8);
static_assert(offsetof(FixtureR5_1LocalPlayer, ped) == 0x104);
static_assert(offsetof(FixtureR5_1LocalPlayer, last_any_update) == 0x13F);
static_assert(sizeof(FixtureDlNetGame) == 0x3E2);
static_assert(offsetof(FixtureDlNetGame, rak_client) == 0x2C);
static_assert(offsetof(FixtureDlNetGame, game_state) == 0x3CD);
static_assert(offsetof(FixtureDlNetGame, pools) == 0x3DE);
static_assert(sizeof(FixtureDlInput) == 0x1AFC);
static_assert(offsetof(FixtureDlInput, command_count) == 0x14DC);
static_assert(offsetof(FixtureDlInput, enabled) == 0x14E0);
static_assert(sizeof(FixtureDlDialog) == 0x29D);
static_assert(offsetof(FixtureDlDialog, active) == 0x28);
static_assert(offsetof(FixtureDlDialog, caption) == 0x40);
static_assert(sizeof(FixtureDlPools) == 0x24);
static_assert(offsetof(FixtureDlPools, pickup) == 0x10);
static_assert(offsetof(FixtureDlPools, object) == 0x14);
static_assert(offsetof(FixtureDlPools, gangzone) == 0x18);
static_assert(offsetof(FixtureDlPools, label) == 0x1C);
static_assert(offsetof(FixtureDlPools, textdraw) == 0x20);
static_assert(sizeof(FixtureDlSampString) == 0x1C);
static_assert(sizeof(FixtureDlPlayerInfo) == 0x2C);
static_assert(offsetof(FixtureDlPlayerInfo, score) == 0x00);
static_assert(offsetof(FixtureDlPlayerInfo, is_npc) == 0x04);
static_assert(offsetof(FixtureDlPlayerInfo, player) == 0x08);
static_assert(offsetof(FixtureDlPlayerInfo, ping) == 0x0C);
static_assert(offsetof(FixtureDlPlayerInfo, nickname) == 0x10);
static_assert(sizeof(FixtureDlPlayerPool) == 0x2F3E);
static_assert(offsetof(FixtureDlPlayerPool, local_id) == 0x00);
static_assert(offsetof(FixtureDlPlayerPool, local_name) == 0x02);
static_assert(offsetof(FixtureDlPlayerPool, local_player) == 0x1E);
static_assert(offsetof(FixtureDlPlayerPool, largest_id) == 0x22);
static_assert(offsetof(FixtureDlPlayerPool, unknown_24) == 0x24);
static_assert(offsetof(FixtureDlPlayerPool, players) == 0x26);
static_assert(offsetof(FixtureDlPlayerPool, not_empty) == 0xFD6);
static_assert(offsetof(FixtureDlPlayerPool, previous_collision) == 0x1F86);
static_assert(offsetof(FixtureDlPlayerPool, local_ping) == 0x2F36);
static_assert(offsetof(FixtureDlPlayerPool, local_score) == 0x2F3A);
static_assert(sizeof(FixtureDlLocalPlayer) == 0x328);
static_assert(offsetof(FixtureDlLocalPlayer, ped) == 0x00);
static_assert(offsetof(FixtureDlLocalPlayer, trailer) == 0x04);
static_assert(offsetof(FixtureDlLocalPlayer, onfoot) == 0x3A);
static_assert(offsetof(FixtureDlLocalPlayer, passenger) == 0x7E);
static_assert(offsetof(FixtureDlLocalPlayer, incar) == 0x96);
static_assert(offsetof(FixtureDlLocalPlayer, aim) == 0xD5);
static_assert(offsetof(FixtureDlLocalPlayer, active) == 0xF4);
static_assert(offsetof(FixtureDlLocalPlayer, current_vehicle) == 0xFC);
static_assert(offsetof(FixtureDlLocalPlayer, last_any_update) == 0x110);
static_assert(sizeof(FixtureDlRemotePlayer) == 0x1FD);
static_assert(offsetof(FixtureDlRemotePlayer, player_id) == 0x00);
static_assert(offsetof(FixtureDlRemotePlayer, vehicle_id) == 0x02);
static_assert(offsetof(FixtureDlRemotePlayer, ped) == 0x04);
static_assert(offsetof(FixtureDlRemotePlayer, vehicle) == 0x08);
static_assert(offsetof(FixtureDlRemotePlayer, special_action) == 0x18);
static_assert(offsetof(FixtureDlRemotePlayer, passenger) == 0x24);
static_assert(offsetof(FixtureDlRemotePlayer, onfoot) == 0x3C);
static_assert(offsetof(FixtureDlRemotePlayer, incar) == 0x80);
static_assert(offsetof(FixtureDlRemotePlayer, trailer) == 0xBF);
static_assert(offsetof(FixtureDlRemotePlayer, aim) == 0xF5);
static_assert(offsetof(FixtureDlRemotePlayer, reported_armour) == 0x1AC);
static_assert(offsetof(FixtureDlRemotePlayer, reported_health) == 0x1B0);
static_assert(offsetof(FixtureDlRemotePlayer, animation) == 0x1C0);
static_assert(sizeof(FixtureDlVehiclePool) == 0x17898);
static_assert(offsetof(FixtureDlVehiclePool, not_empty) == 0x3074);
static_assert(offsetof(FixtureDlVehiclePool, game_objects) == 0x4FB4);
static_assert(sizeof(FixtureDlObjectPool) == 0x41A4);
static_assert(offsetof(FixtureDlObjectPool, not_empty) == 0x04);
static_assert(offsetof(FixtureDlObjectPool, objects) == 0x20D4);
static_assert(sizeof(FixtureDlPickupPool) == 0x23004);
static_assert(offsetof(FixtureDlPickupPool, handles) == 0x04);
static_assert(sizeof(FixtureDlEntity) == 0x48);
static_assert(offsetof(FixtureDlEntity, handle) == 0x44);
static_assert(sizeof(FixtureDlPed) == 0x32D);
static_assert(offsetof(FixtureDlPed, game_ped) == 0x2A4);
static_assert(sizeof(FixtureDlGangzone) == 0x18);
static_assert(sizeof(FixtureDlGangzonePool) == 0x2000);
static_assert(offsetof(FixtureDlGangzonePool, not_empty) == 0x1000);
static_assert(sizeof(FixtureDlTextLabel) == 0x1D);
static_assert(sizeof(FixtureDlTextLabelPool) == 0x10800);
static_assert(offsetof(FixtureDlTextLabelPool, not_empty) == 0xE800);
static_assert(sizeof(FixtureDlTextdrawTransmit) == 0x3F);
static_assert(offsetof(FixtureDlTextdrawTransmit, x) == 0x21);
static_assert(offsetof(FixtureDlTextdrawTransmit, y) == 0x25);
static_assert(sizeof(FixtureDlTextdrawData) == 0x73);
static_assert(offsetof(FixtureDlTextdrawData, proportional) == 0x1B);
static_assert(offsetof(FixtureDlTextdrawData, style) == 0x24);
static_assert(offsetof(FixtureDlTextdrawData, x) == 0x28);
static_assert(offsetof(FixtureDlTextdrawData, model_id) == 0x45);
static_assert(sizeof(FixtureDlTextdraw) == 0x9D6);
static_assert(offsetof(FixtureDlTextdraw, string) == 0x321);
static_assert(offsetof(FixtureDlTextdraw, data) == 0x963);
static_assert(sizeof(FixtureDlTextdrawPool) == 0x4800);
static_assert(offsetof(FixtureDlTextdrawPool, objects) == 0x2400);
static_assert(sizeof(FixtureDlChatEntry) == 0xFC);
static_assert(offsetof(FixtureDlChatEntry, prefix) == 0x04);
static_assert(offsetof(FixtureDlChatEntry, text) == 0x20);
static_assert(offsetof(FixtureDlChatEntry, text_colour) == 0xF4);
static_assert(offsetof(FixtureDlChatEntry, prefix_colour) == 0xF8);
static_assert(sizeof(FixtureDlChat) == 0x63EA);
static_assert(offsetof(FixtureDlChat, mode) == 0x08);
static_assert(offsetof(FixtureDlChat, entries) == 0x132);
static_assert(sizeof(FixtureDlScoreboard) == 0x44);
static_assert(offsetof(FixtureDlScoreboard, enabled) == 0x00);
static_assert(sizeof(FixtureDlGamePrefix) == 0x65);
static_assert(offsetof(FixtureDlGamePrefix, cursor_mode) == 0x61);
static_assert(offsetof(FixtureDlListboxPrefix, selected) == 0x143);
static_assert(offsetof(FixtureDlListboxPrefix, items) == 0x14C);
static_assert(offsetof(FixtureDlListboxPrefix, item_count) == 0x150);
static_assert(sizeof(FixtureDlListboxItemPrefix::text) == 256);
static_assert(sizeof(FixtureDlAnimationEntry) == 0x24);

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

std::size_t samp_client_sdk_fixture_r1_remote_onfoot_offset() {
    return offsetof(FixtureR1RemotePlayerPrefix, onfoot);
}

std::size_t samp_client_sdk_fixture_r1_local_incar_offset() {
    return offsetof(FixtureR1LocalPlayerSyncPrefix, incar);
}

std::size_t samp_client_sdk_fixture_r1_remote_incar_offset() {
    return offsetof(FixtureR1RemotePlayerPrefix, incar);
}

std::size_t samp_client_sdk_fixture_r1_local_passenger_offset() {
    return offsetof(FixtureR1LocalPlayerSyncPrefix, passenger);
}

std::size_t samp_client_sdk_fixture_r1_remote_passenger_offset() {
    return offsetof(FixtureR1RemotePlayerPrefix, passenger);
}

std::size_t samp_client_sdk_fixture_r1_local_trailer_offset() {
    return offsetof(FixtureR1LocalPlayerSyncPrefix, trailer);
}

std::size_t samp_client_sdk_fixture_r1_remote_trailer_offset() {
    return offsetof(FixtureR1RemotePlayerPrefix, trailer);
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

std::size_t samp_client_sdk_fixture_r1_vehicle_pool_game_objects_offset() {
    return offsetof(FixtureR1VehiclePoolExistsPrefix, game_objects);
}

std::size_t samp_client_sdk_fixture_r1_object_pool_objects_offset() {
    return offsetof(FixtureR1ObjectPoolExistsPrefix, objects);
}

std::size_t samp_client_sdk_fixture_r1_pickup_pool_handles_offset() {
    return offsetof(FixtureR1PickupPoolHandlesPrefix, handles);
}

std::size_t samp_client_sdk_fixture_r1_entity_handle_offset() {
    return offsetof(FixtureR1Entity, handle);
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

std::size_t samp_client_sdk_fixture_r1_textdraw_transmit_size() {
    return sizeof(FixtureR1TextDrawTransmit);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_transmit_x_offset() {
    return offsetof(FixtureR1TextDrawTransmit, x);
}

std::size_t samp_client_sdk_fixture_r1_textdraw_transmit_y_offset() {
    return offsetof(FixtureR1TextDrawTransmit, y);
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

std::size_t samp_client_sdk_fixture_r1_dialog_editbox_offset() {
    return offsetof(FixtureR1DialogSnapshot, editbox);
}

std::size_t samp_client_sdk_fixture_r1_dialog_text_offset() {
    return offsetof(FixtureR1DialogSnapshot, text);
}

std::size_t samp_client_sdk_fixture_dxut_listbox_items_offset() {
    return offsetof(FixtureDxutListBoxSelection, items);
}

std::size_t samp_client_sdk_fixture_dxut_combobox_item_text_offset() {
    return offsetof(FixtureDxutComboBoxItem, str_text);
}

std::size_t samp_client_sdk_fixture_dxut_combobox_item_text_capacity() {
    return sizeof(FixtureDxutComboBoxItem::str_text);
}

std::size_t samp_client_sdk_fixture_dxut_combobox_item_data_offset() {
    return offsetof(FixtureDxutComboBoxItem, data);
}

std::size_t samp_client_sdk_fixture_dxut_combobox_item_active_rect_offset() {
    return offsetof(FixtureDxutComboBoxItem, active_rect);
}

std::size_t samp_client_sdk_fixture_dxut_combobox_item_visible_offset() {
    return offsetof(FixtureDxutComboBoxItem, visible);
}

std::size_t samp_client_sdk_fixture_dxut_combobox_item_size() {
    return sizeof(FixtureDxutComboBoxItem);
}

std::size_t samp_client_sdk_fixture_r1_input_enabled_offset() {
    return offsetof(FixtureR1InputPrefix, is_enabled);
}

std::size_t samp_client_sdk_fixture_r3_1_netgame_size() {
    return sizeof(FixtureR3_1NetGame);
}

std::size_t samp_client_sdk_fixture_r3_1_netgame_rak_client_offset() {
    return offsetof(FixtureR3_1NetGame, rak_client);
}

std::size_t samp_client_sdk_fixture_r3_1_netgame_host_address_offset() {
    return offsetof(FixtureR3_1NetGame, host_address);
}

std::size_t samp_client_sdk_fixture_r3_1_netgame_hostname_offset() {
    return offsetof(FixtureR3_1NetGame, hostname);
}

std::size_t samp_client_sdk_fixture_r3_1_netgame_port_offset() {
    return offsetof(FixtureR3_1NetGame, port);
}

std::size_t samp_client_sdk_fixture_r3_1_netgame_game_state_offset() {
    return offsetof(FixtureR3_1NetGame, game_state);
}

std::size_t samp_client_sdk_fixture_r3_1_netgame_pools_offset() {
    return offsetof(FixtureR3_1NetGame, pools);
}

std::size_t samp_client_sdk_fixture_r3_1_pools_label_offset() {
    return offsetof(FixtureR3_1Pools, label);
}

std::size_t samp_client_sdk_fixture_r3_1_text_label_size() {
    return sizeof(FixtureR3_1TextLabel);
}

std::size_t samp_client_sdk_fixture_r3_1_label_pool_not_empty_offset() {
    return offsetof(FixtureR3_1LabelPool, not_empty);
}

std::size_t samp_client_sdk_fixture_r3_1_vehicle_pool_not_empty_offset() {
    return offsetof(FixtureR3_1VehiclePoolPrefix, not_empty);
}

std::size_t samp_client_sdk_fixture_r3_1_vehicle_pool_game_objects_offset() {
    return offsetof(FixtureR3_1VehiclePoolPrefix, game_objects);
}

std::size_t samp_client_sdk_fixture_r3_1_pools_object_offset() {
    return offsetof(FixtureR3_1Pools, object);
}

std::size_t samp_client_sdk_fixture_r3_1_object_pool_not_empty_offset() {
    return offsetof(FixtureR3_1ObjectPoolPrefix, not_empty);
}

std::size_t samp_client_sdk_fixture_r3_1_object_pool_objects_offset() {
    return offsetof(FixtureR3_1ObjectPoolPrefix, objects);
}

std::size_t samp_client_sdk_fixture_r3_1_pools_pickup_offset() {
    return offsetof(FixtureR3_1Pools, pickup);
}

std::size_t samp_client_sdk_fixture_r3_1_pickup_pool_handles_offset() {
    return offsetof(FixtureR3_1PickupPoolPrefix, handles);
}

std::size_t samp_client_sdk_fixture_r3_1_entity_handle_offset() {
    return offsetof(FixtureR3_1EntityPrefix, handle);
}

std::size_t samp_client_sdk_fixture_r3_1_pools_gangzone_offset() {
    return offsetof(FixtureR3_1Pools, gang_zone);
}

std::size_t samp_client_sdk_fixture_r3_1_gangzone_size() {
    return sizeof(FixtureR3_1Gangzone);
}

std::size_t samp_client_sdk_fixture_r3_1_gangzone_pool_not_empty_offset() {
    return offsetof(FixtureR3_1GangzonePool, not_empty);
}

std::size_t samp_client_sdk_fixture_r3_1_pools_textdraw_offset() {
    return offsetof(FixtureR3_1Pools, textdraw);
}

std::size_t samp_client_sdk_fixture_r3_1_textdraw_size() {
    return sizeof(FixtureR3_1Textdraw);
}

std::size_t samp_client_sdk_fixture_r3_1_textdraw_pool_objects_offset() {
    return offsetof(FixtureR3_1TextdrawPool, objects);
}

std::size_t samp_client_sdk_fixture_r3_1_animation_entry_size() {
    return sizeof(FixtureR3_1AnimationEntry);
}

std::size_t samp_client_sdk_fixture_r3_1_player_pool_local_id_offset() {
    return offsetof(FixtureR3_1PlayerPool, local_info.id);
}

std::size_t samp_client_sdk_fixture_r3_1_player_pool_size() {
    return sizeof(FixtureR3_1PlayerPool);
}

std::size_t samp_client_sdk_fixture_r3_1_player_pool_largest_id_offset() {
    return offsetof(FixtureR3_1PlayerPool, largest_id);
}

std::size_t samp_client_sdk_fixture_r3_1_player_pool_objects_offset() {
    return offsetof(FixtureR3_1PlayerPool, objects);
}

std::size_t samp_client_sdk_fixture_r3_1_player_info_size() {
    return sizeof(FixtureR3_1PlayerInfo);
}

std::size_t samp_client_sdk_fixture_r3_1_player_info_is_npc_offset() {
    return offsetof(FixtureR3_1PlayerInfo, is_npc);
}

std::size_t samp_client_sdk_fixture_r3_1_remote_player_special_action_offset() {
    return offsetof(FixtureR3_1RemotePlayerPrefix, special_action);
}

std::size_t samp_client_sdk_fixture_r3_1_remote_player_reported_armour_offset() {
    return offsetof(FixtureR3_1RemotePlayerPrefix, reported_armour);
}

std::size_t samp_client_sdk_fixture_r3_1_remote_player_reported_health_offset() {
    return offsetof(FixtureR3_1RemotePlayerPrefix, reported_health);
}

std::size_t samp_client_sdk_fixture_r3_1_remote_player_animation_offset() {
    return offsetof(FixtureR3_1RemotePlayerPrefix, animation);
}

std::size_t samp_client_sdk_fixture_r3_1_local_player_incar_offset() {
    return offsetof(FixtureR3_1LocalPlayerPrefix, incar);
}

std::size_t samp_client_sdk_fixture_r3_1_local_player_onfoot_offset() {
    return offsetof(FixtureR3_1LocalPlayerPrefix, onfoot);
}

std::size_t samp_client_sdk_fixture_r3_1_local_player_active_offset() {
    return offsetof(FixtureR3_1LocalPlayerPrefix, active);
}

std::size_t samp_client_sdk_fixture_r3_1_local_player_current_vehicle_offset() {
    return offsetof(FixtureR3_1LocalPlayerPrefix, current_vehicle);
}

std::size_t samp_client_sdk_fixture_r3_1_local_player_last_any_update_offset() {
    return offsetof(FixtureR3_1LocalPlayerPrefix, last_any_update);
}

std::size_t samp_client_sdk_fixture_r3_1_ped_game_ped_offset() {
    return offsetof(FixtureR3_1PedPrefix, game_ped);
}

std::size_t samp_client_sdk_fixture_r3_1_input_size() {
    return sizeof(FixtureR3_1Input);
}

std::size_t samp_client_sdk_fixture_r3_1_input_editbox_offset() {
    return offsetof(FixtureR3_1Input, editbox);
}

std::size_t samp_client_sdk_fixture_r3_1_input_command_count_offset() {
    return offsetof(FixtureR3_1Input, command_count);
}

std::size_t samp_client_sdk_fixture_r3_1_input_command_names_offset() {
    return offsetof(FixtureR3_1Input, command_names);
}

std::size_t samp_client_sdk_fixture_r3_1_input_command_name_capacity() {
    return sizeof(FixtureR3_1Input::command_names[0]);
}

std::size_t samp_client_sdk_fixture_r3_1_input_enabled_offset() {
    return offsetof(FixtureR3_1Input, enabled);
}

std::size_t samp_client_sdk_fixture_r3_1_dialog_size() {
    return sizeof(FixtureR3_1Dialog);
}

std::size_t samp_client_sdk_fixture_r3_1_dialog_active_offset() {
    return offsetof(FixtureR3_1Dialog, active);
}

std::size_t samp_client_sdk_fixture_r3_1_dialog_caption_offset() {
    return offsetof(FixtureR3_1Dialog, caption);
}

std::size_t samp_client_sdk_fixture_r3_1_dialog_listbox_offset() {
    return offsetof(FixtureR3_1Dialog, listbox);
}

std::size_t samp_client_sdk_fixture_r3_1_dialog_editbox_offset() {
    return offsetof(FixtureR3_1Dialog, editbox);
}

std::size_t samp_client_sdk_fixture_r3_1_dialog_type_offset() {
    return offsetof(FixtureR3_1Dialog, type);
}

std::size_t samp_client_sdk_fixture_r3_1_dialog_id_offset() {
    return offsetof(FixtureR3_1Dialog, id);
}

std::size_t samp_client_sdk_fixture_r3_1_dialog_text_offset() {
    return offsetof(FixtureR3_1Dialog, text);
}

std::size_t samp_client_sdk_fixture_r3_1_dialog_server_side_offset() {
    return offsetof(FixtureR3_1Dialog, server_side);
}

std::size_t samp_client_sdk_fixture_r3_1_listbox_selected_offset() {
    return offsetof(FixtureR3_1ListboxPrefix, selected);
}

std::size_t samp_client_sdk_fixture_r3_1_listbox_items_offset() {
    return offsetof(FixtureR3_1ListboxPrefix, items);
}

std::size_t samp_client_sdk_fixture_r3_1_listbox_item_count_offset() {
    return offsetof(FixtureR3_1ListboxPrefix, item_count);
}

std::size_t samp_client_sdk_fixture_r3_1_scoreboard_size() {
    return sizeof(FixtureR3_1Scoreboard);
}

std::size_t samp_client_sdk_fixture_r3_1_scoreboard_enabled_offset() {
    return offsetof(FixtureR3_1Scoreboard, is_enabled);
}

std::size_t samp_client_sdk_fixture_r3_1_game_size() {
    return sizeof(FixtureR3_1Game);
}

std::size_t samp_client_sdk_fixture_r3_1_game_cursor_mode_offset() {
    return offsetof(FixtureR3_1Game, cursor_mode);
}

std::size_t samp_client_sdk_fixture_r5_1_netgame_size() {
    return sizeof(FixtureR5_1NetGame);
}

std::size_t samp_client_sdk_fixture_r5_1_netgame_rak_client_offset() {
    return offsetof(FixtureR5_1NetGame, rak_client);
}

std::size_t samp_client_sdk_fixture_r5_1_netgame_game_state_offset() {
    return offsetof(FixtureR5_1NetGame, game_state);
}

std::size_t samp_client_sdk_fixture_r5_1_netgame_pools_offset() {
    return offsetof(FixtureR5_1NetGame, pools);
}

std::size_t samp_client_sdk_fixture_r5_1_input_size() {
    return sizeof(FixtureR5_1Input);
}

std::size_t samp_client_sdk_fixture_r5_1_input_command_count_offset() {
    return offsetof(FixtureR5_1Input, command_count);
}

std::size_t samp_client_sdk_fixture_r5_1_input_enabled_offset() {
    return offsetof(FixtureR5_1Input, enabled);
}

std::size_t samp_client_sdk_fixture_r5_1_dialog_size() {
    return sizeof(FixtureR5_1Dialog);
}

std::size_t samp_client_sdk_fixture_r5_1_dialog_active_offset() {
    return offsetof(FixtureR5_1Dialog, active);
}

std::size_t samp_client_sdk_fixture_r5_1_dialog_caption_offset() {
    return offsetof(FixtureR5_1Dialog, caption);
}

#define SAMP_CLIENT_SDK_SIZE_FIXTURE(name, type) \
    std::size_t name() { return sizeof(type); }
#define SAMP_CLIENT_SDK_OFFSET_FIXTURE(name, type, field) \
    std::size_t name() { return offsetof(type, field); }

SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_r5_1_pools_size, FixtureR5_1Pools)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_pools_pickup_offset, FixtureR5_1Pools, pickup)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_pools_object_offset, FixtureR5_1Pools, object)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_pools_gangzone_offset, FixtureR5_1Pools, gangzone)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_pools_label_offset, FixtureR5_1Pools, label)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_pools_textdraw_offset, FixtureR5_1Pools, textdraw)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_r5_1_player_info_size, FixtureR5_1PlayerInfo)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_player_info_is_npc_offset, FixtureR5_1PlayerInfo, is_npc)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_r5_1_player_pool_size, FixtureR5_1PlayerPool)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_player_pool_local_id_offset, FixtureR5_1PlayerPool, local_id)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_player_pool_objects_offset, FixtureR5_1PlayerPool, objects)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_player_pool_largest_id_offset, FixtureR5_1PlayerPool, largest_id)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_r5_1_remote_player_size, FixtureR5_1RemotePlayer)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_remote_player_special_action_offset, FixtureR5_1RemotePlayer, special_action)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_remote_player_animation_offset, FixtureR5_1RemotePlayer, animation)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_remote_player_ped_offset, FixtureR5_1RemotePlayer, ped)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_r5_1_local_player_size, FixtureR5_1LocalPlayer)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_local_player_incar_offset, FixtureR5_1LocalPlayer, incar)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_local_player_aim_offset, FixtureR5_1LocalPlayer, aim)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_local_player_trailer_offset, FixtureR5_1LocalPlayer, trailer)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_local_player_onfoot_offset, FixtureR5_1LocalPlayer, onfoot)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_local_player_passenger_offset, FixtureR5_1LocalPlayer, passenger)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_local_player_active_offset, FixtureR5_1LocalPlayer, active)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_local_player_current_vehicle_offset, FixtureR5_1LocalPlayer, current_vehicle)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_local_player_ped_offset, FixtureR5_1LocalPlayer, ped)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_r5_1_local_player_last_any_update_offset, FixtureR5_1LocalPlayer, last_any_update)

SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_pools_size, FixtureDlPools)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_pools_pickup_offset, FixtureDlPools, pickup)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_pools_object_offset, FixtureDlPools, object)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_pools_gangzone_offset, FixtureDlPools, gangzone)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_pools_label_offset, FixtureDlPools, label)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_pools_textdraw_offset, FixtureDlPools, textdraw)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_samp_string_size, FixtureDlSampString)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_player_info_size, FixtureDlPlayerInfo)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_player_info_is_npc_offset, FixtureDlPlayerInfo, is_npc)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_player_pool_size, FixtureDlPlayerPool)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_player_pool_local_id_offset, FixtureDlPlayerPool, local_id)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_player_pool_local_player_offset, FixtureDlPlayerPool, local_player)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_player_pool_largest_id_offset, FixtureDlPlayerPool, largest_id)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_player_pool_players_offset, FixtureDlPlayerPool, players)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_player_pool_not_empty_offset, FixtureDlPlayerPool, not_empty)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_player_pool_collision_offset, FixtureDlPlayerPool, previous_collision)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_player_pool_ping_offset, FixtureDlPlayerPool, local_ping)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_player_pool_score_offset, FixtureDlPlayerPool, local_score)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_local_player_size, FixtureDlLocalPlayer)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_local_player_ped_offset, FixtureDlLocalPlayer, ped)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_local_player_trailer_offset, FixtureDlLocalPlayer, trailer)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_local_player_onfoot_offset, FixtureDlLocalPlayer, onfoot)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_local_player_passenger_offset, FixtureDlLocalPlayer, passenger)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_local_player_incar_offset, FixtureDlLocalPlayer, incar)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_local_player_aim_offset, FixtureDlLocalPlayer, aim)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_local_player_active_offset, FixtureDlLocalPlayer, active)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_local_player_current_vehicle_offset, FixtureDlLocalPlayer, current_vehicle)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_local_player_last_any_update_offset, FixtureDlLocalPlayer, last_any_update)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_remote_player_size, FixtureDlRemotePlayer)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_remote_player_ped_offset, FixtureDlRemotePlayer, ped)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_remote_player_special_action_offset, FixtureDlRemotePlayer, special_action)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_remote_player_passenger_offset, FixtureDlRemotePlayer, passenger)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_remote_player_onfoot_offset, FixtureDlRemotePlayer, onfoot)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_remote_player_incar_offset, FixtureDlRemotePlayer, incar)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_remote_player_trailer_offset, FixtureDlRemotePlayer, trailer)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_remote_player_aim_offset, FixtureDlRemotePlayer, aim)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_remote_player_armour_offset, FixtureDlRemotePlayer, reported_armour)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_remote_player_health_offset, FixtureDlRemotePlayer, reported_health)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_remote_player_animation_offset, FixtureDlRemotePlayer, animation)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_vehicle_pool_size, FixtureDlVehiclePool)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_vehicle_pool_not_empty_offset, FixtureDlVehiclePool, not_empty)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_vehicle_pool_game_objects_offset, FixtureDlVehiclePool, game_objects)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_object_pool_size, FixtureDlObjectPool)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_object_pool_not_empty_offset, FixtureDlObjectPool, not_empty)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_object_pool_objects_offset, FixtureDlObjectPool, objects)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_pickup_pool_size, FixtureDlPickupPool)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_pickup_pool_handles_offset, FixtureDlPickupPool, handles)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_entity_size, FixtureDlEntity)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_entity_handle_offset, FixtureDlEntity, handle)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_ped_size, FixtureDlPed)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_ped_game_ped_offset, FixtureDlPed, game_ped)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_gangzone_pool_size, FixtureDlGangzonePool)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_gangzone_pool_not_empty_offset, FixtureDlGangzonePool, not_empty)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_label_pool_size, FixtureDlTextLabelPool)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_label_pool_not_empty_offset, FixtureDlTextLabelPool, not_empty)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_textdraw_size, FixtureDlTextdraw)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_textdraw_data_offset, FixtureDlTextdraw, data)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_textdraw_pool_size, FixtureDlTextdrawPool)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_textdraw_pool_objects_offset, FixtureDlTextdrawPool, objects)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_chat_size, FixtureDlChat)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_chat_mode_offset, FixtureDlChat, mode)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_chat_entries_offset, FixtureDlChat, entries)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_scoreboard_size, FixtureDlScoreboard)
SAMP_CLIENT_SDK_OFFSET_FIXTURE(samp_client_sdk_fixture_dl_game_cursor_mode_offset, FixtureDlGamePrefix, cursor_mode)
SAMP_CLIENT_SDK_SIZE_FIXTURE(samp_client_sdk_fixture_dl_animation_entry_size, FixtureDlAnimationEntry)

#undef SAMP_CLIENT_SDK_OFFSET_FIXTURE
#undef SAMP_CLIENT_SDK_SIZE_FIXTURE

std::size_t samp_client_sdk_fixture_dl_netgame_size() {
    return sizeof(FixtureDlNetGame);
}

std::size_t samp_client_sdk_fixture_dl_netgame_rak_client_offset() {
    return offsetof(FixtureDlNetGame, rak_client);
}

std::size_t samp_client_sdk_fixture_dl_netgame_game_state_offset() {
    return offsetof(FixtureDlNetGame, game_state);
}

std::size_t samp_client_sdk_fixture_dl_netgame_pools_offset() {
    return offsetof(FixtureDlNetGame, pools);
}

std::size_t samp_client_sdk_fixture_dl_input_size() {
    return sizeof(FixtureDlInput);
}

std::size_t samp_client_sdk_fixture_dl_input_command_count_offset() {
    return offsetof(FixtureDlInput, command_count);
}

std::size_t samp_client_sdk_fixture_dl_input_enabled_offset() {
    return offsetof(FixtureDlInput, enabled);
}

std::size_t samp_client_sdk_fixture_dl_dialog_size() {
    return sizeof(FixtureDlDialog);
}

std::size_t samp_client_sdk_fixture_dl_dialog_active_offset() {
    return offsetof(FixtureDlDialog, active);
}

std::size_t samp_client_sdk_fixture_dl_dialog_caption_offset() {
    return offsetof(FixtureDlDialog, caption);
}

}
