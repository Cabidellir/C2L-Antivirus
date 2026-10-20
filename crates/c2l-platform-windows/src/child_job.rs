//! Own only the application's child processes; closing the GUI also closes the job.
use std::{io, os::windows::io::AsRawHandle, process::Child, ptr};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::JobObjects::*,
};
pub struct ChildJob(HANDLE);
impl Drop for ChildJob {
    fn drop(&mut self) {
        // SAFETY: this wrapper exclusively owns the valid job handle.
        unsafe {
            CloseHandle(self.0);
        }
    }
}
impl ChildJob {
    pub fn attach(child: &mut Child) -> io::Result<Self> {
        let result = (|| {
            // SAFETY: null security/name requests an unnamed non-inheritable job.
            let handle = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
            if handle.is_null() {
                return Err(io::Error::last_os_error());
            }
            let job = Self(handle);
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            limits.BasicLimitInformation.LimitFlags =
                JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
            limits.ProcessMemoryLimit = 2 * 1024 * 1024 * 1024;
            // SAFETY: live job and exact initialized structure; child handle is borrowed.
            if unsafe {
                SetInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    std::mem::size_of_val(&limits) as u32,
                )
            } == 0
                || unsafe { AssignProcessToJobObject(handle, child.as_raw_handle()) } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(job)
        })();
        if result.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        result
    }
    pub fn peak_memory(&self) -> io::Result<u64> {
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        // SAFETY: live job handle and a correctly sized writable result structure.
        if unsafe {
            QueryInformationJobObject(
                self.0,
                JobObjectExtendedLimitInformation,
                (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(limits.PeakProcessMemoryUsed as u64)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closing_job_terminates_only_attached_child() {
        let shell = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let mut child = std::process::Command::new(shell)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 30",
            ])
            .spawn()
            .unwrap();
        let job = ChildJob::attach(&mut child).unwrap();
        drop(job);
        let start = std::time::Instant::now();
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            assert!(start.elapsed() < std::time::Duration::from_secs(5));
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
    }
}
