//! SCM boundary. The macro-generated FFI remains outside the safe desktop crate.
use std::{
    ffi::OsString,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use windows_service::{
    define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState,
        ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};
pub const NAME: &str = "C2LMonitor";
type Worker = fn(Arc<AtomicBool>) -> Result<(), String>;
static WORKER: OnceLock<Worker> = OnceLock::new();
define_windows_service!(ffi_main, service_main);

pub fn dispatch(worker: Worker) -> Result<(), String> {
    WORKER
        .set(worker)
        .map_err(|_| "Service dispatcher already initialized".to_owned())?;
    service_dispatcher::start(NAME, ffi_main).map_err(|e| e.to_string())
}
fn status(state: ServiceState, code: u32) -> ServiceStatus {
    ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: state,
        controls_accepted: if state == ServiceState::Running {
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN
        } else {
            ServiceControlAccept::empty()
        },
        exit_code: ServiceExitCode::Win32(code),
        checkpoint: if state == ServiceState::StopPending {
            1
        } else {
            0
        },
        wait_hint: if state == ServiceState::StopPending {
            Duration::from_secs(30)
        } else {
            Duration::ZERO
        },
        process_id: None,
    }
}
fn service_main(_: Vec<OsString>) {
    let stop = Arc::new(AtomicBool::new(false));
    let signal = stop.clone();
    let Ok(handle) = service_control_handler::register(NAME, move |control| match control {
        ServiceControl::Stop | ServiceControl::Shutdown => {
            signal.store(true, Ordering::Release);
            ServiceControlHandlerResult::NoError
        }
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        _ => ServiceControlHandlerResult::NotImplemented,
    }) else {
        return;
    };
    if handle
        .set_service_status(status(ServiceState::Running, 0))
        .is_err()
    {
        return;
    }
    let Some(worker) = WORKER.get().copied() else {
        return;
    };
    let worker_stop = stop.clone();
    let task = std::thread::spawn(move || worker(worker_stop));
    while !task.is_finished() && !stop.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(100));
    }
    if stop.load(Ordering::Acquire) {
        let _ = handle.set_service_status(status(ServiceState::StopPending, 0));
    }
    let mut checkpoint = 1;
    while !task.is_finished() {
        let mut pending = status(ServiceState::StopPending, 0);
        pending.checkpoint = checkpoint;
        checkpoint += 1;
        let _ = handle.set_service_status(pending);
        std::thread::sleep(Duration::from_secs(1));
    }
    let ok = matches!(task.join(), Ok(Ok(())));
    let _ = handle.set_service_status(status(ServiceState::Stopped, if ok { 0 } else { 1066 }));
}
/// Query-only SCM access; never starts/stops/reconfigures a service from the GUI.
pub fn is_running() -> Result<bool, String> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
        .map_err(|e| e.to_string())?;
    let service = manager
        .open_service(NAME, ServiceAccess::QUERY_STATUS)
        .map_err(|e| e.to_string())?;
    Ok(service
        .query_status()
        .map_err(|e| e.to_string())?
        .current_state
        == ServiceState::Running)
}
