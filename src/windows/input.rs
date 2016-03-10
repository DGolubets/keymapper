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

pub struct Hook {
    hook_int: Rc<HookInternal>
}

impl Hook {
    pub fn set_keyboard_hook<H: Fn(&KbEvent) -> HookAction + 'static>(handler: H) -> Hook {
        HOOKS.with(|hooks| {
            let mut hooks = hooks.borrow_mut();
            let handle = unsafe { SetWindowsHookExW(13, Some(low_level_keyboard_proc), ptr::null_mut(), 0) };

            let hook_internal = HookInternal {
                handle: handle,
                handler: Box::new(handler)
            };

            let hook_internal = Rc::new(hook_internal);

            hooks.push(Rc::downgrade(&hook_internal));

            Hook {
                hook_int: hook_internal
            }
        })
    }
}

pub struct HookInternal {
    handle: HHOOK,
    handler: Box<Fn(&KbEvent) -> HookAction>
}

impl Drop for HookInternal {
    fn drop(&mut self) {
        debug!("Dropping hook..");
        unsafe {
            UnhookWindowsHookEx(self.handle);
        }
        cleanup_hooks();
    }
}

pub enum HookAction {
    Block,
    Forward
}

pub struct KbEvent {
    pub vk_code: i32,
    pub flags: i32
}

pub fn send_input_key(virtual_key: u16, up: bool) {
    unsafe {
        let mut input = INPUT { type_: INPUT_KEYBOARD, u: Default::default() };
        *input.ki_mut() = KEYBDINPUT {
            wVk: virtual_key,
            dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
            dwExtraInfo: 0,
            wScan: 0,
            time: 0
        };

        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
    }
}

/* PRIVATE */

unsafe extern "system" fn low_level_keyboard_proc(n_code: i32, w_param: u64, l_param: i64) -> i64 {
    if n_code == 0 {
        let kbhs = &*(l_param as *const KBDLLHOOKSTRUCT);
        let event = KbEvent {
            vk_code: kbhs.vkCode as i32,
            flags: kbhs.flags as i32,
        };
        let action = HOOKS.with(|hooks| {
            let hooks = hooks.borrow();
            for hook in hooks.iter() {
                if let Some(hook) = hook.upgrade() {
                    let handler = &hook.handler;
                    match handler(&event) {
                        HookAction::Block => return HookAction::Block,
                        HookAction::Forward => continue
                    }
                }
            }

            HookAction::Forward
        });

        match action {
            HookAction::Block => return 1,
            HookAction::Forward => ()
        }
    }

    CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param)
}

fn cleanup_hooks() {
    debug!("Cleaning hook list..");
    HOOKS.with(|hooks| {
        let mut hooks = hooks.borrow_mut();
        for i in (0..hooks.len()).rev() {
            if let None = hooks[i].upgrade() {
                debug!("Removing dead hook from the list..");
                hooks.swap_remove(i);
            }
        }
    });
}

thread_local!(static HOOKS: RefCell<Vec<Weak<HookInternal>>> = RefCell::new(vec![]));
