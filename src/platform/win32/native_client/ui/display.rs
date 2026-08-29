//! Cursor and scoreboard operations.

use super::*;

type R1GameSetCursorModeFn = unsafe extern "thiscall" fn(*mut c_void, i32, i32);
type ClassicGameSetCursorModeFn = unsafe extern "thiscall" fn(*mut c_void, i32, i32);
type R1GameProcessInputEnablingFn = unsafe extern "thiscall" fn(*mut c_void);
type ClassicGameProcessInputEnablingFn = unsafe extern "thiscall" fn(*mut c_void);

impl NativeClientProfile {
    pub(crate) fn set_cursor_mode(self, mode: i32) -> Result<(), DirectClientError> {
        if !matches!(mode, 0..=4) {
            return Err(DirectClientError::NotReady);
        }
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let target = self.ui_target(self.spec.ui.game.set_cursor_mode_rva)?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let set_mode: R1GameSetCursorModeFn = mem::transmute(target);
                    set_mode(game, mode, i32::from(mode != 0));
                }
                PoolGetterAbi::Classic => {
                    let set_mode: ClassicGameSetCursorModeFn = mem::transmute(target);
                    set_mode(game, mode, i32::from(mode != 0));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn toggle_cursor(self, show: bool) -> Result<(), DirectClientError> {
        self.set_cursor_mode(if show { 3 } else { 0 })?;
        if !show {
            let game = self.game().ok_or(DirectClientError::NotReady)?;
            let target = self.ui_target(self.spec.ui.game.process_input_enabling_rva)?;
            unsafe {
                match self.spec.strategies.pool_getter_abi {
                    PoolGetterAbi::R1 => {
                        let process: R1GameProcessInputEnablingFn = mem::transmute(target);
                        process(game);
                    }
                    PoolGetterAbi::Classic => {
                        let process: ClassicGameProcessInputEnablingFn = mem::transmute(target);
                        process(game);
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn set_scoreboard_open(self, open: bool) -> Result<(), DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        unsafe {
            write_unaligned(
                (scoreboard as usize)
                    .checked_add(self.spec.ui.scoreboard.enabled_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
                i32::from(open),
            )
            .then_some(())
            .ok_or(DirectClientError::NotReady)
        }
    }

    pub(crate) fn cursor_mode(self) -> Result<i32, DirectClientError> {
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let mode = unsafe {
            read_unaligned::<i32>(
                (game as usize)
                    .checked_add(self.spec.ui.game.cursor_mode_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        matches!(mode, 0..=4)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn scoreboard_is_open(self) -> Result<bool, DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        read_i32_bool(
            (scoreboard as usize)
                .checked_add(self.spec.ui.scoreboard.enabled_offset.get())
                .ok_or(DirectClientError::NotReady)?,
        )
    }
}
