//! R1 pool reads and mutations.

use super::super::argb_to_native_rgba;
use super::*;

impl R1ClientProfile {
    /// Reads one R1 vehicle-pool existence flag on the game-thread pump.
    /// Only the copied boolean crosses the private profile boundary.
    pub(in super::super) fn vehicle_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_VEHICLES {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_vehicle_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_VEHICLE_POOL_RVA) };
        let pool = unsafe { get_vehicle_pool(net_game) };
        let checked_len =
            VEHICLE_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if pool.is_null() || !readable_range(pool.cast(), checked_len) {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + VEHICLE_POOL_DOES_EXIST_RVA) };
        match unsafe { does_exist(pool, id) } {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Reads one R1 3D text-label-pool existence flag on the game-thread pump.
    /// Only the copied boolean crosses the private profile boundary.
    pub(in super::super) fn text_label_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_LABEL_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_LABEL_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            LABEL_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        read_r1_bool(pool + LABEL_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>())
    }

    /// Invokes the documented R1 label-pool delete method on the game thread.
    pub(in super::super) fn delete_text_label(self, id: u16) -> Result<(), DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|value| *value != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_LABEL_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_LABEL_POOL_OFFSET) }
            .filter(|value| *value != 0)
            .ok_or(DirectClientError::NotReady)? as *mut c_void;
        if !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let delete: LabelPoolDeleteFn =
            unsafe { mem::transmute(self.module_base + LABEL_POOL_DELETE_RVA) };
        if unsafe { delete(pool, id) } == 0 {
            return Err(DirectClientError::NotReady);
        }
        Ok(())
    }

    /// Finds the lowest free R1 3D text-label slot on the game thread.
    pub(in super::super) fn first_free_text_label_id(self) -> Result<u16, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|value| *value != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_LABEL_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_LABEL_POOL_OFFSET) }
            .filter(|value| *value != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flag_end =
            LABEL_POOL_NOT_EMPTY_OFFSET + usize::from(MAX_SAMP_TEXT_LABELS) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, flag_end) {
            return Err(DirectClientError::NotReady);
        }
        for id in 0..usize::from(MAX_SAMP_TEXT_LABELS) {
            if !read_r1_bool(pool + LABEL_POOL_NOT_EMPTY_OFFSET + id * mem::size_of::<i32>())? {
                return Ok(id as u16);
            }
        }
        Err(DirectClientError::NotReady)
    }

    /// Invokes the documented R1 label-pool create method on the game thread.
    #[allow(clippy::too_many_arguments)]
    pub(in super::super) fn create_text_label(
        self,
        id: u16,
        text: &[u8],
        colour: u32,
        position: Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<(), DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS
            || text.len() > MAX_TEXT_LABEL_TEXT_BYTES
            || text.contains(&0)
            || !position.x.is_finite()
            || !position.y.is_finite()
            || !position.z.is_finite()
            || !draw_distance.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|value| *value != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_LABEL_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_LABEL_POOL_OFFSET) }
            .filter(|value| *value != 0)
            .ok_or(DirectClientError::NotReady)? as *mut c_void;
        if !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let text = nul_terminated(text.to_vec());
        let create: LabelPoolCreateFn =
            unsafe { mem::transmute(self.module_base + LABEL_POOL_CREATE_RVA) };
        unsafe {
            create(
                pool,
                id,
                text.as_ptr(),
                argb_to_native_rgba(colour),
                NativeVector3::from(position),
                draw_distance,
                u8::from(behind_walls),
                attached_player_id,
                attached_vehicle_id,
            );
        }
        Ok(())
    }

    /// Copies one R1 3D text-label record on the game-thread pump. The native
    /// string allocation is read only after its matching pool flag is true,
    /// bounded by the R1 encoded-string limit, and copied before this method
    /// returns. No native pointer crosses the private profile boundary.
    pub(in super::super) fn text_label(
        self,
        id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_LABEL_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_LABEL_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            LABEL_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + LABEL_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>(),
        )? {
            return Ok(None);
        }
        let label = pool + usize::from(id) * LABEL_SIZE;
        if !readable_range(label as *const u8, LABEL_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let text = unsafe { read_unaligned::<usize>(label + LABEL_TEXT_OFFSET) }
            .filter(|text| *text != 0)
            .ok_or(DirectClientError::NotReady)?;
        let text = unsafe { bounded_c_string(text as *const u8, MAX_TEXT_LABEL_TEXT_BYTES + 1) }
            .ok_or(DirectClientError::NotReady)?;
        let colour = unsafe { read_unaligned::<u32>(label + LABEL_COLOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let position = unsafe { read_vector3(label + LABEL_POSITION_OFFSET) }
            .filter(|position| {
                position.x.is_finite() && position.y.is_finite() && position.z.is_finite()
            })
            .ok_or(DirectClientError::NotReady)?;
        let draw_distance = unsafe { read_unaligned::<f32>(label + LABEL_DRAW_DISTANCE_OFFSET) }
            .filter(|draw_distance| draw_distance.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let behind_walls = match unsafe { read_unaligned::<u8>(label + LABEL_BEHIND_WALLS_OFFSET) }
        {
            Some(0) => false,
            Some(1) => true,
            _ => return Err(DirectClientError::NotReady),
        };
        let attached_player =
            unsafe { read_unaligned::<u16>(label + LABEL_ATTACHED_PLAYER_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let attached_vehicle =
            unsafe { read_unaligned::<u16>(label + LABEL_ATTACHED_VEHICLE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        Ok(Some(TextLabelSnapshot {
            id,
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id: (attached_player != u16::MAX).then_some(attached_player),
            attached_vehicle_id: (attached_vehicle != u16::MAX).then_some(attached_vehicle),
        }))
    }

    /// Reads one R1 object-pool existence flag on the game-thread pump.
    /// Only the copied boolean crosses the private profile boundary.
    pub(in super::super) fn object_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_OBJECTS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_OBJECT_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_OBJECT_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            OBJECT_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        read_r1_bool(pool + OBJECT_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>())
    }

    /// Copies one R1 object-pool handle (GTAREF) on the game thread. The
    /// handle is the `SCEntity::m_handle` field of the object's SAMP wrapper.
    /// Copies one R1 gangzone record on the game-thread pump. No client or
    /// GTA pointer crosses the private profile boundary.
    pub(in super::super) fn gangzone(
        self,
        id: u16,
    ) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_GANGZONES {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_GANGZONE_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_GANGZONE_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            GANGZONE_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + GANGZONE_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>(),
        )? {
            return Ok(None);
        }
        let gangzone =
            unsafe { read_unaligned::<usize>(pool + usize::from(id) * mem::size_of::<usize>()) }
                .filter(|gangzone| *gangzone != 0)
                .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            gangzone as *const u8,
            GANGZONE_ALTERNATE_COLOUR_OFFSET + mem::size_of::<u32>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let left = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_LEFT_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let bottom = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_BOTTOM_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let right = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_RIGHT_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let top = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_TOP_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let colour = unsafe { read_unaligned::<u32>(gangzone + GANGZONE_COLOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let alternate_colour =
            unsafe { read_unaligned::<u32>(gangzone + GANGZONE_ALTERNATE_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        Ok(Some(GangzoneSnapshot {
            id,
            left,
            bottom,
            right,
            top,
            colour,
            alternate_colour,
        }))
    }
}
