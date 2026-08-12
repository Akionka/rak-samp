use super::*;

impl R1ClientProfile {
    /// Reads one R1 textdraw-pool existence flag on the game-thread pump.
    /// The raw pool index covers the 2,048 global and 256 local slots. Only
    /// the copied boolean crosses the private profile boundary.
    pub(in super::super) fn textdraw_exists(
        self,
        pool_index: u16,
    ) -> Result<bool, DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )
    }

    /// Invokes the documented R1 textdraw-pool deletion method on the game
    /// thread after resolving the private pool pointer.
    pub(in super::super) fn delete_textdraw(
        self,
        pool_index: u16,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)? as *mut c_void;
        if !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let delete: TextdrawPoolDeleteFn =
            unsafe { mem::transmute(self.module_base + TEXTDRAW_POOL_DELETE_RVA) };
        unsafe { delete(pool, pool_index) };
        Ok(())
    }

    /// Creates one R1 textdraw in a caller-selected free pool slot. The native
    /// pool constructor owns allocation and copies the transient stack data.
    pub(in super::super) fn create_textdraw(
        self,
        pool_index: u16,
        text: &[u8],
        x: f32,
        y: f32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS
            || text.len() > MAX_TEXTDRAW_CREATE_TEXT_BYTES
            || text.contains(&0)
            || !x.is_finite()
            || !y.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        let flag =
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>();
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object =
            unsafe { read_unaligned::<usize>(object_slot) }.ok_or(DirectClientError::NotReady)?;
        if read_r1_bool(flag)? || object != 0 {
            return Err(DirectClientError::NotReady);
        }

        let mut transmit = [0_u8; TEXTDRAW_TRANSMIT_SIZE];
        transmit[TEXTDRAW_TRANSMIT_X_OFFSET..TEXTDRAW_TRANSMIT_X_OFFSET + mem::size_of::<f32>()]
            .copy_from_slice(&x.to_le_bytes());
        transmit[TEXTDRAW_TRANSMIT_Y_OFFSET..TEXTDRAW_TRANSMIT_Y_OFFSET + mem::size_of::<f32>()]
            .copy_from_slice(&y.to_le_bytes());
        let mut native_text = Vec::with_capacity(text.len() + 1);
        native_text.extend_from_slice(text);
        native_text.push(0);
        let create: TextdrawPoolCreateFn =
            unsafe { mem::transmute(self.module_base + TEXTDRAW_POOL_CREATE_RVA) };
        if unsafe {
            create(
                (pool as *mut u8).cast(),
                i32::from(pool_index),
                transmit.as_mut_ptr().cast(),
                native_text.as_ptr(),
            )
        }
        .is_null()
        {
            return Err(DirectClientError::NotReady);
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's finite screen position on the game
    /// thread. The fixture-backed pool, object, and two scalar fields are
    /// validated before the direct write.
    pub(in super::super) fn set_textdraw_position(
        self,
        pool_index: u16,
        x: f32,
        y: f32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS || !x.is_finite() || !y.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_X_OFFSET) as *mut f32;
        if !writable_range(field.cast(), 2 * mem::size_of::<f32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field, x);
            ptr::write_unaligned(field.add(1), y);
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's supported font/style selector on
    /// the game thread.
    pub(in super::super) fn set_textdraw_style(
        self,
        pool_index: u16,
        style: i32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS || !(0..=5).contains(&style) {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end))
            || !read_r1_bool(
                pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET
                    + usize::from(pool_index) * mem::size_of::<i32>(),
            )?
        {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_STYLE_OFFSET) as *mut i32;
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, style) };
        Ok(())
    }

    /// Updates one existing R1 textdraw's finite letter dimensions and colour.
    pub(in super::super) fn set_textdraw_letter_style(
        self,
        pool_index: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS || !width.is_finite() || !height.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_LETTER_WIDTH_OFFSET) as *mut u8;
        if !writable_range(field, mem::size_of::<f32>() * 2 + mem::size_of::<u32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field.cast::<f32>(), width);
            ptr::write_unaligned(field.add(mem::size_of::<f32>()).cast::<f32>(), height);
            ptr::write_unaligned(field.add(mem::size_of::<f32>() * 2).cast::<u32>(), colour);
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's proportional flag on the game thread.
    pub(in super::super) fn set_textdraw_proportional(
        self,
        pool_index: u16,
        proportional: bool,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_PROPORTIONAL_OFFSET) as *mut u8;
        if !writable_range(field, mem::size_of::<u8>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, u8::from(proportional)) };
        Ok(())
    }

    /// Updates one existing R1 textdraw's shadow and background colour.
    pub(in super::super) fn set_textdraw_shadow(
        self,
        pool_index: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_BACKGROUND_COLOUR_OFFSET) as *mut u8;
        let len = TEXTDRAW_SHADOW_OFFSET + mem::size_of::<u8>() - TEXTDRAW_BACKGROUND_COLOUR_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field.cast::<u32>(), colour);
            ptr::write_unaligned(
                field.add(TEXTDRAW_SHADOW_OFFSET - TEXTDRAW_BACKGROUND_COLOUR_OFFSET),
                shadow,
            );
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's outline and background colour.
    pub(in super::super) fn set_textdraw_outline(
        self,
        pool_index: u16,
        outline: u8,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_BACKGROUND_COLOUR_OFFSET) as *mut u8;
        let len =
            TEXTDRAW_OUTLINE_OFFSET + mem::size_of::<u8>() - TEXTDRAW_BACKGROUND_COLOUR_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field.cast::<u32>(), colour);
            ptr::write_unaligned(
                field.add(TEXTDRAW_OUTLINE_OFFSET - TEXTDRAW_BACKGROUND_COLOUR_OFFSET),
                outline,
            );
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's finite box dimensions and colour.
    pub(in super::super) fn set_textdraw_box(
        self,
        pool_index: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS || !width.is_finite() || !height.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_BOX_ENABLED_OFFSET) as *mut u8;
        let len = TEXTDRAW_BOX_COLOUR_OFFSET + mem::size_of::<u32>() - TEXTDRAW_BOX_ENABLED_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field, u8::from(enabled));
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_BOX_WIDTH_OFFSET - TEXTDRAW_BOX_ENABLED_OFFSET)
                    .cast::<f32>(),
                width,
            );
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_BOX_HEIGHT_OFFSET - TEXTDRAW_BOX_ENABLED_OFFSET)
                    .cast::<f32>(),
                height,
            );
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_BOX_COLOUR_OFFSET - TEXTDRAW_BOX_ENABLED_OFFSET)
                    .cast::<u32>(),
                colour,
            );
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's one-of-three alignment flags.
    pub(in super::super) fn set_textdraw_alignment(
        self,
        pool_index: u16,
        alignment: u8,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS || !(1..=3).contains(&alignment) {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_ALIGN_CENTER_OFFSET) as *mut u8;
        let len = TEXTDRAW_ALIGN_RIGHT_OFFSET + 1 - TEXTDRAW_ALIGN_CENTER_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field, u8::from(alignment == 2));
            ptr::write_unaligned(
                field.add(TEXTDRAW_ALIGN_LEFT_OFFSET - TEXTDRAW_ALIGN_CENTER_OFFSET),
                u8::from(alignment == 1),
            );
            ptr::write_unaligned(
                field.add(TEXTDRAW_ALIGN_RIGHT_OFFSET - TEXTDRAW_ALIGN_CENTER_OFFSET),
                u8::from(alignment == 3),
            );
        }
        Ok(())
    }

    /// Replaces one existing R1 textdraw's bounded display string.
    pub(in super::super) fn set_textdraw_model_style(
        self,
        pool_index: u16,
        rotation: Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS
            || !rotation.x.is_finite()
            || !rotation.y.is_finite()
            || !rotation.z.is_finite()
            || !zoom.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end))
            || !read_r1_bool(
                pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET
                    + usize::from(pool_index) * mem::size_of::<i32>(),
            )?
        {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_ROTATION_OFFSET) as *mut u8;
        let len = TEXTDRAW_MODEL_COLOUR2_OFFSET + mem::size_of::<u16>() - TEXTDRAW_ROTATION_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field.cast::<f32>(), rotation.x);
            ptr::write_unaligned(field.add(4).cast::<f32>(), rotation.y);
            ptr::write_unaligned(field.add(8).cast::<f32>(), rotation.z);
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_ZOOM_OFFSET - TEXTDRAW_ROTATION_OFFSET)
                    .cast::<f32>(),
                zoom,
            );
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_MODEL_COLOUR1_OFFSET - TEXTDRAW_ROTATION_OFFSET)
                    .cast::<u16>(),
                colour1,
            );
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_MODEL_COLOUR2_OFFSET - TEXTDRAW_ROTATION_OFFSET)
                    .cast::<u16>(),
                colour2,
            );
        }
        Ok(())
    }

    /// Replaces one existing R1 textdraw's bounded display string.
    pub(in super::super) fn set_textdraw_string(
        self,
        pool_index: u16,
        text: &[u8],
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS
            || text.len() > MAX_TEXTDRAW_STRING_BYTES
            || text.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let destination = (object + 801) as *mut u8;
        if !writable_range(destination, MAX_TEXTDRAW_STRING_BYTES + 1) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_bytes(destination, 0, MAX_TEXTDRAW_STRING_BYTES + 1);
            ptr::copy_nonoverlapping(text.as_ptr(), destination, text.len());
        }
        Ok(())
    }

    /// Copies one R1 numeric textdraw record on the game-thread pump. The raw
    /// index preserves the native 2,048-global then 256-local pool order. No
    /// textdraw/pool pointer or unproven display-string buffer crosses the
    /// private profile boundary.
    pub(in super::super) fn textdraw(
        self,
        pool_index: u16,
    ) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, flags_end) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Ok(None);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let last_field_end = TEXTDRAW_MODEL_COLOUR2_OFFSET + mem::size_of::<u16>();
        if !readable_range(object as *const u8, last_field_end) {
            return Err(DirectClientError::NotReady);
        }
        let letter_width = unsafe { read_unaligned::<f32>(object + TEXTDRAW_LETTER_WIDTH_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let letter_height =
            unsafe { read_unaligned::<f32>(object + TEXTDRAW_LETTER_HEIGHT_OFFSET) }
                .filter(|value| value.is_finite())
                .ok_or(DirectClientError::NotReady)?;
        let letter_colour =
            unsafe { read_unaligned::<u32>(object + TEXTDRAW_LETTER_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let x = unsafe { read_unaligned::<f32>(object + TEXTDRAW_X_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let y = unsafe { read_unaligned::<f32>(object + TEXTDRAW_Y_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let box_width = unsafe { read_unaligned::<f32>(object + TEXTDRAW_BOX_WIDTH_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let box_height = unsafe { read_unaligned::<f32>(object + TEXTDRAW_BOX_HEIGHT_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let box_colour = unsafe { read_unaligned::<u32>(object + TEXTDRAW_BOX_COLOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let background_colour =
            unsafe { read_unaligned::<u32>(object + TEXTDRAW_BACKGROUND_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let style = unsafe { read_unaligned::<i32>(object + TEXTDRAW_STYLE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let model_id = unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let rotation = unsafe { read_vector3(object + TEXTDRAW_ROTATION_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let zoom = unsafe { read_unaligned::<f32>(object + TEXTDRAW_ZOOM_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let text = unsafe {
            bounded_c_string(
                (object + TEXTDRAW_STRING_OFFSET) as *const u8,
                MAX_TEXTDRAW_STRING_BYTES + 1,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        Ok(Some(TextdrawSnapshot {
            pool_index,
            text,
            letter_width,
            letter_height,
            letter_colour,
            x,
            y,
            shadow: unsafe { read_unaligned::<u8>(object + TEXTDRAW_SHADOW_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            outline: unsafe { read_unaligned::<u8>(object + TEXTDRAW_OUTLINE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            background_colour,
            style,
            proportional: read_u8_bool(object + TEXTDRAW_PROPORTIONAL_OFFSET)?,
            align_left: read_u8_bool(object + TEXTDRAW_ALIGN_LEFT_OFFSET)?,
            align_center: read_u8_bool(object + TEXTDRAW_ALIGN_CENTER_OFFSET)?,
            align_right: read_u8_bool(object + TEXTDRAW_ALIGN_RIGHT_OFFSET)?,
            box_enabled: read_u8_bool(object + TEXTDRAW_BOX_ENABLED_OFFSET)?,
            box_width,
            box_height,
            box_colour,
            model_id,
            rotation,
            zoom,
            model_colour1: unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_COLOUR1_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            model_colour2: unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_COLOUR2_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
        }))
    }
}
