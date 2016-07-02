extern crate log;
extern crate winapi;
extern crate user32;

use std::ptr;
use std::mem;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

use self::winapi::windef::*;
use self::winapi::winuser::*;
use self::user32::*;

pub fn send_input_key(virtual_key: i32, up: bool) {
    unsafe {
        let mut input = INPUT { type_: INPUT_KEYBOARD, u: Default::default() };
        *input.ki_mut() = KEYBDINPUT {
            wVk: virtual_key as u16,
            dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
            dwExtraInfo: 0,
            wScan: 0,
            time: 0
        };

        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
    }
}
