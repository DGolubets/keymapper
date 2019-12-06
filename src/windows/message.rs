extern crate user32;
extern crate winapi;

use std::mem;
use std::ptr;

use user32::*;
use winapi::winuser::*;

pub fn message_loop() {
    unsafe {
        let mut msg: MSG = mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            DispatchMessageW(&msg);
        }
    }
}

pub fn post_quit_message() {
    unsafe { PostQuitMessage(0) };
}
