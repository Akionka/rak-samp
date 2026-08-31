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

    #[must_use]
    pub fn id(&self) -> Option<u64> {
        self.id.map(|id| id.0)
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

    pub fn try_take(&mut self) -> Result<Option<()>, ModResult> {
        self.poll()
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

    pub fn release(mut self) -> Result<(), ModResult> {
        let Some(id) = self.id.take() else {
            return Ok(());
        };
        self.core.receipt_release(id)
    }
}

impl Drop for CommandReceipt {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.receipt_release(id);
        }
    }
}

#[must_use = "dropping a pending receipt detaches it without cancelling the command"]
pub struct TextLabelCreateReceipt {
    core: Core,
    id: Option<CommandReceiptId>,
}

impl TextLabelCreateReceipt {
    pub(crate) fn new(core: Core, id: CommandReceiptId) -> Result<Self, ModResult> {
        if id.is_zero() {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        }
        Ok(Self { core, id: Some(id) })
    }

    #[must_use]
    pub fn id(&self) -> Option<u64> {
        self.id.map(|id| id.0)
    }

    pub fn try_take(&mut self) -> Result<Option<crate::TextLabelId>, ModResult> {
        let Some(id) = self.id else {
            return Err(modkit_abi::MOD_INVALID_ARGUMENT);
        };
        let Some(completion) = self.core.receipt_poll(id)? else {
            return Ok(None);
        };
        self.id = None;
        text_label_completion(completion).map(Some)
    }

    pub fn wait(mut self, timeout: Duration) -> Result<crate::TextLabelId, ModResult> {
        let Some(id) = self.id else {
            return Err(modkit_abi::MOD_INVALID_ARGUMENT);
        };
        let completion = self.core.receipt_wait(id, timeout)?;
        self.id = None;
        text_label_completion(completion)
    }

    pub fn release(mut self) -> Result<(), ModResult> {
        let Some(id) = self.id.take() else {
            return Ok(());
        };
        self.core.receipt_release(id)
    }
}

impl Drop for TextLabelCreateReceipt {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.receipt_release(id);
        }
    }
}

fn text_label_completion(
    completion: modkit_abi::CommandCompletionV1,
) -> Result<crate::TextLabelId, ModResult> {
    if !completion.status.is_ok() {
        return Err(completion.status);
    }
    let raw = u16::try_from(completion.value0).map_err(|_| modkit_abi::MOD_NATIVE_CALL_FAILED)?;
    crate::TextLabelId::new(raw).ok_or(modkit_abi::MOD_NATIVE_CALL_FAILED)
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

    pub fn unregister_and_wait(&mut self, timeout: Duration) -> Result<(), ModResult> {
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
