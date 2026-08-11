//! Producer-side bounded cache-refresh request queues.

use super::{
    BackendState, CHAT_ENTRY_REQUEST_QUEUE_CAPACITY, CHAT_ENTRY_REQUESTS_PER_PUMP,
    GANGZONE_REQUEST_QUEUE_CAPACITY, GANGZONE_REQUESTS_PER_PUMP, INCAR_SYNC_REQUEST_QUEUE_CAPACITY,
    INCAR_SYNC_REQUESTS_PER_PUMP, OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY,
    OBJECT_EXISTS_REQUESTS_PER_PUMP, OBJECT_HANDLE_REQUESTS_PER_PUMP,
    OBJECT_HANDLE_REVERSE_REQUESTS_PER_PUMP, ONFOOT_SYNC_REQUEST_QUEUE_CAPACITY,
    ONFOOT_SYNC_REQUESTS_PER_PUMP, PICKUP_HANDLE_REQUESTS_PER_PUMP,
    PICKUP_HANDLE_REVERSE_REQUESTS_PER_PUMP, PLAYER_HANDLE_REQUESTS_PER_PUMP,
    PLAYER_HANDLE_REVERSE_REQUESTS_PER_PUMP, PLAYER_INFO_REQUEST_QUEUE_CAPACITY,
    PLAYER_INFO_REQUESTS_PER_PUMP, REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY,
    REMOTE_PLAYER_STATE_REQUESTS_PER_PUMP, TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY,
    TEXT_LABEL_EXISTS_REQUESTS_PER_PUMP, TEXT_LABEL_REQUEST_QUEUE_CAPACITY,
    TEXT_LABEL_REQUESTS_PER_PUMP, TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY,
    TEXTDRAW_EXISTS_REQUESTS_PER_PUMP, TEXTDRAW_REQUEST_QUEUE_CAPACITY, TEXTDRAW_REQUESTS_PER_PUMP,
    VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY, VEHICLE_EXISTS_REQUESTS_PER_PUMP,
    VEHICLE_HANDLE_REQUESTS_PER_PUMP, VEHICLE_HANDLE_REVERSE_REQUESTS_PER_PUMP, try_lock_direct,
};
use crate::runtime::DirectClientError;
use std::{collections::VecDeque, sync::Mutex};

fn queue_unique_request<T: PartialEq>(
    requests: &Mutex<VecDeque<T>>,
    queue_capacity: usize,
    value: T,
) -> Result<(), DirectClientError> {
    let mut requests = try_lock_direct(requests)?;
    if requests.contains(&value) {
        return Ok(());
    }
    if requests.len() == queue_capacity {
        return Err(DirectClientError::QueueFull);
    }
    requests.push_back(value);
    Ok(())
}

fn take_requests<T>(queue: &Mutex<VecDeque<T>>, limit: usize) -> Vec<T> {
    queue
        .try_lock()
        .map(|mut queue| {
            let count = queue.len().min(limit);
            queue.drain(..count).collect()
        })
        .unwrap_or_default()
}

impl BackendState {
    pub(super) fn queue_player_info_request(&self, id: u16) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.player_info_requests,
            PLAYER_INFO_REQUEST_QUEUE_CAPACITY,
            id,
        )
    }

    pub(super) fn queue_remote_player_state_request(
        &self,
        id: u16,
    ) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.remote_player_state_requests,
            REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY,
            id,
        )
    }

    pub(super) fn queue_onfoot_sync_request(&self, id: u16) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.onfoot_sync_requests,
            ONFOOT_SYNC_REQUEST_QUEUE_CAPACITY,
            id,
        )
    }

    pub(super) fn queue_incar_sync_request(&self, id: u16) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.incar_sync_requests,
            INCAR_SYNC_REQUEST_QUEUE_CAPACITY,
            id,
        )
    }

    pub(super) fn queue_vehicle_exists_request(&self, id: u16) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.vehicle_exists_requests,
            VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY,
            id,
        )
    }

    pub(super) fn queue_text_label_exists_request(&self, id: u16) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.text_label_exists_requests,
            TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY,
            id,
        )
    }

    pub(super) fn queue_text_label_request(&self, id: u16) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.text_label_requests,
            TEXT_LABEL_REQUEST_QUEUE_CAPACITY,
            id,
        )
    }

    pub(super) fn queue_textdraw_exists_request(
        &self,
        pool_index: u16,
    ) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.textdraw_exists_requests,
            TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY,
            pool_index,
        )
    }

    pub(super) fn queue_textdraw_request(&self, pool_index: u16) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.textdraw_requests,
            TEXTDRAW_REQUEST_QUEUE_CAPACITY,
            pool_index,
        )
    }

    pub(super) fn queue_chat_entry_request(&self, id: u16) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.chat_entry_requests,
            CHAT_ENTRY_REQUEST_QUEUE_CAPACITY,
            id,
        )
    }

    pub(super) fn queue_object_exists_request(&self, id: u16) -> Result<(), DirectClientError> {
        queue_unique_request(
            &self.object_exists_requests,
            OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY,
            id,
        )
    }

    pub(super) fn queue_gangzone_request(&self, id: u16) -> Result<(), DirectClientError> {
        queue_unique_request(&self.gangzone_requests, GANGZONE_REQUEST_QUEUE_CAPACITY, id)
    }

    pub(super) fn queue_handle_request(
        &self,
        requests: &Mutex<VecDeque<u16>>,
        queue_capacity: usize,
        id: u16,
    ) -> Result<(), DirectClientError> {
        queue_unique_request(requests, queue_capacity, id)
    }

    pub(super) fn queue_handle_id_request(
        &self,
        requests: &Mutex<VecDeque<i32>>,
        queue_capacity: usize,
        handle: i32,
    ) -> Result<(), DirectClientError> {
        queue_unique_request(requests, queue_capacity, handle)
    }

    pub(super) fn take_player_info_requests(&self) -> Vec<u16> {
        take_requests(&self.player_info_requests, PLAYER_INFO_REQUESTS_PER_PUMP)
    }

    pub(super) fn take_remote_player_state_requests(&self) -> Vec<u16> {
        take_requests(
            &self.remote_player_state_requests,
            REMOTE_PLAYER_STATE_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_onfoot_sync_requests(&self) -> Vec<u16> {
        take_requests(&self.onfoot_sync_requests, ONFOOT_SYNC_REQUESTS_PER_PUMP)
    }

    pub(super) fn take_incar_sync_requests(&self) -> Vec<u16> {
        take_requests(&self.incar_sync_requests, INCAR_SYNC_REQUESTS_PER_PUMP)
    }

    pub(super) fn take_vehicle_exists_requests(&self) -> Vec<u16> {
        take_requests(
            &self.vehicle_exists_requests,
            VEHICLE_EXISTS_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_text_label_exists_requests(&self) -> Vec<u16> {
        take_requests(
            &self.text_label_exists_requests,
            TEXT_LABEL_EXISTS_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_text_label_requests(&self) -> Vec<u16> {
        take_requests(&self.text_label_requests, TEXT_LABEL_REQUESTS_PER_PUMP)
    }

    pub(super) fn take_textdraw_exists_requests(&self) -> Vec<u16> {
        take_requests(
            &self.textdraw_exists_requests,
            TEXTDRAW_EXISTS_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_textdraw_requests(&self) -> Vec<u16> {
        take_requests(&self.textdraw_requests, TEXTDRAW_REQUESTS_PER_PUMP)
    }

    pub(super) fn take_chat_entry_requests(&self) -> Vec<u16> {
        take_requests(&self.chat_entry_requests, CHAT_ENTRY_REQUESTS_PER_PUMP)
    }

    pub(super) fn take_object_exists_requests(&self) -> Vec<u16> {
        take_requests(
            &self.object_exists_requests,
            OBJECT_EXISTS_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_gangzone_requests(&self) -> Vec<u16> {
        take_requests(&self.gangzone_requests, GANGZONE_REQUESTS_PER_PUMP)
    }

    pub(super) fn take_object_handle_requests(&self) -> Vec<u16> {
        take_requests(
            &self.object_handle_requests,
            OBJECT_HANDLE_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_pickup_handle_requests(&self) -> Vec<u16> {
        take_requests(
            &self.pickup_handle_requests,
            PICKUP_HANDLE_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_vehicle_handle_requests(&self) -> Vec<u16> {
        take_requests(
            &self.vehicle_handle_requests,
            VEHICLE_HANDLE_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_player_handle_requests(&self) -> Vec<u16> {
        take_requests(
            &self.player_handle_requests,
            PLAYER_HANDLE_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_object_handle_id_requests(&self) -> Vec<i32> {
        take_requests(
            &self.object_handle_reverse_requests,
            OBJECT_HANDLE_REVERSE_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_pickup_handle_id_requests(&self) -> Vec<i32> {
        take_requests(
            &self.pickup_handle_reverse_requests,
            PICKUP_HANDLE_REVERSE_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_vehicle_handle_id_requests(&self) -> Vec<i32> {
        take_requests(
            &self.vehicle_handle_reverse_requests,
            VEHICLE_HANDLE_REVERSE_REQUESTS_PER_PUMP,
        )
    }

    pub(super) fn take_player_handle_id_requests(&self) -> Vec<i32> {
        take_requests(
            &self.player_handle_reverse_requests,
            PLAYER_HANDLE_REVERSE_REQUESTS_PER_PUMP,
        )
    }
}
