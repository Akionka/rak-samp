use super::{CallbackState, ChatCommandCallbackState, HostApi};
use crate::{SampClientSdkResult, SampClientSdkSubscription};
use core::fmt;

/// An owned packet or RPC callback registration.
///
/// Call [`Self::unregister_and_wait`] from a worker thread before unloading the plugin ASI.
/// Dropping this value attempts a nonblocking listener removal and intentionally retains the
/// callback allocation, so it is memory-safe but does not prepare a plugin for `FreeLibrary`.
#[must_use = "a subscription must be synchronized before unloading the plugin ASI"]
pub struct Subscription {
    pub(crate) api: HostApi,
    pub(crate) raw: SampClientSdkSubscription,
    pub(crate) callback: Option<Box<CallbackState>>,
}

impl Subscription {
    /// Returns this registration's host-assigned identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.raw.id
    }

    /// Removes this listener and waits until the host cannot invoke its callback anymore.
    ///
    /// Call this from a worker thread, never from `DllMain` or from this subscription's callback.
    /// On failure, the returned error retains the subscription so shutdown can be retried.
    pub fn unregister_and_wait(mut self) -> Result<(), SubscriptionShutdownError> {
        let result = unsafe { (self.api.raw.unregister_and_wait)(self.raw) };
        if matches!(
            result,
            SampClientSdkResult::Ok | SampClientSdkResult::SubscriptionNotFound
        ) {
            drop(self.callback.take());
            Ok(())
        } else {
            Err(SubscriptionShutdownError {
                result,
                subscription: self,
            })
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            // Do not wait here: Drop may run inside DllMain or a callback. The host listener is
            // detached, but the allocation must stay valid for any callback already in flight.
            let _ = unsafe { (self.api.raw.unregister)(self.raw) };
            let _ = Box::leak(callback);
        }
    }
}

impl fmt::Debug for Subscription {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Subscription")
            .field("id", &self.id())
            .finish_non_exhaustive()
    }
}

/// A synchronized subscription removal that the host could not complete.
#[derive(Debug)]
pub struct SubscriptionShutdownError {
    result: SampClientSdkResult,
    subscription: Subscription,
}

impl SubscriptionShutdownError {
    /// Returns the host result that prevented synchronized removal.
    #[must_use]
    pub const fn result(&self) -> SampClientSdkResult {
        self.result
    }

    /// Returns the still-registered subscription so shutdown can be retried.
    pub fn into_subscription(self) -> Subscription {
        self.subscription
    }
}

impl fmt::Display for SubscriptionShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "host could not synchronize subscription {}: {:?}",
            self.subscription.id(),
            self.result
        )
    }
}

impl std::error::Error for SubscriptionShutdownError {}

/// An owned local chat-command registration.
///
/// Call [`Self::unregister_and_wait`] from a worker thread before unloading
/// the plugin ASI. Dropping this value requests asynchronous native removal
/// and deliberately retains the callback allocation if synchronization is not
/// possible.
#[must_use = "a chat-command subscription must be synchronized before unloading the plugin ASI"]
pub struct ChatCommandSubscription {
    pub(crate) api: HostApi,
    pub(crate) raw: SampClientSdkSubscription,
    pub(crate) callback: Option<Box<ChatCommandCallbackState>>,
}

impl ChatCommandSubscription {
    /// Returns this registration's host-assigned identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.raw.id
    }

    /// Removes the native R1 command and waits until its callback cannot run.
    ///
    /// Call this from a worker thread, never from `DllMain` or the command
    /// callback. On failure, the returned error retains the registration for a
    /// retry.
    pub fn unregister_and_wait(mut self) -> Result<(), ChatCommandShutdownError> {
        let result = unsafe { (self.api.raw.unregister_and_wait)(self.raw) };
        if matches!(
            result,
            SampClientSdkResult::Ok | SampClientSdkResult::SubscriptionNotFound
        ) {
            drop(self.callback.take());
            Ok(())
        } else {
            Err(ChatCommandShutdownError {
                result,
                subscription: self,
            })
        }
    }
}

impl Drop for ChatCommandSubscription {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            // Do not wait in Drop. The host disables this callback before it
            // queues native removal, and retaining the allocation keeps any
            // callback already in flight safe through plugin unload.
            let _ = unsafe { (self.api.raw.unregister)(self.raw) };
            let _ = Box::leak(callback);
        }
    }
}

impl fmt::Debug for ChatCommandSubscription {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChatCommandSubscription")
            .field("id", &self.id())
            .finish_non_exhaustive()
    }
}

/// A chat-command removal that could not be synchronized.
#[derive(Debug)]
pub struct ChatCommandShutdownError {
    result: SampClientSdkResult,
    subscription: ChatCommandSubscription,
}

impl ChatCommandShutdownError {
    /// Returns the host result that prevented synchronized removal.
    #[must_use]
    pub const fn result(&self) -> SampClientSdkResult {
        self.result
    }

    /// Returns the still-registered command for a shutdown retry.
    pub fn into_subscription(self) -> ChatCommandSubscription {
        self.subscription
    }
}

impl fmt::Display for ChatCommandShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "host could not synchronize chat-command subscription {}: {:?}",
            self.subscription.id(),
            self.result
        )
    }
}

impl std::error::Error for ChatCommandShutdownError {}

/// A group of callback subscriptions that should be stopped together.
///
/// Call [`Self::unregister_and_wait`] from a worker thread before unloading the plugin ASI.
#[must_use = "subscriptions must be synchronized before unloading the plugin ASI"]
#[derive(Debug, Default)]
pub struct SubscriptionSet {
    subscriptions: Vec<Subscription>,
}

impl SubscriptionSet {
    /// Creates an empty subscription group.
    pub const fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
        }
    }

    /// Adds one successful registration to this group.
    pub fn push(&mut self, subscription: Subscription) {
        self.subscriptions.push(subscription);
    }

    /// Returns the number of owned subscriptions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.subscriptions.len()
    }

    /// Returns whether this group has no subscriptions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.subscriptions.is_empty()
    }

    /// Adds a registration result while preserving earlier registrations if it failed.
    ///
    /// This is primarily useful to [`register_handlers!`] and other batch-registration helpers.
    pub fn try_add(
        mut self,
        registration: Result<Subscription, SampClientSdkResult>,
    ) -> Result<Self, SubscriptionRegistrationError> {
        match registration {
            Ok(subscription) => {
                self.push(subscription);
                Ok(self)
            }
            Err(result) => Err(SubscriptionRegistrationError {
                result,
                subscriptions: self,
            }),
        }
    }

    /// Stops every callback and waits until the host cannot invoke any of them.
    ///
    /// Call this from a worker thread, never from `DllMain` or from one of the registered
    /// callbacks. Failures retain only the subscriptions that still need a retry.
    pub fn unregister_and_wait(self) -> Result<(), SubscriptionSetShutdownError> {
        let mut subscriptions = Vec::new();
        let mut failures = Vec::new();
        for subscription in self.subscriptions {
            if let Err(error) = subscription.unregister_and_wait() {
                let result = error.result();
                let subscription = error.into_subscription();
                failures.push(SubscriptionShutdownFailure {
                    id: subscription.id(),
                    result,
                });
                subscriptions.push(subscription);
            }
        }
        if subscriptions.is_empty() {
            Ok(())
        } else {
            Err(SubscriptionSetShutdownError {
                failures,
                subscriptions: Self { subscriptions },
            })
        }
    }
}

/// A callback registration that failed after earlier batch registrations succeeded.
#[derive(Debug)]
pub struct SubscriptionRegistrationError {
    result: SampClientSdkResult,
    subscriptions: SubscriptionSet,
}

impl SubscriptionRegistrationError {
    /// Returns the host result from the failed registration.
    #[must_use]
    pub const fn result(&self) -> SampClientSdkResult {
        self.result
    }

    /// Returns the earlier successful registrations for synchronized cleanup or retry.
    pub fn into_subscriptions(self) -> SubscriptionSet {
        self.subscriptions
    }
}

impl fmt::Display for SubscriptionRegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "host rejected a callback registration: {:?}",
            self.result
        )
    }
}

impl std::error::Error for SubscriptionRegistrationError {}

/// One subscription that the host could not synchronize.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubscriptionShutdownFailure {
    id: u64,
    result: SampClientSdkResult,
}

impl SubscriptionShutdownFailure {
    /// Returns the host-assigned subscription identifier.
    #[must_use]
    pub const fn id(self) -> u64 {
        self.id
    }

    /// Returns the host result that prevented synchronized removal.
    #[must_use]
    pub const fn result(self) -> SampClientSdkResult {
        self.result
    }
}

/// A batch shutdown that left one or more callbacks registered.
#[derive(Debug)]
pub struct SubscriptionSetShutdownError {
    failures: Vec<SubscriptionShutdownFailure>,
    subscriptions: SubscriptionSet,
}

impl SubscriptionSetShutdownError {
    /// Returns each callback that still needs synchronized removal.
    #[must_use]
    pub fn failures(&self) -> &[SubscriptionShutdownFailure] {
        &self.failures
    }

    /// Returns the remaining subscriptions so shutdown can be retried.
    pub fn into_subscriptions(self) -> SubscriptionSet {
        self.subscriptions
    }
}

impl fmt::Display for SubscriptionSetShutdownError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "host could not synchronize {} subscriptions",
            self.failures.len()
        )
    }
}

impl std::error::Error for SubscriptionSetShutdownError {}

/// Registers a batch of packet and RPC handlers into one [`SubscriptionSet`].
///
/// The macro accepts raw `packet`, `rpc`, `packet_id`, and `rpc_id` entries plus directional
/// `incoming_typed_packet`, `outgoing_typed_packet`, `incoming_typed_rpc`,
/// `outgoing_typed_rpc`, `incoming_protocol_packet`, `outgoing_protocol_packet`,
/// `incoming_protocol_rpc`, and `outgoing_protocol_rpc` entries. If one registration fails, the
/// error retains every earlier successful subscription so the caller can synchronize them before
/// unloading the plugin.
#[macro_export]
macro_rules! register_handlers {
    ($api:expr; $($kind:ident($($argument:expr),*)),+ $(,)?) => {{
        (|| -> Result<$crate::SubscriptionSet, $crate::SubscriptionRegistrationError> {
            let api = $api;
            let subscriptions = $crate::SubscriptionSet::new();
            $(
                let subscriptions = $crate::register_handlers!(
                    @add subscriptions, api, $kind, $($argument),*
                )?;
            )+
            Ok(subscriptions)
        })()
    }};
    (@add $subscriptions:ident, $api:ident, packet, $direction:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_packet($direction, $handler))
    };
    (@add $subscriptions:ident, $api:ident, rpc, $direction:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_rpc($direction, $handler))
    };
    (@add $subscriptions:ident, $api:ident, packet_id, $direction:expr, $id:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_packet_id($direction, $id, $handler))
    };
    (@add $subscriptions:ident, $api:ident, rpc_id, $direction:expr, $id:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_rpc_id($direction, $id, $handler))
    };
    (@add $subscriptions:ident, $api:ident, incoming_typed_packet, $descriptor:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_incoming_typed_packet($descriptor, $handler))
    };
    (@add $subscriptions:ident, $api:ident, outgoing_typed_packet, $descriptor:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_outgoing_typed_packet($descriptor, $handler))
    };
    (@add $subscriptions:ident, $api:ident, incoming_typed_rpc, $descriptor:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_incoming_typed_rpc($descriptor, $handler))
    };
    (@add $subscriptions:ident, $api:ident, outgoing_typed_rpc, $descriptor:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_outgoing_typed_rpc($descriptor, $handler))
    };
    (@add $subscriptions:ident, $api:ident, incoming_protocol_packet, $descriptor:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_incoming_protocol_packet($descriptor, $handler))
    };
    (@add $subscriptions:ident, $api:ident, outgoing_protocol_packet, $descriptor:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_outgoing_protocol_packet($descriptor, $handler))
    };
    (@add $subscriptions:ident, $api:ident, incoming_protocol_rpc, $descriptor:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_incoming_protocol_rpc($descriptor, $handler))
    };
    (@add $subscriptions:ident, $api:ident, outgoing_protocol_rpc, $descriptor:expr, $handler:expr) => {
        $subscriptions.try_add($api.on_outgoing_protocol_rpc($descriptor, $handler))
    };
}
