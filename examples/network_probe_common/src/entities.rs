//! Entity IDs, handles, player pools, and local-player validation.

use super::*;

pub(super) fn parse_entity_ids(message: &[u8]) -> Option<EntityIds> {
    let values = message.strip_prefix(ENTITY_IDS_PREFIX)?;
    let mut fields = values
        .split(|byte| *byte == b',')
        .map(|value| std::str::from_utf8(value).ok()?.parse::<u16>().ok());
    let ids = EntityIds {
        object: fields.next()??,
        vehicle: fields.next()??,
        pickup: fields.next()??,
        gangzone: fields.next()??,
    };
    fields.next().is_none().then_some(ids)
}

pub(super) fn verify_entity_handles(samp: Samp) -> Result<(), SampClientSdkResult> {
    let request = probe_protocol_send(samp.net().send_chat(ENTITY_REQUEST_MARKER))?;
    wait_for_receipt(request)?;
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        let ids = *ENTITY_IDS.lock().unwrap_or_else(|error| error.into_inner());
        let Some(ids) = ids else {
            if deadline.saturating_duration_since(Instant::now()).is_zero() {
                return Err(SampClientSdkResult::TimedOut);
            }
            thread::sleep(RETRY_DELAY);
            continue;
        };
        let object = ObjectId::new(ids.object).ok_or(SampClientSdkResult::NativeCallFailed)?;
        let vehicle = VehicleId::new(ids.vehicle).ok_or(SampClientSdkResult::NativeCallFailed)?;
        let gangzone =
            GangzoneId::new(ids.gangzone).ok_or(SampClientSdkResult::NativeCallFailed)?;
        let local = match samp.local().player() {
            Ok(local) => local,
            Err(SampClientSdkResult::NotReady) => {
                if deadline.saturating_duration_since(Instant::now()).is_zero() {
                    return Err(SampClientSdkResult::TimedOut);
                }
                thread::sleep(RETRY_DELAY);
                continue;
            }
            Err(error) => return Err(error),
        };
        let player = samp
            .players()
            .player(PlayerId::new(local.id).ok_or(SampClientSdkResult::NativeCallFailed)?);
        let entity_results = (
            samp.objects().exists(object),
            samp.vehicles().exists(vehicle),
            samp.gangzones().get(gangzone),
            samp.objects().handle(object),
            samp.vehicles().handle(vehicle),
            samp.pickups().handle(ids.pickup),
            player.ped_handle(),
        );
        match entity_results {
            (
                Ok(true),
                Ok(true),
                Ok(Some(_)),
                Ok(Some(object_handle)),
                Ok(Some(vehicle_handle)),
                Ok(Some(pickup_handle)),
                Ok(Some(ped_handle)),
            ) => {
                let reverse_results = (
                    samp.objects().id_by_handle(object_handle),
                    samp.vehicles().id_by_handle(vehicle_handle),
                    samp.pickups().id_by_handle(pickup_handle),
                    samp.players().id_by_ped_handle(ped_handle),
                );
                match reverse_results {
                    (
                        Ok(Some(object_id)),
                        Ok(Some(vehicle_id)),
                        Ok(Some(pickup_id)),
                        Ok(Some(player_id)),
                    ) if object_id == object
                        && vehicle_id == vehicle
                        && pickup_id == ids.pickup
                        && player_id == player.id() =>
                    {
                        return Ok(());
                    }
                    (Err(SampClientSdkResult::NotReady), _, _, _)
                    | (_, Err(SampClientSdkResult::NotReady), _, _)
                    | (_, _, Err(SampClientSdkResult::NotReady), _)
                    | (_, _, _, Err(SampClientSdkResult::NotReady)) => {}
                    (Ok(_), Ok(_), Ok(_), Ok(_)) => {
                        return Err(SampClientSdkResult::NativeCallFailed);
                    }
                    (Err(error), _, _, _)
                    | (_, Err(error), _, _)
                    | (_, _, Err(error), _)
                    | (_, _, _, Err(error)) => return Err(error),
                }
            }
            (Err(SampClientSdkResult::NotReady), _, _, _, _, _, _)
            | (_, Err(SampClientSdkResult::NotReady), _, _, _, _, _)
            | (_, _, Err(SampClientSdkResult::NotReady), _, _, _, _)
            | (_, _, _, Err(SampClientSdkResult::NotReady), _, _, _)
            | (_, _, _, _, Err(SampClientSdkResult::NotReady), _, _)
            | (_, _, _, _, _, Err(SampClientSdkResult::NotReady), _)
            | (_, _, _, _, _, _, Err(SampClientSdkResult::NotReady))
            | (Ok(false), _, _, _, _, _, _)
            | (_, Ok(false), _, _, _, _, _)
            | (_, _, Ok(None), _, _, _, _)
            | (_, _, _, Ok(None), _, _, _)
            | (_, _, _, _, Ok(None), _, _)
            | (_, _, _, _, _, Ok(None), _)
            | (_, _, _, _, _, _, Ok(None)) => {}
            (Err(error), _, _, _, _, _, _)
            | (_, Err(error), _, _, _, _, _)
            | (_, _, Err(error), _, _, _, _)
            | (_, _, _, Err(error), _, _, _)
            | (_, _, _, _, Err(error), _, _)
            | (_, _, _, _, _, Err(error), _)
            | (_, _, _, _, _, _, Err(error)) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn verify_cached_cnetgame_scalars(samp: Samp) -> Result<(), SampClientSdkResult> {
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        match (samp.game_state(), samp.server().info()) {
            (Ok(game_state), Ok(info)) => {
                record_scalar_observation(game_state, &info);
                Ok(scalar_snapshot_matches(game_state, &info))
            }
            (Err(SampClientSdkResult::NotReady), _) | (_, Err(SampClientSdkResult::NotReady)) => {
                Ok(false)
            }
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    })
}

pub(super) fn scalar_snapshot_matches(game_state: i32, info: &samp_client_sdk::ServerInfo) -> bool {
    game_state == PROFILE_INITIAL_GAME_STATE
        && info.address == b"127.0.0.1"
        && info.hostname == PROFILE_SERVER_HOSTNAME
        && info.port == 7777
}

pub(super) fn record_scalar_observation(game_state: i32, info: &samp_client_sdk::ServerInfo) {
    *SCALAR_OBSERVATION
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(ScalarObservation {
        game_state,
        address: info.address.clone(),
        hostname: info.hostname.clone(),
        port: info.port,
    });
}

pub(super) fn verify_cached_local_player(samp: Samp) -> Result<(), SampClientSdkResult> {
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || match samp.local().player() {
        Ok(player) => Ok(local_player_snapshot_is_valid(&player) && player.spawned),
        Err(SampClientSdkResult::NotReady) => Ok(false),
        Err(error) => Err(error),
    })
}

pub(super) fn local_player_snapshot_is_valid(player: &samp_client_sdk::LocalPlayer) -> bool {
    player.id < 1004
        && !player.nickname.is_empty()
        && player.nickname.len() <= 256
        && player.health.is_finite()
        && player.armour.is_finite()
        && player.position.x.is_finite()
        && player.position.y.is_finite()
        && player.position.z.is_finite()
        && player.velocity.x.is_finite()
        && player.velocity.y.is_finite()
        && player.velocity.z.is_finite()
        && player.vehicle_id.is_none_or(|id| id < 2000)
}

pub(super) fn verify_cached_player_pool_scalars(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match (
            samp.players().count(true),
            samp.players().count(false),
            samp.players().max_id(),
        ) {
            (Ok(including_npcs), Ok(excluding_npcs), Ok(max_id)) => {
                let max_id = max_id.map(|id| id.get());
                *PLAYER_POOL_OBSERVATION
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()) = Some(PlayerPoolObservation {
                    including_npcs,
                    excluding_npcs,
                    max_id,
                });
                if including_npcs == excluding_npcs.saturating_add(1) && max_id.is_some() {
                    return Ok(());
                }
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            (Err(SampClientSdkResult::NotReady), _, _)
            | (_, Err(SampClientSdkResult::NotReady), _)
            | (_, _, Err(SampClientSdkResult::NotReady)) => {}
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn verify_cached_remote_player_directory(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        let local_id = match samp.local().player() {
            Ok(player) => PlayerId::new(player.id).ok_or(SampClientSdkResult::NativeCallFailed)?,
            Err(SampClientSdkResult::NotReady) => {
                if deadline.saturating_duration_since(Instant::now()).is_zero() {
                    return Err(SampClientSdkResult::TimedOut);
                }
                thread::sleep(RETRY_DELAY);
                continue;
            }
            Err(error) => return Err(error),
        };
        match samp.players().player(local_id).is_defined() {
            Ok(true) => {}
            Ok(false) => return Err(SampClientSdkResult::NativeCallFailed),
            Err(SampClientSdkResult::NotReady) => {
                if deadline.saturating_duration_since(Instant::now()).is_zero() {
                    return Err(SampClientSdkResult::TimedOut);
                }
                thread::sleep(RETRY_DELAY);
                continue;
            }
            Err(error) => return Err(error),
        }

        let Some(max_id) = samp.players().max_id()? else {
            if deadline.saturating_duration_since(Instant::now()).is_zero() {
                return Err(SampClientSdkResult::TimedOut);
            }
            thread::sleep(RETRY_DELAY);
            continue;
        };
        for raw_id in 0..=max_id.get() {
            let Some(id) = PlayerId::new(raw_id) else {
                return Err(SampClientSdkResult::NativeCallFailed);
            };
            if id == local_id {
                continue;
            }
            match (
                samp.players().player(id).is_defined(),
                samp.players().get(id),
            ) {
                (Ok(true), Ok(Some(player)))
                    if player.id == id.get()
                        && !player.is_local
                        && player.is_npc
                        && !player.nickname.is_empty() =>
                {
                    match samp.players().remote_state(id) {
                        Ok(Some(state))
                            if state.id == id.get()
                                && state.health.is_finite()
                                && state.armour.is_finite() =>
                        {
                            return Ok(());
                        }
                        Ok(None) | Err(SampClientSdkResult::NotReady) => {}
                        Err(error) => return Err(error),
                        _ => return Err(SampClientSdkResult::NativeCallFailed),
                    }
                }
                (Ok(false), _) => {}
                (Err(SampClientSdkResult::NotReady), _)
                | (_, Err(SampClientSdkResult::NotReady))
                | (Ok(true), Ok(None)) => {}
                (Err(error), _) | (_, Err(error)) => return Err(error),
                _ => return Err(SampClientSdkResult::NativeCallFailed),
            }
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn verify_local_mutations(samp: Samp) -> Result<(), SampClientSdkResult> {
    let local = wait_for_value(SCALAR_CACHE_TIMEOUT, || samp.local().player())?;
    let local_id = PlayerId::new(local.id).ok_or(SampClientSdkResult::NativeCallFailed)?;
    let player = samp.players().player(local_id);
    let original_colour = local.colour;
    let probe_colour: u32 = 0xFF6FCF97;
    wait_for_receipt(player.set_colour(probe_colour.rotate_left(8))?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        player.colour().map(|colour| colour == Some(probe_colour))
    })?;
    wait_for_receipt(player.set_colour(original_colour.rotate_left(8))?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        player
            .colour()
            .map(|colour| colour == Some(original_colour))
    })?;
    wait_for_receipt(samp.local().set_special_action(SpecialAction::None)?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.local()
            .player()
            .map(|snapshot| snapshot.special_action == SpecialAction::None.raw())
    })?;
    for kind in [
        SendRateKind::OnFoot,
        SendRateKind::InVehicle,
        SendRateKind::Aim,
    ] {
        wait_for_receipt(samp.net().set_send_rate(kind, 30)?)?;
    }
    Ok(())
}

pub(super) fn find_remote_player(
    samp: Samp,
    timeout: Duration,
) -> Result<PlayerId, SampClientSdkResult> {
    let deadline = Instant::now() + timeout;
    loop {
        let local_id = PlayerId::new(samp.local().player()?.id)
            .ok_or(SampClientSdkResult::NativeCallFailed)?;
        if let Some(max_id) = samp.players().max_id()? {
            for raw in 0..=max_id.get() {
                let id = PlayerId::new(raw).ok_or(SampClientSdkResult::NativeCallFailed)?;
                if id != local_id && samp.players().player(id).is_defined().unwrap_or(false) {
                    return Ok(id);
                }
            }
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn vector_is_finite(value: Vector3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}
