use std::mem;
use std::ptr;

use winapi::um::winuser::*;

pub fn message_loop() {
    unsafe {
        let mut msg: MSG = mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            DispatchMessageW(&msg);
        }
    }
}

#[allow(dead_code)]
pub fn post_quit_message() {
    unsafe { PostQuitMessage(0) };
}
