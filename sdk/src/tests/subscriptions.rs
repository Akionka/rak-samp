//! Subscription lifecycle tests.

use super::*;

#[test]
fn subscription_set_retains_each_failed_shutdown_for_retry() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let api = test_support::test_api();
    let mut subscriptions = SubscriptionSet::new();
    subscriptions.push(
        api.on_packet(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed"),
    );
    subscriptions.push(
        api.on_rpc(SampClientSdkDirection::Outgoing, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed"),
    );
    test_support::set_unregister_and_wait_result(SampClientSdkResult::CallbackInProgress);

    let error = subscriptions
        .unregister_and_wait()
        .expect_err("failed callbacks must remain available for retry");
    assert_eq!(error.failures().len(), 2);
    assert!(
        error
            .failures()
            .iter()
            .all(|failure| failure.result() == SampClientSdkResult::CallbackInProgress)
    );
    assert_eq!(test_support::registration_stats().registered_callbacks, 2);

    test_support::set_unregister_and_wait_result(SampClientSdkResult::Ok);
    error
        .into_subscriptions()
        .unregister_and_wait()
        .expect("retry must synchronize every callback");
    let stats = test_support::registration_stats();
    assert_eq!(stats.unregister_and_wait_calls, 4);
    assert_eq!(stats.registered_callbacks, 0);
}

#[test]
fn subscription_set_preserves_earlier_registrations_after_a_registration_failure() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let subscription = test_support::test_api()
        .on_packet(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");

    let error = SubscriptionSet::new()
        .try_add(Ok(subscription))
        .and_then(|subscriptions| subscriptions.try_add(Err(SampClientSdkResult::NotReady)))
        .expect_err("the synthetic second registration must fail");
    assert_eq!(error.result(), SampClientSdkResult::NotReady);
    let subscriptions = error.into_subscriptions();
    assert_eq!(subscriptions.len(), 1);
    subscriptions
        .unregister_and_wait()
        .expect("retained subscription must remain cleanly removable");
}

#[test]
fn failed_synchronized_shutdown_keeps_the_subscription_for_retry() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let subscription = test_support::test_api()
        .on_rpc(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");
    test_support::set_unregister_and_wait_result(SampClientSdkResult::CallbackInProgress);

    let error = subscription
        .unregister_and_wait()
        .expect_err("callback-thread shutdown must remain retryable");
    assert_eq!(error.result(), SampClientSdkResult::CallbackInProgress);
    let subscription = error.into_subscription();
    test_support::set_unregister_and_wait_result(SampClientSdkResult::Ok);
    subscription
        .unregister_and_wait()
        .expect("retry must synchronize");
}

#[test]
fn failed_registration_releases_the_handler() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    test_support::set_register_result(SampClientSdkResult::NotReady);
    let drops = Arc::new(AtomicUsize::new(0));
    let counter = DropCounter(Arc::clone(&drops));

    let result = test_support::test_api().on_packet(SampClientSdkDirection::Incoming, move |_| {
        let _ = &counter;
        SampClientSdkHookAction::Continue
    });
    assert_eq!(result.unwrap_err(), SampClientSdkResult::NotReady);
    assert_eq!(drops.load(Ordering::Acquire), 1);
}

#[test]
fn dropping_a_subscription_detaches_without_freeing_callback_state() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let drops = Arc::new(AtomicUsize::new(0));
    let counter = DropCounter(Arc::clone(&drops));
    let subscription = test_support::test_api()
        .on_packet(SampClientSdkDirection::Incoming, move |_| {
            let _ = &counter;
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");

    drop(subscription);
    assert_eq!(drops.load(Ordering::Acquire), 0);
    assert_eq!(test_support::invoke_registered_callback(1), None);
    assert_eq!(test_support::registration_stats().unregister_calls, 1);
}
