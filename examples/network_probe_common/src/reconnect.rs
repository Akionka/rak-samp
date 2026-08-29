//! Disconnect invalidation and reconnect restoration validation.

use super::*;

pub(super) fn verify_reconnect_on_request(samp: Samp) -> Result<(), SampClientSdkResult> {
    RECONNECT_REQUESTED.store(false, Ordering::Release);
    let reconnect_command = samp
        .chat_input()
        .register_command(RECONNECT_COMMAND_NAME, |_| {
            RECONNECT_REQUESTED.store(true, Ordering::Release);
        })?;
    wait_for_receipt(samp.chat().add(LocalChatMessage {
        style: LocalChatMessageStyle::Info,
        text: MAIN_PASS_MESSAGE,
        prefix: b"",
        text_colour: 0xFF6FCF97,
        prefix_colour: 0,
    })?)?;

    let request = wait_for_condition(HOST_CONNECTION_TIMEOUT, || {
        Ok(RECONNECT_REQUESTED.load(Ordering::Acquire))
    });
    let unregister = reconnect_command
        .unregister_and_wait()
        .map_err(|error| error.result());
    request?;
    unregister?;

    wait_for_receipt(samp.net().disconnect(500)?)?;
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        let server_not_ready = matches!(samp.server().info(), Err(SampClientSdkResult::NotReady));
        let local_not_ready = matches!(samp.local().player(), Err(SampClientSdkResult::NotReady));
        #[cfg(feature = "r1-probe")]
        let raw_connection_state_invalidated = matches!(
            (
                unsafe { raw::player_pool(samp) },
                unsafe { raw::vehicle_pool(samp) },
                unsafe { raw::player(samp) },
            ),
            (
                Err(SampClientSdkResult::NotReady),
                Err(SampClientSdkResult::NotReady),
                Err(SampClientSdkResult::NotReady),
            )
        );
        #[cfg(not(feature = "r1-probe"))]
        let raw_connection_state_invalidated = true;
        if server_not_ready
            && local_not_ready
            && raw_connection_state_invalidated
            && !samp.net().incoming_emulation_ready()
        {
            break;
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
    STATUS.fetch_or(STATUS_DISCONNECT_INVALIDATION, Ordering::AcqRel);
    publish_status();

    wait_for_receipt(samp.net().connect(b"127.0.0.1", 7777)?)?;
    wait_for_condition(HOST_CONNECTION_TIMEOUT, || {
        let server = samp.server().info();
        let local = samp.local().player();
        let game_state = samp.game_state().ok();
        let observation = ReconnectObservation {
            server_ready: server
                .as_ref()
                .is_ok_and(|server| server.address == b"127.0.0.1" && server.port == 7777),
            local_ready: local.is_ok(),
            game_state,
            spawned: local.as_ref().ok().map(|local| local.spawned),
            incoming_ready: samp.net().incoming_emulation_ready(),
        };
        let ready = observation.server_ready
            && observation.game_state == Some(PROFILE_CONNECTED_STATE)
            && observation.spawned == Some(true)
            && observation.incoming_ready;
        let mut published = RECONNECT_OBSERVATION
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if published.as_ref() != Some(&observation) {
            *published = Some(observation);
            drop(published);
            publish_status();
        }
        Ok(ready)
    })?;

    let replies_before = INCOMING_REPLY_COUNT.load(Ordering::Acquire);
    wait_for_receipt(probe_protocol_send(samp.net().send_chat(OUTBOUND_MARKER))?)?;
    wait_for_condition(CALLBACK_TIMEOUT, || {
        Ok(INCOMING_REPLY_COUNT.load(Ordering::Acquire) > replies_before)
    })?;
    STATUS.fetch_or(STATUS_RECONNECT_RESTORED, Ordering::AcqRel);
    publish_status();
    Ok(())
}
