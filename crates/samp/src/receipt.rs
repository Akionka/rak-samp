use modkit_abi::{CommandReceiptId, ModResult, SubscriptionId};
use modkit_sdk::Core;
use std::time::Duration;

#[must_use = "dropping a pending receipt detaches it without cancelling the command"]
pub struct CommandReceipt {
    core: Core,
    id: Option<CommandReceiptId>,
}

impl CommandReceipt {
    pub(crate) fn new(core: Core, id: CommandReceiptId) -> Result<Self, ModResult> {
        if id.is_zero() {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        }
        Ok(Self { core, id: Some(id) })
    }

    pub fn poll(&mut self) -> Result<Option<()>, ModResult> {
        let Some(id) = self.id else {
            return Ok(Some(()));
        };
        let Some(completion) = self.core.receipt_poll(id)? else {
            return Ok(None);
        };
        self.id = None;
        if completion.status.is_ok() {
            Ok(Some(()))
        } else {
            Err(completion.status)
        }
    }

    pub fn wait(mut self, timeout: Duration) -> Result<(), ModResult> {
        let Some(id) = self.id else {
            return Ok(());
        };
        let completion = self.core.receipt_wait(id, timeout)?;
        self.id = None;
        if completion.status.is_ok() {
            Ok(())
        } else {
            Err(completion.status)
        }
    }
}

impl Drop for CommandReceipt {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.receipt_release(id);
        }
    }
}

#[must_use = "keep the subscription alive while callbacks are required"]
pub struct Subscription {
    core: Core,
    id: Option<SubscriptionId>,
}

impl Subscription {
    pub(crate) fn new(core: Core, id: SubscriptionId) -> Result<Self, ModResult> {
        if id.is_zero() {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        }
        Ok(Self { core, id: Some(id) })
    }

    pub fn unregister_and_wait(mut self, timeout: Duration) -> Result<(), ModResult> {
        let Some(id) = self.id else {
            return Ok(());
        };
        self.core.unregister_and_wait(id, timeout)?;
        self.id = None;
        Ok(())
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.unregister(id);
        }
    }
}
