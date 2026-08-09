//! Producer-side bounded cache-refresh request queues.

use super::{
    BackendState, CHAT_ENTRY_REQUEST_QUEUE_CAPACITY, GANGZONE_REQUEST_QUEUE_CAPACITY,
    OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY, PLAYER_INFO_REQUEST_QUEUE_CAPACITY,
    REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY, TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY,
    TEXT_LABEL_REQUEST_QUEUE_CAPACITY, TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY,
    TEXTDRAW_REQUEST_QUEUE_CAPACITY, VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY,
};
use crate::runtime::DirectClientError;
use std::{collections::VecDeque, sync::Mutex};

impl BackendState {
    pub(super) fn queue_player_info_request(&self, id: u16) -> Result<(), DirectClientError> {
        let mut requests = self
            .player_info_requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&id) {
            return Ok(());
        }
        if requests.len() == PLAYER_INFO_REQUEST_QUEUE_CAPACITY {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(id);
        Ok(())
    }

    pub(super) fn queue_remote_player_state_request(
        &self,
        id: u16,
    ) -> Result<(), DirectClientError> {
        let mut requests = self
            .remote_player_state_requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&id) {
            return Ok(());
        }
        if requests.len() == REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(id);
        Ok(())
    }

    pub(super) fn queue_vehicle_exists_request(&self, id: u16) -> Result<(), DirectClientError> {
        let mut requests = self
            .vehicle_exists_requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&id) {
            return Ok(());
        }
        if requests.len() == VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(id);
        Ok(())
    }

    pub(super) fn queue_text_label_exists_request(&self, id: u16) -> Result<(), DirectClientError> {
        let mut requests = self
            .text_label_exists_requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&id) {
            return Ok(());
        }
        if requests.len() == TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(id);
        Ok(())
    }

    pub(super) fn queue_text_label_request(&self, id: u16) -> Result<(), DirectClientError> {
        let mut requests = self
            .text_label_requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&id) {
            return Ok(());
        }
        if requests.len() == TEXT_LABEL_REQUEST_QUEUE_CAPACITY {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(id);
        Ok(())
    }

    pub(super) fn queue_textdraw_exists_request(
        &self,
        pool_index: u16,
    ) -> Result<(), DirectClientError> {
        let mut requests = self
            .textdraw_exists_requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&pool_index) {
            return Ok(());
        }
        if requests.len() == TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(pool_index);
        Ok(())
    }

    pub(super) fn queue_textdraw_request(&self, pool_index: u16) -> Result<(), DirectClientError> {
        let mut requests = self
            .textdraw_requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&pool_index) {
            return Ok(());
        }
        if requests.len() == TEXTDRAW_REQUEST_QUEUE_CAPACITY {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(pool_index);
        Ok(())
    }

    pub(super) fn queue_chat_entry_request(&self, id: u16) -> Result<(), DirectClientError> {
        let mut requests = self
            .chat_entry_requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&id) {
            return Ok(());
        }
        if requests.len() == CHAT_ENTRY_REQUEST_QUEUE_CAPACITY {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(id);
        Ok(())
    }

    pub(super) fn queue_object_exists_request(&self, id: u16) -> Result<(), DirectClientError> {
        let mut requests = self
            .object_exists_requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&id) {
            return Ok(());
        }
        if requests.len() == OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(id);
        Ok(())
    }

    pub(super) fn queue_gangzone_request(&self, id: u16) -> Result<(), DirectClientError> {
        let mut requests = self
            .gangzone_requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&id) {
            return Ok(());
        }
        if requests.len() == GANGZONE_REQUEST_QUEUE_CAPACITY {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(id);
        Ok(())
    }

    pub(super) fn queue_handle_request(
        &self,
        requests: &Mutex<VecDeque<u16>>,
        queue_capacity: usize,
        id: u16,
    ) -> Result<(), DirectClientError> {
        let mut requests = requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&id) {
            return Ok(());
        }
        if requests.len() == queue_capacity {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(id);
        Ok(())
    }

    pub(super) fn queue_handle_id_request(
        &self,
        requests: &Mutex<VecDeque<i32>>,
        queue_capacity: usize,
        handle: i32,
    ) -> Result<(), DirectClientError> {
        let mut requests = requests
            .try_lock()
            .map_err(|_| DirectClientError::QueueFull)?;
        if requests.contains(&handle) {
            return Ok(());
        }
        if requests.len() == queue_capacity {
            return Err(DirectClientError::QueueFull);
        }
        requests.push_back(handle);
        Ok(())
    }
}
