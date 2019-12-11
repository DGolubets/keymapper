use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;
use std::thread::LocalKey;

use winapi::shared::minwindef::HIWORD;
use winapi::shared::windef::*;
use winapi::um::winuser::*;

use crate::util::*;

pub struct Hook {
    _hook_int: Vec<WeakCollectionItem<HookInternal>>,
}

impl Hook {
    pub fn set_input_hook<H: Fn(&InputEvent) -> HookAction + 'static>(handler: H) -> Hook {
        let handler = Rc::new(handler);

        let handler_for_kb = handler.clone();
        let keyboard_hook = Hook::set_keyboard_hook(move |e| {
            let e = InputEvent::Keyboard(*e);
            handler_for_kb(&e)
        });

        let handler_for_mouse = handler.clone();
        let mouse_hook = Hook::set_mouse_hook(move |e| {
            let e = InputEvent::Mouse(*e);
            handler_for_mouse(&e)
        });

        let mut hooks = Vec::new();
        hooks.extend(keyboard_hook._hook_int.into_iter());
        hooks.extend(mouse_hook._hook_int.into_iter());

        Hook { _hook_int: hooks }
    }

    pub fn set_keyboard_hook<H: Fn(&KeyboardEvent) -> HookAction + 'static>(handler: H) -> Hook {
        KEYBOARD_HOOKS.with(|hooks| {
            let mut hooks = hooks.borrow_mut();
            let handle = unsafe {
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), ptr::null_mut(), 0)
            };

            let handler = move |_, _, l_param| {
                let kbhs = unsafe { &*(l_param as *const KBDLLHOOKSTRUCT) };
                let event = KeyboardEvent {
                    vk_code: kbhs.vkCode as i32,
                    flags: kbhs.flags as i32,
                };
                handler(&event)
            };

            let hook = HookInternal {
                handle,
                handler: Box::new(handler),
            };

            Hook {
                _hook_int: vec![hooks.push(hook)],
            }
        })
    }

    pub fn set_mouse_hook<H: Fn(&MouseEvent) -> HookAction + 'static>(handler: H) -> Hook {
        MOUSE_HOOKS.with(|hooks| {
            let mut hooks = hooks.borrow_mut();
            let handle = unsafe {
                SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), ptr::null_mut(), 0)
            };

            let handler = move |_: i32, w_param: usize, l_param: isize| {
                let mshs = unsafe { &*(l_param as *const MSLLHOOKSTRUCT) };

                match w_param as u32 {
                    WM_MOUSEWHEEL => {
                        let event = MouseEvent::MouseWheel {
                            x: mshs.pt.x,
                            y: mshs.pt.y,
                            delta: HIWORD(mshs.mouseData) as i16,
                        };
                        handler(&event)
                    }
                    _ => HookAction::Forward,
                }
            };

            let hook = HookInternal {
                handle,
                handler: Box::new(handler),
            };

            Hook {
                _hook_int: vec![hooks.push(hook)],
            }
        })
    }
}

pub struct HookInternal {
    handle: HHOOK,
    handler: Box<dyn Fn(i32, usize, isize) -> HookAction>,
}

impl Drop for HookInternal {
    fn drop(&mut self) {
        log::debug!("Dropping hook..");
        unsafe {
            UnhookWindowsHookEx(self.handle);
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum HookAction {
    Block,
    Forward,
}

#[derive(Debug)]
pub enum InputEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
}

#[derive(Debug, Copy, Clone)]
pub struct KeyboardEvent {
    pub vk_code: i32,
    pub flags: i32,
}

impl KeyboardEvent {
    pub fn alt(&self) -> bool {
        const LLKHF_ALTDOWN: i32 = 0x20;
        self.flags & LLKHF_ALTDOWN > 0
    }

    pub fn up(&self) -> bool {
        const LLKHF_UP: i32 = 0x80;
        self.flags & LLKHF_UP > 0
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseEvent {
    MouseWheel { x: i32, y: i32, delta: i16 },
}

/* PRIVATE */
unsafe extern "system" fn keyboard_hook_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    base_hook_proc(&KEYBOARD_HOOKS, n_code, w_param, l_param)
}

unsafe extern "system" fn mouse_hook_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    base_hook_proc(&MOUSE_HOOKS, n_code, w_param, l_param)
}

unsafe fn base_hook_proc(
    local: &'static LocalKey<RefCell<WeakCollection<HookInternal>>>,
    n_code: i32,
    w_param: usize,
    l_param: isize,
) -> isize {
    // If nCode is less than zero, the hook procedure must pass the message to the CallNextHookEx function without further processing
    // and should return the value returned by CallNextHookEx.
    if n_code == 0 {
        let action = local.with(|hooks| {
            let hooks = hooks.borrow();

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

thread_local!(static KEYBOARD_HOOKS: RefCell<WeakCollection<HookInternal>> = RefCell::new(WeakCollection::new()));
thread_local!(static MOUSE_HOOKS: RefCell<WeakCollection<HookInternal>> = RefCell::new(WeakCollection::new()));
