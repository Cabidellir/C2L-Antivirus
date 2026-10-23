//! Per-user notification-area icon; never invoked from the service session.
use std::{io, mem::size_of, ptr::null_mut};
use windows_sys::Win32::{
    Foundation::HWND,
    UI::{Shell::*, WindowsAndMessaging::*},
};

pub struct Tray {
    window: HWND,
    icon: NOTIFYICONDATAW,
}
fn wide<const N: usize>(text: &str) -> [u16; N] {
    let mut buffer = [0; N];
    for (to, from) in buffer.iter_mut().take(N - 1).zip(text.encode_utf16()) {
        *to = from;
    }
    buffer
}
impl Tray {
    pub fn new() -> io::Result<Self> {
        let class = wide::<7>("STATIC");
        let title = wide::<32>("C2L Notifications");
        // A hidden top-level window belongs to this interactive thread.
        let window = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
            )
        };
        if window.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut icon: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        icon.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        icon.hWnd = window;
        icon.uID = 1;
        icon.uFlags = NIF_ICON | NIF_TIP;
        icon.hIcon = unsafe { LoadIconW(null_mut(), IDI_APPLICATION) };
        icon.szTip = wide("C2L — avisos de deteção. Abra Monitorização para detalhes.");
        let tray = Self { window, icon };
        if unsafe { Shell_NotifyIconW(NIM_ADD, &tray.icon) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(tray)
    }
    /// Success means accepted by the shell, not proof the user saw the message.
    pub fn notify(&mut self, title: &str, message: &str) -> io::Result<()> {
        self.icon.uFlags = NIF_ICON | NIF_TIP | NIF_INFO;
        self.icon.szInfoTitle = wide(title);
        self.icon.szInfo = wide(message);
        self.icon.dwInfoFlags = NIIF_WARNING | NIIF_RESPECT_QUIET_TIME;
        if unsafe { Shell_NotifyIconW(NIM_MODIFY, &self.icon) } == 0
            && unsafe { Shell_NotifyIconW(NIM_ADD, &self.icon) } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
    pub fn pump(&self) {
        let mut message = unsafe { std::mem::zeroed() };
        unsafe {
            while PeekMessageW(&mut message, self.window, 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
}
impl Drop for Tray {
    fn drop(&mut self) {
        unsafe {
            Shell_NotifyIconW(NIM_DELETE, &self.icon);
            DestroyWindow(self.window);
        }
    }
}
