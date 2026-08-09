//! Published forward and reverse GTA handle cache reads.

use super::{
    BackendState, HandleCacheEntry, MAX_SAMP_OBJECTS, MAX_SAMP_PICKUPS, MAX_SAMP_PLAYERS,
    MAX_SAMP_VEHICLES, OBJECT_HANDLE_REQUEST_QUEUE_CAPACITY,
    OBJECT_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY, PICKUP_HANDLE_REQUEST_QUEUE_CAPACITY,
    PICKUP_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY, PLAYER_HANDLE_REQUEST_QUEUE_CAPACITY,
    PLAYER_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY, VEHICLE_HANDLE_REQUEST_QUEUE_CAPACITY,
    VEHICLE_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
};
use crate::runtime::DirectClientError;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Mutex, atomic::Ordering},
};

impl BackendState {
    pub(super) fn object_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.cached_handle(
            usize::from(id),
            MAX_SAMP_OBJECTS,
            &self.object_handle_cache,
            &self.object_handle_requests,
            OBJECT_HANDLE_REQUEST_QUEUE_CAPACITY,
            self.rak_client.load(Ordering::Acquire) != 0,
        )
    }

    pub(super) fn object_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.cached_handle_id(
            handle,
            &self.object_handle_reverse_cache,
            &self.object_handle_reverse_requests,
            OBJECT_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
            self.rak_client.load(Ordering::Acquire) != 0,
        )
    }

    pub(super) fn pickup_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.cached_handle(
            usize::from(id),
            MAX_SAMP_PICKUPS,
            &self.pickup_handle_cache,
            &self.pickup_handle_requests,
            PICKUP_HANDLE_REQUEST_QUEUE_CAPACITY,
            self.rak_client.load(Ordering::Acquire) != 0,
        )
    }

    pub(super) fn pickup_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.cached_handle_id(
            handle,
            &self.pickup_handle_reverse_cache,
            &self.pickup_handle_reverse_requests,
            PICKUP_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
            self.rak_client.load(Ordering::Acquire) != 0,
        )
    }

    pub(super) fn vehicle_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.cached_handle(
            usize::from(id),
            MAX_SAMP_VEHICLES,
            &self.vehicle_handle_cache,
            &self.vehicle_handle_requests,
            VEHICLE_HANDLE_REQUEST_QUEUE_CAPACITY,
            self.rak_client.load(Ordering::Acquire) != 0,
        )
    }

    pub(super) fn vehicle_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.cached_handle_id(
            handle,
            &self.vehicle_handle_reverse_cache,
            &self.vehicle_handle_reverse_requests,
            VEHICLE_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
            self.rak_client.load(Ordering::Acquire) != 0,
        )
    }

    pub(super) fn player_ped_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.cached_handle(
            usize::from(id),
            MAX_SAMP_PLAYERS,
            &self.player_handle_cache,
            &self.player_handle_requests,
            PLAYER_HANDLE_REQUEST_QUEUE_CAPACITY,
            self.rak_client.load(Ordering::Acquire) != 0,
        )
    }

    pub(super) fn player_id_by_ped_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.cached_handle_id(
            handle,
            &self.player_handle_reverse_cache,
            &self.player_handle_reverse_requests,
            PLAYER_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
            self.rak_client.load(Ordering::Acquire) != 0,
        )
    }

    pub(super) fn cached_handle(
        &self,
        index: usize,
        maximum: usize,
        cache: &Mutex<Vec<HandleCacheEntry>>,
        requests: &Mutex<VecDeque<u16>>,
        queue_capacity: usize,
        client_available: bool,
    ) -> Result<Option<i32>, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if index >= maximum || !client_available || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        match cache
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
            .get(index)
            .cloned()
            .ok_or(DirectClientError::NotReady)?
        {
            HandleCacheEntry::Known(handle) => {
                let _ = self.queue_handle_request(requests, queue_capacity, index as u16);
                Ok(handle)
            }
            HandleCacheEntry::Unknown => {
                self.queue_handle_request(requests, queue_capacity, index as u16)?;
                Err(DirectClientError::NotReady)
            }
        }
    }

    pub(super) fn cached_handle_id(
        &self,
        handle: i32,
        cache: &Mutex<HashMap<i32, Option<u16>>>,
        requests: &Mutex<VecDeque<i32>>,
        queue_capacity: usize,
        client_available: bool,
    ) -> Result<Option<u16>, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if !client_available || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if let Some(id) = cache
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
            .get(&handle)
            .copied()
        {
            let _ = self.queue_handle_id_request(requests, queue_capacity, handle);
            return Ok(id);
        }
        self.queue_handle_id_request(requests, queue_capacity, handle)?;
        Err(DirectClientError::NotReady)
    }
}
