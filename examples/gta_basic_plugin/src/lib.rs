//! Minimal typed GTA SA tick plugin.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("gta-basic-plugin supports only 32-bit Windows x86 targets");

use gta_sa::{CameraSnapshot, Error, HostGtaSaExt, TickSubscription, TimerSnapshot, Vector3};
use modkit_sdk::Host;
use std::{
    collections::VecDeque,
    ffi::c_void,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{HINSTANCE, TRUE},
    System::{
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
};
use windows_sys::core::BOOL;

static SUBSCRIPTION: Mutex<Option<TickSubscription>> = Mutex::new(None);
const STATUS_PATH: &str = "gta-basic-plugin.status";
const OBSERVATION_CAPACITY: usize = 256;

#[derive(Clone, Copy)]
struct CameraObservation {
    frame: u32,
    camera: CameraSnapshot,
}

type CameraObservations = Arc<(Mutex<VecDeque<CameraObservation>>, Condvar)>;

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(
    instance: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe { DisableThreadLibraryCalls(instance) };
            let _ = std::thread::Builder::new()
                .name("gta-basic-plugin-init".into())
                .spawn(initialize);
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    TRUE
}

fn initialize() {
    let _ = std::fs::remove_file(STATUS_PATH);
    let host = match Host::connect(Duration::from_secs(120)) {
        Ok(host) => host,
        Err(error) => {
            eprintln!("gta-basic-plugin: host connection failed: {error}");
            return;
        }
    };
    let gta = match host.gta_sa() {
        Ok(gta) => gta,
        Err(error) => {
            eprintln!("gta-basic-plugin: GTA service unavailable: {error:?}");
            return;
        }
    };
    let observations = CameraObservations::default();
    let callback_observations = Arc::clone(&observations);
    let subscription = match gta.on_tick(move |context| {
        let timer = context.timer().snapshot()?;
        let camera = context.camera().snapshot()?;
        publish_camera_observation(&callback_observations, timer, camera);
        match context.player() {
            Ok(player) => {
                let snapshot = player.snapshot()?;
                let position = player.position();
                let exists = context.peds().exists(snapshot.handle)?;
                let ground_z = context.world().ground_z(position.x, position.y)?;
                let _ = (position, snapshot.health, exists, ground_z, timer.time_step);
            }
            Err(Error::NoLocalPed) => {}
            Err(error) => return Err(error),
        }
        Ok(())
    }) {
        Ok(subscription) => subscription,
        Err(error) => {
            eprintln!("gta-basic-plugin: tick registration failed: {error:?}");
            return;
        }
    };
    *SUBSCRIPTION
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(subscription);

    let status = run_camera_probe(gta, &observations)
        .unwrap_or_else(|error| format!("STATUS=FAIL error={error}\n"));
    if let Err(error) = std::fs::write(STATUS_PATH, status.as_bytes()) {
        eprintln!("gta-basic-plugin: status write failed: {error}");
    }
}

fn publish_camera_observation(
    observations: &CameraObservations,
    timer: TimerSnapshot,
    camera: CameraSnapshot,
) {
    let (queue, ready) = &**observations;
    let mut queue = queue.lock().unwrap_or_else(|error| error.into_inner());
    if queue.len() == OBSERVATION_CAPACITY {
        queue.pop_front();
    }
    queue.push_back(CameraObservation {
        frame: timer.frame_counter,
        camera,
    });
    drop(queue);
    ready.notify_all();
}

fn run_camera_probe(gta: gta_sa::Gta, observations: &CameraObservations) -> Result<String, String> {
    let timer_receipt = gta
        .timer()
        .snapshot()
        .map_err(|error| format!("submit_timer:{error:?}"))?;
    let camera_receipt = gta
        .camera()
        .snapshot()
        .map_err(|error| format!("submit_camera:{error:?}"))?;
    let queued_timer = timer_receipt
        .wait(Duration::from_secs(10))
        .map_err(|error| format!("wait_timer:{error:?}"))?;
    let queued_camera = camera_receipt
        .wait(Duration::from_secs(10))
        .map_err(|error| format!("wait_camera:{error:?}"))?;
    let direct_camera = wait_for_camera_frame(
        observations,
        queued_timer.frame_counter,
        Duration::from_secs(10),
    )?;
    if direct_camera != queued_camera {
        return Err(format!(
            "camera_mismatch frame={} direct={} queued={}",
            queued_timer.frame_counter,
            format_camera(direct_camera),
            format_camera(queued_camera)
        ));
    }
    Ok(format!(
        "STATUS=PASS frame={} camera={}\n",
        queued_timer.frame_counter,
        format_camera(queued_camera)
    ))
}

fn wait_for_camera_frame(
    observations: &CameraObservations,
    frame: u32,
    timeout: Duration,
) -> Result<CameraSnapshot, String> {
    let deadline = Instant::now() + timeout;
    let (queue, ready) = &**observations;
    let mut queue = queue.lock().unwrap_or_else(|error| error.into_inner());
    loop {
        if let Some(observation) = queue.iter().find(|observation| observation.frame == frame) {
            return Ok(observation.camera);
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(format!("direct_frame_timeout frame={frame}"));
        }
        let (next, result) = ready
            .wait_timeout(queue, remaining)
            .unwrap_or_else(|error| error.into_inner());
        queue = next;
        if result.timed_out() {
            return Err(format!("direct_frame_timeout frame={frame}"));
        }
    }
}

fn format_camera(camera: CameraSnapshot) -> String {
    format!(
        "game:{};right:{};forward:{};up:{};position:{}",
        format_vector(camera.game_position),
        format_vector(camera.transform.right),
        format_vector(camera.transform.forward),
        format_vector(camera.transform.up),
        format_vector(camera.transform.position)
    )
}

fn format_vector(vector: Vector3) -> String {
    format!(
        "{:08X},{:08X},{:08X}",
        vector.x.to_bits(),
        vector.y.to_bits(),
        vector.z.to_bits()
    )
}

/// Stops callbacks before an unload manager calls `FreeLibrary`.
#[unsafe(no_mangle)]
pub extern "system" fn GtaBasicPlugin_Shutdown() -> BOOL {
    let subscription = SUBSCRIPTION
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take();
    let Some(subscription) = subscription else {
        return TRUE;
    };
    if subscription
        .unregister_and_wait(Duration::from_secs(10))
        .is_ok()
    {
        TRUE
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_status_is_bounded_and_machine_readable() {
        let record = format!(
            "STATUS=PASS frame=42 camera={}\n",
            format_camera(CameraSnapshot::default())
        );
        assert!(record.starts_with("STATUS=PASS frame=42 camera=game:"));
        assert!(record.contains(";right:"));
        assert!(record.contains(";position:"));
        assert!(record.len() < 512);
    }
}
