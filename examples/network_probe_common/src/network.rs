//! Listener registration, connection readiness, and network validation.

use super::*;

#[cfg(feature = "r1-probe")]
pub(super) const R1_EXACT_BIT_TEST_ID: u8 = 0xFE;
#[cfg(feature = "r1-probe")]
pub(super) const R1_EXACT_BIT_PAYLOAD: [u8; 1] = [0b1010_0000];
#[cfg(feature = "r1-probe")]
pub(super) const R1_EXACT_BIT_COUNT: usize = 3;
#[cfg(feature = "r1-probe")]
pub(super) const R1_CODEC_VALUE: &[u8] = b"samp-client-sdk-r1-network-probe";
#[cfg(feature = "r1-probe")]
pub(super) static R1_CODEC_ROUND_TRIP: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "r1-probe")]
pub(super) static R1_PACKET_EXACT_BITS: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "r1-probe")]
pub(super) static R1_RPC_EXACT_BITS: AtomicBool = AtomicBool::new(false);

pub(super) fn register_listeners(samp: Samp) -> Result<SubscriptionSet, SampClientSdkResult> {
    let reply_subscription = samp
        .net()
        .on_incoming_typed_rpc(SERVER_MESSAGE, |message| {
            if message.text == INCOMING_MARKER {
                STATUS.fetch_or(STATUS_REPLY_OBSERVED, Ordering::AcqRel);
                INCOMING_REPLY_COUNT.fetch_add(1, Ordering::AcqRel);
            }
            if let Some(ids) = parse_entity_ids(&message.text) {
                *ENTITY_IDS.lock().unwrap_or_else(|error| error.into_inner()) = Some(ids);
            }
            record_vehicle_phase(&message.text);
            // The visible normal-chat reply is the required human proof that
            // SA-MP's original incoming-RPC handler ran after this callback.
            ProtocolAction::Continue
        })?;
    let mut subscriptions = SubscriptionSet::new();
    subscriptions.push(reply_subscription);
    #[cfg(feature = "r1-probe")]
    if let Err(error) = register_r1_exact_bit_listeners(samp, &mut subscriptions) {
        STATE
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .subscriptions = Some(subscriptions);
        return Err(error);
    }
    for (packet_id, observed_bit, count_index) in [
        (SendAimSync::ID, SYNC_PACKET_AIM, SYNC_INDEX_AIM),
        (SendPlayerSync::ID, SYNC_PACKET_ONFOOT, SYNC_INDEX_ONFOOT),
        (SendStatsUpdate::ID, SYNC_PACKET_STATS, SYNC_INDEX_STATS),
        (
            SendWeaponsUpdate::ID,
            SYNC_PACKET_WEAPONS,
            SYNC_INDEX_WEAPONS,
        ),
        (SendVehicleSync::ID, SYNC_PACKET_VEHICLE, SYNC_INDEX_VEHICLE),
        (
            SendPassengerSync::ID,
            SYNC_PACKET_PASSENGER,
            SYNC_INDEX_PASSENGER,
        ),
        (
            SendUnoccupiedSync::ID,
            SYNC_PACKET_UNOCCUPIED,
            SYNC_INDEX_UNOCCUPIED,
        ),
        (SendTrailerSync::ID, SYNC_PACKET_TRAILER, SYNC_INDEX_TRAILER),
    ] {
        let subscription =
            match samp
                .net()
                .on_packet_id(SampClientSdkDirection::Outgoing, packet_id, move |_| {
                    SYNC_PACKETS_OBSERVED.fetch_or(observed_bit, Ordering::AcqRel);
                    SYNC_PACKET_COUNTS[count_index].fetch_add(1, Ordering::AcqRel);
                    SampClientSdkHookAction::Continue
                }) {
                Ok(subscription) => subscription,
                Err(error) => {
                    STATE
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .subscriptions = Some(subscriptions);
                    return Err(error);
                }
            };
        subscriptions.push(subscription);
    }
    Ok(subscriptions)
}

#[cfg(feature = "r1-probe")]
pub(super) fn register_r1_exact_bit_listeners(
    samp: Samp,
    subscriptions: &mut SubscriptionSet,
) -> Result<(), SampClientSdkResult> {
    let packet = samp.net().on_packet_id(
        SampClientSdkDirection::Incoming,
        R1_EXACT_BIT_TEST_ID,
        |event| {
            if r1_exact_bit_payload(event) {
                R1_PACKET_EXACT_BITS.store(true, Ordering::Release);
            } else {
                record_failure(SampClientSdkResult::NativeCallFailed);
            }
            SampClientSdkHookAction::Block
        },
    )?;
    subscriptions.push(packet);
    let rpc = samp.net().on_rpc_id(
        SampClientSdkDirection::Incoming,
        R1_EXACT_BIT_TEST_ID,
        |event| {
            if r1_exact_bit_payload(event) {
                R1_RPC_EXACT_BITS.store(true, Ordering::Release);
            } else {
                record_failure(SampClientSdkResult::NativeCallFailed);
            }
            SampClientSdkHookAction::Block
        },
    )?;
    subscriptions.push(rpc);
    Ok(())
}

#[cfg(feature = "r1-probe")]
pub(super) fn r1_exact_bit_payload(event: &mut samp_client_sdk::events::Event<'_>) -> bool {
    event.remaining_bits() == R1_EXACT_BIT_COUNT
        && matches!(event.read_bits(R1_EXACT_BIT_COUNT), Ok(payload) if payload == R1_EXACT_BIT_PAYLOAD)
        && event.remaining_bits() == 0
}

#[cfg(feature = "r1-probe")]
pub(super) fn verify_r1_codec_and_exact_bits(samp: Samp) -> Result<(), SampClientSdkResult> {
    let codec_deadline = Instant::now() + INITIALIZATION_TIMEOUT;
    loop {
        match r1_codec_round_trip(samp) {
            Ok(()) => break,
            Err(SampClientSdkResult::NotReady | SampClientSdkResult::Busy)
                if !codec_deadline
                    .saturating_duration_since(Instant::now())
                    .is_zero() =>
            {
                thread::sleep(RETRY_DELAY);
            }
            Err(error) => return Err(error),
        }
    }
    R1_CODEC_ROUND_TRIP.store(true, Ordering::Release);
    publish_status();

    let emulation_deadline = Instant::now() + INCOMING_READY_TIMEOUT;
    while !samp.net().incoming_emulation_ready() {
        if emulation_deadline
            .saturating_duration_since(Instant::now())
            .is_zero()
        {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }

    wait_for_receipt(samp.net().emulate_incoming_packet(
        R1_EXACT_BIT_TEST_ID,
        &R1_EXACT_BIT_PAYLOAD,
        R1_EXACT_BIT_COUNT,
    )?)?;
    wait_for_r1_exact_bit(&R1_PACKET_EXACT_BITS)?;
    wait_for_receipt(samp.net().emulate_incoming_rpc(
        R1_EXACT_BIT_TEST_ID,
        &R1_EXACT_BIT_PAYLOAD,
        R1_EXACT_BIT_COUNT,
    )?)?;
    wait_for_r1_exact_bit(&R1_RPC_EXACT_BITS)?;
    publish_status();
    Ok(())
}

#[cfg(feature = "r1-probe")]
pub(super) fn r1_codec_round_trip(samp: Samp) -> Result<(), SampClientSdkResult> {
    let encoded = samp.net().encode_string(R1_CODEC_VALUE)?;
    let mut stream = BitStream::from_bits(encoded.as_bytes().to_vec(), encoded.len_bits())
        .map_err(|_| SampClientSdkResult::NativeCallFailed)?;
    let decoded = samp.net().decode_string(&mut stream)?;
    (decoded == R1_CODEC_VALUE && stream.remaining_bits() == 0)
        .then_some(())
        .ok_or(SampClientSdkResult::NativeCallFailed)
}

#[cfg(feature = "r1-probe")]
pub(super) fn wait_for_r1_exact_bit(observed: &AtomicBool) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + CALLBACK_TIMEOUT;
    loop {
        if STATUS.load(Ordering::Acquire) & STATUS_FAILED != 0 {
            return Err(stored_failure());
        }
        if observed.load(Ordering::Acquire) {
            return Ok(());
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

#[cfg(feature = "r1-probe")]
pub(super) fn verify_r1_raw_addresses(samp: Samp) -> Result<(), SampClientSdkResult> {
    wait_for_value(SCALAR_CACHE_TIMEOUT, || {
        let base = unsafe { raw::base() }.ok_or(SampClientSdkResult::NotReady)?;
        let rakclient = unsafe { raw::rakclient(samp) }?;
        let rakpeer = unsafe { raw::rakpeer(samp) }?;
        let player_pool = unsafe { raw::player_pool(samp) }?;
        let vehicle_pool = unsafe { raw::vehicle_pool(samp) }?;
        let player = unsafe { raw::player(samp) }?;
        (base.as_ptr() != rakclient.as_ptr()
            && rakpeer.as_ptr() != rakclient.as_ptr()
            && player_pool.as_ptr() != vehicle_pool.as_ptr()
            && player.as_ptr() != rakclient.as_ptr())
        .then_some(())
        .ok_or(SampClientSdkResult::NativeCallFailed)
    })
}

pub(super) fn verify_runtime_identity(samp: Samp) -> Result<(), SampClientSdkResult> {
    if samp.status() != SampClientSdkHostStatus::Ready || !samp.probe().is_samp_loaded() {
        return Err(SampClientSdkResult::NotReady);
    }
    if samp.version()? != PROFILE_CLIENT_VERSION {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    let Some(base) = (unsafe { raw::base() }) else {
        return Err(SampClientSdkResult::NotReady);
    };
    if unsafe { module_entry_point_rva(base.as_ptr()) } != Some(PROFILE_ENTRY_POINT_RVA) {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    Ok(())
}

/// Reads only the bounded DOS/PE headers from an already-loaded module.
///
/// The caller first verifies the host's ready state and version, then supplies
/// the opaque `samp.dll` base. No Rust reference is constructed for client memory.
pub(super) unsafe fn module_entry_point_rva(base: *mut c_void) -> Option<u32> {
    let base = base.cast::<u8>() as usize;
    parse_pe_entry_point_rva(|offset, destination| {
        let Some(address) = base.checked_add(offset) else {
            return false;
        };
        unsafe {
            ptr::copy_nonoverlapping(
                address as *const u8,
                destination.as_mut_ptr(),
                destination.len(),
            )
        };
        true
    })
}

pub(super) fn parse_pe_entry_point_rva(
    mut read: impl FnMut(usize, &mut [u8]) -> bool,
) -> Option<u32> {
    let mut dos = [0_u8; 64];
    if !read(0, &mut dos) || dos[..2] != *b"MZ" {
        return None;
    }
    let pe_offset = usize::try_from(u32::from_le_bytes(dos[0x3C..0x40].try_into().ok()?)).ok()?;
    if !(0x40..=MAX_PE_HEADER_OFFSET).contains(&pe_offset) {
        return None;
    }
    let mut header = [0_u8; 44];
    if !read(pe_offset, &mut header)
        || header[..4] != *b"PE\0\0"
        || header[24..26] != 0x10B_u16.to_le_bytes()
    {
        return None;
    }
    Some(u32::from_le_bytes(header[40..44].try_into().ok()?))
}

pub(super) fn connect_host() -> Option<Samp> {
    let deadline = Instant::now() + HOST_CONNECTION_TIMEOUT;
    loop {
        if is_shutting_down() {
            return None;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            record_failure(SampClientSdkResult::TimedOut);
            return None;
        }
        match Samp::connect(remaining.min(RETRY_DELAY)) {
            Ok(samp) => return Some(samp),
            Err(samp_client_sdk::ResolveError::TimedOut) => {}
            Err(_) => {
                record_failure(SampClientSdkResult::NotReady);
                return None;
            }
        }
    }
}

pub(super) fn wait_for_incoming_ready(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + INCOMING_READY_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        if samp.net().incoming_emulation_ready() {
            return Ok(());
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

pub(super) fn wait_for_receipt(mut receipt: CommandReceipt<()>) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + INITIALIZATION_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        match receipt.wait(remaining.min(RETRY_DELAY)) {
            Err(SampClientSdkResult::TimedOut) => {}
            result => return result,
        }
    }
}

pub(super) fn probe_protocol_send<T>(
    result: Result<T, ProtocolSendError>,
) -> Result<T, SampClientSdkResult> {
    result.map_err(|error| match error {
        ProtocolSendError::Encode(_) => SampClientSdkResult::InvalidArgument,
        ProtocolSendError::Host(result) => result,
    })
}

pub(super) fn wait_for_status(required: u32, timeout: Duration) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + timeout;
    loop {
        if STATUS.load(Ordering::Acquire) & STATUS_FAILED != 0 {
            return Err(stored_failure());
        }
        if STATUS.load(Ordering::Acquire) & required == required {
            return Ok(());
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}
