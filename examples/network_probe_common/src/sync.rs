//! Sync snapshots, force-sync receipts, packet observations, and vehicle phases.

use super::*;

pub(super) fn record_vehicle_phase(message: &[u8]) {
    let mut phases = VEHICLE_PHASES
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if let Some(values) = parse_u16_fields(message, LOCAL_DRIVER_READY_PREFIX) {
        phases.local_driver = (values.len() == 1).then_some(values[0]);
    } else if let Some(values) = parse_u16_fields(message, LOCAL_PASSENGER_READY_PREFIX) {
        phases.local_passenger = (values.len() == 1).then_some(values[0]);
    } else if let Some(values) = parse_u16_fields(message, LOCAL_TRAILER_READY_PREFIX) {
        phases.local_trailer = (values.len() == 2).then(|| VehiclePair {
            vehicle: values[0],
            trailer: values[1],
        });
    } else if message == VEHICLE_CLEANUP_READY_MARKER {
        phases.cleanup = true;
    }
}

pub(super) fn parse_u16_fields(message: &[u8], prefix: &[u8]) -> Option<Vec<u16>> {
    message
        .strip_prefix(prefix)?
        .split(|byte| *byte == b',')
        .map(|value| std::str::from_utf8(value).ok()?.parse::<u16>().ok())
        .collect()
}

pub(super) fn verify_force_sync_receipts(samp: Samp) -> Result<(), SampClientSdkResult> {
    wait_for_receipt(samp.local().force_aim_sync()?)?;
    wait_for_receipt(samp.local().force_onfoot_sync()?)?;
    wait_for_receipt(samp.local().force_stats_sync()?)?;
    wait_for_receipt(samp.local().force_weapons_sync()?)
}

pub(super) fn verify_packet_after_command(
    count_index: usize,
    submit: impl FnOnce() -> Result<CommandReceipt<()>, SampClientSdkResult>,
) -> Result<(), SampClientSdkResult> {
    let before = SYNC_PACKET_COUNTS[count_index].load(Ordering::Acquire);
    wait_for_receipt(submit()?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        Ok(SYNC_PACKET_COUNTS[count_index].load(Ordering::Acquire) > before)
    })
}

pub(super) fn verify_vehicle_sync(samp: Samp) -> Result<(), SampClientSdkResult> {
    let local_id =
        PlayerId::new(wait_for_value(SCALAR_CACHE_TIMEOUT, || samp.local().player())?.id)
            .ok_or(SampClientSdkResult::NativeCallFailed)?;

    set_vehicle_phase("driver_request_before");
    wait_for_receipt(probe_protocol_send(
        samp.net().send_chat(LOCAL_DRIVER_REQUEST),
    )?)?;
    set_vehicle_phase("driver_request_after");
    let local_vehicle = wait_for_vehicle_phase(SCALAR_CACHE_TIMEOUT, |phases| phases.local_driver)?;
    let local_vehicle_id =
        VehicleId::new(local_vehicle).ok_or(SampClientSdkResult::NativeCallFailed)?;
    set_vehicle_phase("driver_snapshot_before");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.players().player(local_id).vehicle_sync().map(|sync| {
            sync.is_some_and(|sync| in_car_sync_is_valid(sync, local_id, local_vehicle))
        })
    })?;
    set_vehicle_phase("driver_snapshot_after");
    set_vehicle_phase("driver_force_before");
    verify_packet_after_command(SYNC_INDEX_VEHICLE, || {
        samp.local().force_vehicle_sync(local_vehicle_id)
    })?;
    set_vehicle_phase("driver_force_after");

    set_vehicle_phase("passenger_request_before");
    wait_for_receipt(probe_protocol_send(
        samp.net().send_chat(LOCAL_PASSENGER_REQUEST),
    )?)?;
    set_vehicle_phase("passenger_request_after");
    let passenger_vehicle =
        wait_for_vehicle_phase(SCALAR_CACHE_TIMEOUT, |phases| phases.local_passenger)?;
    if passenger_vehicle != local_vehicle {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    set_vehicle_phase("passenger_snapshot_before");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.players()
            .player(local_id)
            .passenger_sync()
            .map(|sync| {
                sync.is_some_and(|sync| {
                    sync.id == local_id.get()
                        && sync.vehicle_id == local_vehicle
                        && sync.seat_id == 1
                        && vector_is_finite(sync.position)
                })
            })
    })?;
    set_vehicle_phase("passenger_snapshot_after");
    set_vehicle_phase("passenger_force_before");
    verify_packet_after_command(SYNC_INDEX_PASSENGER, || {
        samp.local().force_passenger_sync(local_vehicle_id, 1)
    })?;
    set_vehicle_phase("passenger_force_after");

    set_vehicle_phase("trailer_request_before");
    wait_for_receipt(probe_protocol_send(
        samp.net().send_chat(LOCAL_TRAILER_REQUEST),
    )?)?;
    set_vehicle_phase("trailer_request_after");
    let local_trailer =
        wait_for_vehicle_phase(SCALAR_CACHE_TIMEOUT, |phases| phases.local_trailer)?;
    let trailer_id =
        VehicleId::new(local_trailer.trailer).ok_or(SampClientSdkResult::NativeCallFailed)?;
    set_vehicle_phase("trailer_snapshot_before");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        let player = samp.players().player(local_id);
        let in_car = player.vehicle_sync()?;
        let trailer = player.trailer_sync()?;
        Ok(in_car.is_some_and(|sync| {
            in_car_sync_is_valid(sync, local_id, local_trailer.vehicle)
                && sync.trailer_id == local_trailer.trailer
        }) && trailer.is_some_and(|sync| {
            sync.id == local_id.get()
                && sync.trailer_id == local_trailer.trailer
                && vector_is_finite(sync.position)
                && vector_is_finite(sync.speed)
                && vector_is_finite(sync.turn_speed)
                && sync.quaternion.into_iter().all(f32::is_finite)
        }))
    })?;
    set_vehicle_phase("trailer_snapshot_after");
    set_vehicle_phase("trailer_force_before");
    verify_packet_after_command(SYNC_INDEX_TRAILER, || {
        samp.local().force_trailer_sync(trailer_id)
    })?;
    set_vehicle_phase("trailer_force_after");
    let truck_id =
        VehicleId::new(local_trailer.vehicle).ok_or(SampClientSdkResult::NativeCallFailed)?;
    set_vehicle_phase("unoccupied_force_before");
    verify_packet_after_command(SYNC_INDEX_UNOCCUPIED, || {
        samp.local().force_unoccupied_sync(truck_id, 0)
    })?;
    set_vehicle_phase("unoccupied_force_after");

    let vehicle_packet_mask =
        SYNC_PACKET_VEHICLE | SYNC_PACKET_PASSENGER | SYNC_PACKET_UNOCCUPIED | SYNC_PACKET_TRAILER;
    set_vehicle_phase("packet_mask_before");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        Ok(
            SYNC_PACKETS_OBSERVED.load(Ordering::Acquire) & vehicle_packet_mask
                == vehicle_packet_mask,
        )
    })?;
    set_vehicle_phase("packet_mask_after");

    set_vehicle_phase("cleanup_request_before");
    wait_for_receipt(probe_protocol_send(
        samp.net().send_chat(VEHICLE_CLEANUP_REQUEST),
    )?)?;
    set_vehicle_phase("cleanup_request_after");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        Ok(VEHICLE_PHASES
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .cleanup)
    })?;
    set_vehicle_phase("complete");
    Ok(())
}

pub(super) fn set_vehicle_phase(phase: &'static str) {
    *VEHICLE_PHASE
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = phase;
    publish_status();
}

pub(super) fn wait_for_vehicle_phase<T: Copy>(
    timeout: Duration,
    read: impl Fn(&VehiclePhases) -> Option<T>,
) -> Result<T, SampClientSdkResult> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(value) = read(
            &VEHICLE_PHASES
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        ) {
            return Ok(value);
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn in_car_sync_is_valid(
    sync: samp_client_sdk::InCarSync,
    player: PlayerId,
    vehicle: u16,
) -> bool {
    sync.id == player.get()
        && sync.vehicle_id == vehicle
        && vector_is_finite(sync.position)
        && vector_is_finite(sync.speed)
        && sync.vehicle_health.is_finite()
        && sync.quaternion.into_iter().all(f32::is_finite)
}

pub(super) fn verify_sync_snapshots(samp: Samp) -> Result<(), SampClientSdkResult> {
    let local_id =
        PlayerId::new(wait_for_value(SCALAR_CACHE_TIMEOUT, || samp.local().player())?.id)
            .ok_or(SampClientSdkResult::NativeCallFailed)?;
    let remote_id = find_remote_player(samp, SCALAR_CACHE_TIMEOUT)?;
    wait_for_condition(CHAT_INPUT_CACHE_TIMEOUT, || {
        let local = samp.players().player(local_id);
        let remote = samp.players().player(remote_id);
        let Some(local_onfoot) = local.onfoot_sync()? else {
            return Ok(false);
        };
        let Some(local_aim) = local.aim_sync()? else {
            return Ok(false);
        };
        let Some(remote_onfoot) = remote.onfoot_sync()? else {
            return Ok(false);
        };
        Ok(local_onfoot.id == local_id.get()
            && local_aim.id == local_id.get()
            && remote_onfoot.id == remote_id.get()
            && vector_is_finite(local_onfoot.position)
            && vector_is_finite(local_onfoot.speed)
            && local_onfoot.quaternion.into_iter().all(f32::is_finite)
            && vector_is_finite(local_aim.aim_first)
            && vector_is_finite(local_aim.aim_position)
            && local_aim.aim_z.is_finite()
            && vector_is_finite(remote_onfoot.position)
            && vector_is_finite(remote_onfoot.speed)
            && remote_onfoot.quaternion.into_iter().all(f32::is_finite))
    })
}
