extern crate log;
extern crate winapi;
extern crate user32;

use std::ptr;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

use self::winapi::windef::*;
use self::winapi::winuser::*;
use self::user32::*;

pub struct Hook {
    _hook_int: Rc<HookInternal>
}

impl Hook {
    pub fn set_keyboard_hook<H: Fn(&KbEvent) -> HookAction + 'static>(handler: H) -> Hook {
        REGISTRY.with(|registry|{
            let mut registry = registry.borrow_mut();
            let handle = unsafe { SetWindowsHookExW(13, Some(keyboard_hook_proc), ptr::null_mut(), 0) };

            let hook = registry.register(HookType::Keyboard, handle, move |_, _, l_param| {
                let kbhs = unsafe { &*(l_param as *const KBDLLHOOKSTRUCT) };
                let event = KbEvent {
                    vk_code: kbhs.vkCode as i32,
                    flags: kbhs.flags as i32,
                };
                handler(&event)
            });

            Hook {
                _hook_int: hook
            }
        })
    }

    pub fn set_shell_hook<H: Fn(u64) -> HookAction + 'static>(handler: H) -> Hook {
        REGISTRY.with(|registry|{
            let mut registry = registry.borrow_mut();
            let handle = unsafe { SetWindowsHookExW(10, Some(shell_hook_proc), ptr::null_mut(), 0) };

            let hook = registry.register(HookType::Shell, handle, move |_, w_param, l_param| {
                handler(w_param)
            });

            Hook {
                _hook_int: hook
            }
        })
    }
}

pub struct HookInternal {
    id: i32,
    tpe: HookType,
    handle: HHOOK,
    handler: Box<Fn(i32, u64, i64) -> HookAction>
}

impl Drop for HookInternal {
    fn drop(&mut self) {
        debug!("Dropping hook..");
        unsafe {
            UnhookWindowsHookEx(self.handle);
        }

        REGISTRY.with(|registry|{
            let mut registry = registry.borrow_mut();
            registry.unregister(self);
        });
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

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum HookType {
    Keyboard,
    Shell
}

/* PRIVATE */
unsafe extern "system" fn keyboard_hook_proc(n_code: i32, w_param: u64, l_param: i64) -> i64 {
    base_hook_proc(HookType::Keyboard, n_code, w_param, l_param)
}

unsafe extern "system" fn shell_hook_proc(n_code: i32, w_param: u64, l_param: i64) -> i64 {
    base_hook_proc(HookType::Shell, n_code, w_param, l_param)
}

unsafe fn base_hook_proc(tpe: HookType, n_code: i32, w_param: u64, l_param: i64) -> i64 {
    // If nCode is less than zero, the hook procedure must pass the message to the CallNextHookEx function without further processing
    // and should return the value returned by CallNextHookEx.
    if n_code == 0 {
        let action = REGISTRY.with(|registry|{
            let registry = registry.borrow();

            for hook in registry.by_type(tpe) {
                let action = match hook.upgrade() {
                    Some(hook) => {
                        let handler = &hook.as_ref().handler;
                        handler(n_code, w_param, l_param)
                    },
                    None => HookAction::Forward
                };

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

struct HookRegistry {
    last_id: i32,
    table_by_id: HashMap<i32, Weak<HookInternal>>,
    table_by_type: HashMap<HookType, HashMap<i32, Weak<HookInternal>>>
}

impl HookRegistry {
    fn new() -> HookRegistry {
        HookRegistry {
            last_id: 0,
            table_by_id: HashMap::new(),
            table_by_type: HashMap::new()
        }
    }

    fn register<H: Fn(i32, u64, i64) -> HookAction + 'static>(&mut self, tpe: HookType, handle: HHOOK, handler: H) -> Rc<HookInternal> {
        self.last_id += 1;
        let hook = HookInternal {
            id: self.last_id,
            tpe: tpe,
            handle: handle,
            handler: Box::new(handler)
        };

        let hook = Rc::new(hook);

        self.table_by_id.insert(hook.id, Rc::downgrade(&hook));

        let mut table_by_id_for_type = self.table_by_type.entry(hook.tpe).or_insert(HashMap::new());
        table_by_id_for_type.insert(hook.id, Rc::downgrade(&hook));

        debug!("Registered hook with id = {}. Total = {}.", hook.id, self.table_by_id.len());
        hook
    }

    fn unregister(&mut self, hook: &HookInternal){
        self.table_by_id.remove(&hook.id);
        if let Some(t) = self.table_by_type.get_mut(&hook.tpe) {
            t.remove(&hook.id);
        };
        debug!("Unregistered hook with id = {}. Total = {}.", hook.id, self.table_by_id.len());
    }

    fn by_type<'a>(&'a self, tpe: HookType) -> HookIterator<'a> {
        HookIterator {
            values: self.table_by_type.get(&tpe).map(|t| t.values())
        }
    }
}

struct HookIterator<'a> {
    values: Option<::std::collections::hash_map::Values<'a, i32, Weak<HookInternal>>>
}

impl<'a> Iterator for HookIterator<'a> {
    type Item = &'a Weak<HookInternal>;

    fn next(&mut self) -> Option<&'a Weak<HookInternal>> {
        match self.values {
            Some(ref mut v) => v.next(),
            None => None
        }
    }
}

thread_local!(static REGISTRY: RefCell<HookRegistry> = RefCell::new(HookRegistry::new()));
