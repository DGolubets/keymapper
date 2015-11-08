extern crate winapi;
extern crate user32;

use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

use self::winapi::windef::HWND;
use self::user32::{ FindWindowW, IsWindow };

pub struct Window {
    handle: HWND
}

impl Window {
    pub fn find(name: &str) -> Window {
        // todo: how optimal is that string conversion?
        let title = OsString::from(name).as_os_str().encode_wide().chain(Some(0).into_iter()).collect::<Vec<_>>();
        let handle = unsafe { FindWindowW(ptr::null_mut(), title.as_ptr()) };

        Window {
            handle: handle
        }
    }

    pub fn is_valid(&self) -> bool {
        unsafe { IsWindow(self.handle) > 0 }
    }
}
