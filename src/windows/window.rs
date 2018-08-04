extern crate winapi;
extern crate user32;

use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::mem;

use self::winapi::windef::*;
use self::winapi::winuser::*;
use self::user32::*;

pub struct Window {
    handle: HWND
}

impl Window {
    pub fn find(name: &str) -> Vec<Window> {
        // todo: how optimal is that string conversion?
        let title = OsString::from(name).as_os_str().encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();

        let mut handles = Vec::new();
        let mut handle = ptr::null_mut();
        loop {
            handle = unsafe { FindWindowExW(ptr::null_mut(), handle, ptr::null_mut(), title.as_ptr()) };
            if handle == ptr::null_mut() {
                break;
            }
            else {
                handles.push(Window { handle });
            }
        }

        handles
    }

    pub fn foreground() -> Option<Window> {
        let handle = unsafe { GetForegroundWindow() };

        if handle == ptr::null_mut() {
            None
        } else {
            Some(
                Window {
                    handle: handle
                }
            )
        }
    }

    pub fn is_valid(&self) -> bool {
        unsafe { IsWindow(self.handle) > 0 }
    }

    pub fn is_foreground(&self) -> bool {
        unsafe { GetForegroundWindow() == self.handle }
    }

    pub fn is_full_screen(&self) -> bool {
        unsafe {
            let w = GetSystemMetrics(SM_CXSCREEN);
            let h = GetSystemMetrics(SM_CYSCREEN);

            if GetWindowLongW(self.handle, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST > 0 {
                let mut rect: RECT = mem::zeroed();
                GetWindowRect(self.handle, &mut rect);
                if (w == (rect.right - rect.left)) && (h == (rect.bottom - rect.top)) {
                    return true;
                }
            }
        }

        false
    }
}