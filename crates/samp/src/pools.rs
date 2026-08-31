use crate::{PlayerId, VehicleId};
use modkit_abi::{MOD_NATIVE_CALL_FAILED, ModResult};
use modkit_sdk::SampPoolService;
use std::num::NonZeroI32;

#[derive(Clone, Copy)]
pub struct Pools {
    service: SampPoolService,
}

#[derive(Clone, Copy)]
pub struct Objects {
    service: SampPoolService,
}

#[derive(Clone, Copy)]
pub struct Pickups {
    service: SampPoolService,
}

#[derive(Clone, Copy)]
pub struct Vehicles {
    service: SampPoolService,
}

#[derive(Clone, Copy)]
pub struct Gangzones {
    service: SampPoolService,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObjectId(u16);

impl ObjectId {
    pub const fn new(raw: u16) -> Option<Self> {
        if raw < modkit_abi::SAMP_MAX_OBJECTS {
            Some(Self(raw))
        } else {
            None
        }
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PickupId(u16);

impl PickupId {
    pub const fn new(raw: u16) -> Option<Self> {
        if raw < modkit_abi::SAMP_MAX_PICKUPS {
            Some(Self(raw))
        } else {
            None
        }
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GangzoneId(u16);

impl GangzoneId {
    pub const fn new(raw: u16) -> Option<Self> {
        if raw < modkit_abi::SAMP_MAX_GANGZONES {
            Some(Self(raw))
        } else {
            None
        }
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ObjectHandle(NonZeroI32);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PickupHandle(NonZeroI32);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VehicleHandle(NonZeroI32);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PedHandle(NonZeroI32);

macro_rules! handle {
    ($type:ty) => {
        impl $type {
            pub const fn new(raw: i32) -> Option<Self> {
                match NonZeroI32::new(raw) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            pub const fn get(self) -> i32 {
                self.0.get()
            }
        }
    };
}

handle!(ObjectHandle);
handle!(PickupHandle);
handle!(VehicleHandle);
handle!(PedHandle);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gangzone {
    pub id: GangzoneId,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub colour: u32,
    pub alt_colour: u32,
}

impl Pools {
    pub(crate) const fn new(service: SampPoolService) -> Self {
        Self { service }
    }

    pub const fn objects(self) -> Objects {
        Objects {
            service: self.service,
        }
    }

    pub const fn pickups(self) -> Pickups {
        Pickups {
            service: self.service,
        }
    }

    pub const fn vehicles(self) -> Vehicles {
        Vehicles {
            service: self.service,
        }
    }

    pub const fn gangzones(self) -> Gangzones {
        Gangzones {
            service: self.service,
        }
    }
}

impl Objects {
    pub fn exists(self, id: ObjectId) -> Result<bool, ModResult> {
        self.service.object_exists(id.get())
    }

    pub fn handle(self, id: ObjectId) -> Result<Option<ObjectHandle>, ModResult> {
        Ok(self
            .service
            .object_handle(id.get())?
            .and_then(ObjectHandle::new))
    }

    pub fn id_by_handle(self, handle: ObjectHandle) -> Result<Option<ObjectId>, ModResult> {
        checked_optional_id(
            self.service.object_id_by_handle(handle.get())?,
            ObjectId::new,
        )
    }
}

impl Pickups {
    pub fn handle(self, id: PickupId) -> Result<Option<PickupHandle>, ModResult> {
        Ok(self
            .service
            .pickup_handle(id.get())?
            .and_then(PickupHandle::new))
    }

    pub fn id_by_handle(self, handle: PickupHandle) -> Result<Option<PickupId>, ModResult> {
        checked_optional_id(
            self.service.pickup_id_by_handle(handle.get())?,
            PickupId::new,
        )
    }
}

impl Vehicles {
    pub fn exists(self, id: VehicleId) -> Result<bool, ModResult> {
        self.service.vehicle_exists(id.get())
    }

    pub fn handle(self, id: VehicleId) -> Result<Option<VehicleHandle>, ModResult> {
        Ok(self
            .service
            .vehicle_handle(id.get())?
            .and_then(VehicleHandle::new))
    }

    pub fn id_by_handle(self, handle: VehicleHandle) -> Result<Option<VehicleId>, ModResult> {
        checked_optional_id(
            self.service.vehicle_id_by_handle(handle.get())?,
            VehicleId::new,
        )
    }
}

impl Gangzones {
    pub fn get(self, id: GangzoneId) -> Result<Option<Gangzone>, ModResult> {
        let raw = self.service.gangzone(id.get())?;
        if raw.exists == 0 {
            return Ok(None);
        }
        Ok(Some(Gangzone {
            id: GangzoneId::new(raw.id).ok_or(MOD_NATIVE_CALL_FAILED)?,
            left: raw.left,
            top: raw.top,
            right: raw.right,
            bottom: raw.bottom,
            colour: raw.colour,
            alt_colour: raw.alt_colour,
        }))
    }
}

pub(crate) fn player_ped_handle(
    service: SampPoolService,
    id: PlayerId,
) -> Result<Option<PedHandle>, ModResult> {
    Ok(service
        .player_ped_handle(id.get())?
        .and_then(PedHandle::new))
}

pub(crate) fn player_id_by_ped_handle(
    service: SampPoolService,
    handle: PedHandle,
) -> Result<Option<PlayerId>, ModResult> {
    checked_optional_id(
        service.player_id_by_ped_handle(handle.get())?,
        PlayerId::new,
    )
}

fn checked_optional_id<T>(
    raw: Option<u16>,
    convert: impl FnOnce(u16) -> Option<T>,
) -> Result<Option<T>, ModResult> {
    match raw {
        None => Ok(None),
        Some(raw) => convert(raw).map(Some).ok_or(MOD_NATIVE_CALL_FAILED),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_reject_the_absent_zero_sentinel() {
        assert_eq!(ObjectHandle::new(0), None);
        assert_eq!(ObjectHandle::new(7).map(ObjectHandle::get), Some(7));
    }

    #[test]
    fn ids_reject_their_upper_bounds() {
        assert_eq!(ObjectId::new(modkit_abi::SAMP_MAX_OBJECTS), None);
        assert_eq!(PickupId::new(modkit_abi::SAMP_MAX_PICKUPS), None);
        assert_eq!(GangzoneId::new(modkit_abi::SAMP_MAX_GANGZONES), None);
    }
}
