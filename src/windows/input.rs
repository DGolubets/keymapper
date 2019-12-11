use std::mem;

use winapi::um::winuser::*;

pub fn send_input_key(virtual_key: i32, up: bool) {
    unsafe {
        let mut input = INPUT {
            type_: INPUT_KEYBOARD,
            u: std::mem::zeroed(),
        };
        *input.u.ki_mut() = KEYBDINPUT {
            wVk: virtual_key as u16,
            dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
            dwExtraInfo: 0,
            wScan: 0,
            time: 0,
        };

        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
    }
}
