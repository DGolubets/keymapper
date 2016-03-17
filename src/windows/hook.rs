extern crate log;
extern crate winapi;
extern crate user32;

use std::ptr;
use std::cell::RefCell;
use std::thread::LocalKey;

use util::*;

use self::winapi::windef::*;
use self::winapi::winuser::*;
use self::user32::*;

pub struct Hook {
    _hook_int: WeakCollectionItem<HookInternal>
}

impl Hook {
    pub fn set_keyboard_hook<H: Fn(&KbEvent) -> HookAction + 'static>(handler: H) -> Hook {
        KB_HOOKS.with(|hooks|{
            let mut hooks = hooks.borrow_mut();
            let handle = unsafe { SetWindowsHookExW(13, Some(keyboard_hook_proc), ptr::null_mut(), 0) };

            let handler = move |_, _, l_param| {
                let kbhs = unsafe { &*(l_param as *const KBDLLHOOKSTRUCT) };
                let event = KbEvent {
                    vk_code: kbhs.vkCode as i32,
                    flags: kbhs.flags as i32,
                };
                handler(&event)
            };

            let hook = HookInternal {
                handle: handle,
                handler: Box::new(handler)
            };

            Hook {
                _hook_int: hooks.push(hook)
            }
        })
    }
}

pub struct HookInternal {
    handle: HHOOK,
    handler: Box<Fn(i32, u64, i64) -> HookAction>
}

impl Drop for HookInternal {
    fn drop(&mut self) {
        debug!("Dropping hook..");
        unsafe {
            UnhookWindowsHookEx(self.handle);
        }
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

/* PRIVATE */
unsafe extern "system" fn keyboard_hook_proc(n_code: i32, w_param: u64, l_param: i64) -> i64 {
    base_hook_proc(&KB_HOOKS, n_code, w_param, l_param)
}

unsafe fn base_hook_proc(local: &'static LocalKey<RefCell<WeakCollection<HookInternal>>>, n_code: i32, w_param: u64, l_param: i64) -> i64 {
    // If nCode is less than zero, the hook procedure must pass the message to the CallNextHookEx function without further processing
    // and should return the value returned by CallNextHookEx.
    if n_code == 0 {
        let action = local.with(|hooks| {
            let hooks = hooks.borrow_mut();

            for item in hooks.into_iter() {
                let action = (&item.handler)(n_code, w_param, l_param);

                match action {
                    HookAction::Block => return HookAction::Block,
                    _ => {}
                }
            }

            HookAction::Forward
        });

        match action {
            HookAction::Block => return 1,
            _ => {}
        }
    }

    CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param)
}

thread_local!(static KB_HOOKS: RefCell<WeakCollection<HookInternal>> = RefCell::new(WeakCollection::new()));
