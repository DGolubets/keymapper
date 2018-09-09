#[macro_use]
extern crate log;
extern crate log4rs;
extern crate lazy_static;
extern crate config;
extern crate winapi;
extern crate user32;

mod errors;
mod settings;
mod profiles;
mod windows;
mod util;

use profiles::*;
use settings::Settings;
use windows::{ Window, Hook, HookAction, InputEvent, MouseEvent };
use std::collections::HashMap;
use std::time::Instant;
use std::cell::RefCell;

fn main() {
    log4rs::init_file("resources/log.toml", Default::default()).expect("Can't load logging config.");
    info!("Starting Keymapper..");
    let settings = Settings::load().expect("Can't load settings.");
    let profiles = profiles::load_profiles().expect("Can't load profiles.");
    let re_guard = util::ReentranceGuard::new();

    let last_mouse_wheel_time: RefCell<HashMap<bool, Instant>> = RefCell::new(HashMap::new());

    let _hook = Hook::set_input_hook(move |e| {
        let action = profiles
            .iter()
            .fold(HookAction::Forward, |result, profile| {
                // return early
                if result == HookAction::Block  {
                    return result;
                }

                // try another profile
                let should_process = move ||{
                    profile.triggers
                        .iter()
                        .flat_map(|trigger| {
                            match trigger {
                                &Trigger::Window { ref name } => Window::find(name)
                            }
                        })
                        .any(|w| w.is_foreground())
                };

                match e {
                    InputEvent::Keyboard(e) => {
                        for binding in &profile.bindings {
                            if let Binding::Key(binding) = binding {
                                let vcode_matched = binding.vcode == e.vk_code as u16;
                                let flags_matched = e.flags as u16 & binding.flags == binding.flags;
                                let key_matched = vcode_matched && flags_matched;

                                if key_matched && should_process() {
                                    if let Some(_) = re_guard.try_lock() {
                                        trace!("Profile \"{}\" blocked key: {:X} + {:X}", profile.name,  e.vk_code, e.flags);
                                        for key in &binding.keys {
                                            trace!("Sending key: {:X}", key.vcode);
                                            windows::send_input_key(key.vcode as i32, e.up());
                                        }
                                        return HookAction::Block;
                                    }
                                }
                            }
                        }
                    },
                    InputEvent::Mouse(MouseEvent::MouseWheel { delta, .. }) => {
                        for binding in &profile.bindings {
                            if let Binding::MouseWheel(binding) = binding {
                                let up = *delta > 0;
                                let matched = binding.up.iter().all(|v| *v == up);

                                if matched && should_process() {
                                    let now = Instant::now();

                                    let should_throttle = match last_mouse_wheel_time.borrow().get(&up) {
                                        Some(&last) => binding.throttle.iter().any(|d| d > &now.duration_since(last)),
                                        _ => false
                                    };

                                    if should_throttle {
                                        trace!("Profile \"{}\" throttle mouse wheel (up={})", profile.name, up);
                                        return HookAction::Block;
                                    }
                                    else {
                                        last_mouse_wheel_time.borrow_mut().insert(up, now);
                                    }
                                }
                            }
                        }
                    }
                }

                HookAction::Forward
            });
        action
    });

    windows::message_loop();

    info!("Shutting down Keymapper..");
}