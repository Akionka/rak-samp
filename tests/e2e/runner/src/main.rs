//! Loads the ASI ABI fixture host and an independently built plugin DLL.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_e2e_runner supports only 32-bit Windows x86 targets");

use std::{
    ffi::CString,
    fs, mem,
    path::Path,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Foundation::{FreeLibrary, HMODULE},
    System::LibraryLoader::{GetProcAddress, LoadLibraryA},
};

type DispatchIncomingRpc = unsafe extern "system" fn(u8) -> i32;
type ListenerCount = unsafe extern "system" fn() -> u32;
type PluginReady = unsafe extern "system" fn() -> i32;
type PluginCallbackCount = unsafe extern "system" fn() -> u32;
type PluginShutdown = unsafe extern "system" fn() -> i32;

struct LoadedModule(HMODULE);

impl LoadedModule {
    fn load(path: &Path) -> Result<Self, String> {
        let path = CString::new(path.as_os_str().as_encoded_bytes())
            .map_err(|_| format!("fixture path contains a NUL byte: {}", path.display()))?;
        let handle = unsafe { LoadLibraryA(path.as_ptr().cast()) };
        if handle.is_null() {
            return Err(format!(
                "failed to load {}: {}",
                path.to_string_lossy(),
                std::io::Error::last_os_error()
            ));
        }
        Ok(Self(handle))
    }

    fn export(
        &self,
        name: &std::ffi::CStr,
    ) -> Result<unsafe extern "system" fn() -> isize, String> {
        let address = unsafe { GetProcAddress(self.0, name.as_ptr().cast()) };
        address.ok_or_else(|| {
            format!(
                "{} does not export {}",
                self.0 as usize,
                name.to_string_lossy()
            )
        })
    }
}

impl Drop for LoadedModule {
    fn drop(&mut self) {
        unsafe { FreeLibrary(self.0) };
    }
}

fn main() -> Result<(), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .ok_or_else(|| "could not resolve the workspace root".to_owned())?;
    let artifact_dir = root.join("target/i686-pc-windows-msvc/release");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let fixture_dir =
        std::env::temp_dir().join(format!("rak-samp-e2e-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&fixture_dir).map_err(|error| error.to_string())?;
    let result = run(&artifact_dir, &fixture_dir);
    let _ = fs::remove_dir_all(&fixture_dir);
    result
}

fn run(artifact_dir: &Path, fixture_dir: &Path) -> Result<(), String> {
    let host_path = fixture_dir.join("rak_samp.asi");
    let plugin_path = fixture_dir.join("rak_samp_e2e_plugin.asi");
    copy_artifact(artifact_dir, "rak_samp_e2e_host.dll", &host_path)?;
    copy_artifact(artifact_dir, "rak_samp_e2e_plugin.dll", &plugin_path)?;

    let host = LoadedModule::load(&host_path)?;
    let dispatch: DispatchIncomingRpc =
        unsafe { mem::transmute(host.export(c"RakSampE2eHost_DispatchIncomingRpc")?) };
    let listener_count: ListenerCount =
        unsafe { mem::transmute(host.export(c"RakSampE2eHost_ListenerCount")?) };

    let plugin = LoadedModule::load(&plugin_path)?;
    let ready: PluginReady = unsafe { mem::transmute(plugin.export(c"RakSampE2ePlugin_Ready")?) };
    let callback_count: PluginCallbackCount =
        unsafe { mem::transmute(plugin.export(c"RakSampE2ePlugin_CallbackCount")?) };
    let shutdown: PluginShutdown =
        unsafe { mem::transmute(plugin.export(c"RakSampE2ePlugin_Shutdown")?) };

    wait_until("plugin registration", || unsafe { ready() != 0 })?;
    if unsafe { listener_count() } != 1 {
        return Err("plugin did not register exactly one incoming RPC listener".to_owned());
    }
    if unsafe { dispatch(42) } == 0 {
        return Err("fixture host dispatched no callback".to_owned());
    }
    wait_until("plugin callback", || unsafe { callback_count() } == 1)?;
    if unsafe { shutdown() } == 0 {
        return Err("plugin shutdown did not synchronize its subscription".to_owned());
    }
    if unsafe { listener_count() } != 0 {
        return Err("host retained a listener after plugin shutdown".to_owned());
    }
    drop(plugin);
    if unsafe { dispatch(42) } != 0 {
        return Err("host retained a callback after FreeLibrary".to_owned());
    }
    drop(host);
    println!("E2E ASI ABI fixture passed");
    Ok(())
}

fn copy_artifact(artifact_dir: &Path, name: &str, destination: &Path) -> Result<(), String> {
    let source = artifact_dir.join(name);
    fs::copy(&source, destination).map_err(|error| {
        format!(
            "failed to copy {} to {}: {error}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn wait_until(label: &str, mut condition: impl FnMut() -> bool) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if condition() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err(format!("timed out waiting for {label}"))
}
