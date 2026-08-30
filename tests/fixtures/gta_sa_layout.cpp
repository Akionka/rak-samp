// Independent GTA SA layout oracle.
//
// Compile only against the pinned plugin-sdk revision documented in
// docs/evidence/gta-sa-layout-oracle.md. The Rust build enables this fixture
// only when PLUGIN_SDK_DIR points at that checkout.
#define GTASA
#define RW

#include <cstddef>
#include "CCamera.h"
#include "CEntity.h"
#include "CObject.h"
#include "CMatrix.h"
#include "CPools.h"
#include "CPed.h"
#include "CPlaceable.h"
#include "CVehicle.h"
#include "CVector.h"
#include "CVector2D.h"

static_assert(sizeof(CVector2D) == 0x08);
static_assert(sizeof(CVector) == 0x0C);
static_assert(sizeof(CMatrix) == 0x48);
static_assert(offsetof(CMatrix, right) == 0x00);
static_assert(offsetof(CMatrix, up) == 0x10);
static_assert(offsetof(CMatrix, at) == 0x20);
static_assert(offsetof(CMatrix, pos) == 0x30);
static_assert(offsetof(CMatrix, m_pAttachMatrix) == 0x40);
static_assert(offsetof(CMatrix, m_bOwnsAttachedMatrix) == 0x44);

static_assert(sizeof(CPlaceable) == 0x18);
static_assert(offsetof(CPlaceable, m_placement) == 0x04);
static_assert(offsetof(CPlaceable, m_matrix) == 0x14);
static_assert(sizeof(CEntity) == 0x38);
static_assert(sizeof(CPed) == 0x79C);
static_assert(offsetof(CPed, m_fHealth) == 0x540);
static_assert(offsetof(CPed, m_fArmour) == 0x548);
static_assert(sizeof(CVehicle) == 0x5A0);
static_assert(offsetof(CVehicle, m_fHealth) == 0x4C0);
static_assert(sizeof(CObject) == 0x17C);
static_assert(sizeof(CCamera) == 0xD78);
static_assert(offsetof(CCamera, m_vecGameCamPos) == 0x908);
static_assert(offsetof(CCamera, m_mCameraMatrix) == 0x974);

using PedPool = CPool<CPed, CCopPed>;
static_assert(sizeof(PedPool) == 0x14);
static_assert(offsetof(PedPool, m_pObjects) == 0x00);
static_assert(offsetof(PedPool, m_byteMap) == 0x04);
static_assert(offsetof(PedPool, m_nSize) == 0x08);

extern "C" std::size_t gta_sa_fixture_vector2_size() { return sizeof(CVector2D); }
extern "C" std::size_t gta_sa_fixture_vector3_size() { return sizeof(CVector); }
extern "C" std::size_t gta_sa_fixture_matrix_size() { return sizeof(CMatrix); }
extern "C" std::size_t gta_sa_fixture_matrix_right_offset() { return offsetof(CMatrix, right); }
extern "C" std::size_t gta_sa_fixture_matrix_forward_offset() { return offsetof(CMatrix, up); }
extern "C" std::size_t gta_sa_fixture_matrix_up_offset() { return offsetof(CMatrix, at); }
extern "C" std::size_t gta_sa_fixture_matrix_position_offset() { return offsetof(CMatrix, pos); }
extern "C" std::size_t gta_sa_fixture_matrix_attached_offset() { return offsetof(CMatrix, m_pAttachMatrix); }
extern "C" std::size_t gta_sa_fixture_matrix_owns_attached_offset() { return offsetof(CMatrix, m_bOwnsAttachedMatrix); }
extern "C" std::size_t gta_sa_fixture_placeable_size() { return sizeof(CPlaceable); }
extern "C" std::size_t gta_sa_fixture_placeable_position_offset() { return offsetof(CPlaceable, m_placement); }
extern "C" std::size_t gta_sa_fixture_placeable_matrix_offset() { return offsetof(CPlaceable, m_matrix); }
extern "C" std::size_t gta_sa_fixture_entity_size() { return sizeof(CEntity); }
extern "C" std::size_t gta_sa_fixture_ped_size() { return sizeof(CPed); }
extern "C" std::size_t gta_sa_fixture_ped_health_offset() { return offsetof(CPed, m_fHealth); }
extern "C" std::size_t gta_sa_fixture_ped_armour_offset() { return offsetof(CPed, m_fArmour); }
extern "C" std::size_t gta_sa_fixture_vehicle_size() { return sizeof(CVehicle); }
extern "C" std::size_t gta_sa_fixture_vehicle_health_offset() { return offsetof(CVehicle, m_fHealth); }
extern "C" std::size_t gta_sa_fixture_object_size() { return sizeof(CObject); }
extern "C" std::size_t gta_sa_fixture_camera_size() { return sizeof(CCamera); }
extern "C" std::size_t gta_sa_fixture_camera_game_position_offset() { return offsetof(CCamera, m_vecGameCamPos); }
extern "C" std::size_t gta_sa_fixture_camera_matrix_offset() { return offsetof(CCamera, m_mCameraMatrix); }

extern "C" std::size_t gta_sa_fixture_pool_size() { return sizeof(PedPool); }
extern "C" std::size_t gta_sa_fixture_pool_objects_offset() { return offsetof(PedPool, m_pObjects); }
extern "C" std::size_t gta_sa_fixture_pool_flags_offset() { return offsetof(PedPool, m_byteMap); }
extern "C" std::size_t gta_sa_fixture_pool_capacity_offset() { return offsetof(PedPool, m_nSize); }

extern "C" void gta_sa_fixture_invoke_teleport(
    void* target,
    void* object,
    float x,
    float y,
    float z,
    unsigned char reset_rotation
) {
    using TeleportFn = void(__thiscall*)(void*, CVector, bool);
    reinterpret_cast<TeleportFn>(target)(
        object,
        CVector(x, y, z),
        reset_rotation != 0
    );
}
