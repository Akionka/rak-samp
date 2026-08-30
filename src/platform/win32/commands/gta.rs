//! GTA SA game-thread commands.

use super::*;
use gta_sa::{PedSnapshot, Vector3};
use std::sync::{Arc, OnceLock};

#[derive(Debug)]
pub(crate) enum GtaCommand {
    LocalPedSnapshot(Arc<OnceLock<Option<PedSnapshot>>>),
    TeleportLocalPed(Vector3),
}

impl BackendState {
    pub(crate) fn submit_gta_local_ped_snapshot(&self) -> Result<CommandId, DirectClientError> {
        let result = Arc::new(OnceLock::new());
        let id = self.queue_game_command(GameCommand::Gta(GtaCommand::LocalPedSnapshot(
            Arc::clone(&result),
        )))?;
        self.gta_snapshot_results
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(id, result);
        Ok(id)
    }

    pub(crate) fn take_gta_local_ped_snapshot(&self, id: CommandId) -> Option<Option<PedSnapshot>> {
        let mut results = self
            .gta_snapshot_results
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let snapshot = results.get(&id)?.get().copied()?;
        results.remove(&id);
        Some(snapshot)
    }

    pub(crate) fn submit_gta_teleport_local_ped(
        &self,
        destination: Vector3,
    ) -> Result<CommandId, DirectClientError> {
        if !destination.x.is_finite() || !destination.y.is_finite() || !destination.z.is_finite() {
            return Err(DirectClientError::InvalidArgument);
        }
        self.queue_game_command(GameCommand::Gta(GtaCommand::TeleportLocalPed(destination)))
    }

    pub(crate) fn gta_local_ped_snapshot(
        &self,
        token: modkit_runtime::ScopeToken,
    ) -> Result<Option<PedSnapshot>, DirectClientError> {
        self.validate_gta_context(token)?;
        unsafe { gta_sa_native::local_ped_snapshot(self.context.gta_profile) }
            .map_err(|_| DirectClientError::NotReady)
    }

    pub(crate) fn gta_teleport_local_ped(
        &self,
        token: modkit_runtime::ScopeToken,
        destination: Vector3,
    ) -> Result<(), DirectClientError> {
        self.validate_gta_context(token)?;
        unsafe { gta_sa_native::teleport_local_ped(self.context.gta_profile, destination) }.map_err(
            |error| match error {
                gta_sa_native::PedReadError::InvalidPosition => DirectClientError::InvalidArgument,
                _ => DirectClientError::NotReady,
            },
        )
    }

    fn validate_gta_context(
        &self,
        token: modkit_runtime::ScopeToken,
    ) -> Result<(), DirectClientError> {
        self.game_scope
            .validate(
                token,
                modkit_runtime::NativeExecutionConstraint::PostGameProcessOnly,
            )
            .map_err(|error| match error {
                modkit_runtime::ScopeError::ShuttingDown => DirectClientError::NotReady,
                _ => DirectClientError::InvalidArgument,
            })
    }

    pub(crate) fn execute_gta_command(
        &self,
        command: GtaCommand,
    ) -> Result<(), gta_sa_native::PedReadError> {
        match command {
            GtaCommand::LocalPedSnapshot(result) => {
                let snapshot =
                    unsafe { gta_sa_native::local_ped_snapshot(self.context.gta_profile) }?;
                let _ = result.set(snapshot);
                Ok(())
            }
            GtaCommand::TeleportLocalPed(destination) => unsafe {
                gta_sa_native::teleport_local_ped(self.context.gta_profile, destination)
            },
        }
    }
}
