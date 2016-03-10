extern crate winapi;
extern crate user32;

use std::ptr;
use std::mem;

use winapi::winuser::*;
use user32::*;

pub fn message_loop() {
    unsafe {
        let mut msg: MSG = mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0
        {
            DispatchMessageW(&msg);
        }
    }
}

pub fn post_quit_message() {
    unsafe{ PostQuitMessage(0) };
}
