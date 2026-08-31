//! Adapters from the exact-version pool service to the frozen legacy host implementation.

use modkit_abi::{MOD_INVALID_ARGUMENT, ModResult, SampGangzoneV1};
use sdk_abi::{SampClientSdkGangzoneV1, SampClientSdkResult};

use super::modkit::subscription_result;

macro_rules! direct_read {
    ($name:ident, $out:ty, $target:path) => {
        pub(super) unsafe extern "system" fn $name(id: u16, out: *mut $out) -> ModResult {
            subscription_result(unsafe { $target(id, out) })
        }
    };
}

direct_read!(object_exists, u8, super::pools::object_exists);
direct_read!(vehicle_exists, u8, super::pools::vehicle_exists);
direct_read!(object_handle, i32, super::handles::local_object_handle);
direct_read!(pickup_handle, i32, super::handles::local_pickup_handle);
direct_read!(vehicle_handle, i32, super::handles::local_vehicle_handle);
direct_read!(
    player_ped_handle,
    i32,
    super::handles::local_player_ped_handle
);

macro_rules! reverse_read {
    ($name:ident, $target:path) => {
        pub(super) unsafe extern "system" fn $name(handle: i32, out: *mut u16) -> ModResult {
            subscription_result(unsafe { $target(handle, out) })
        }
    };
}

reverse_read!(
    object_id_by_handle,
    super::handles::local_object_id_by_handle
);
reverse_read!(
    pickup_id_by_handle,
    super::handles::local_pickup_id_by_handle
);
reverse_read!(
    vehicle_id_by_handle,
    super::handles::local_vehicle_id_by_handle
);
reverse_read!(
    player_id_by_ped_handle,
    super::handles::local_player_id_by_ped_handle
);

pub(super) unsafe extern "system" fn gangzone(id: u16, out: *mut SampGangzoneV1) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    let mut legacy = SampClientSdkGangzoneV1::default();
    let result = unsafe { super::snapshots::gangzone_info(id, &mut legacy) };
    if result == SampClientSdkResult::Ok {
        *out = unsafe { core::mem::transmute::<SampClientSdkGangzoneV1, SampGangzoneV1>(legacy) };
    }
    subscription_result(result)
}
