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
use windows::{ Window, Hook, HookAction };

fn main() {
    log4rs::init_file("resources/log.toml", Default::default()).expect("Can't load logging config.");
    info!("Starting Keymapper..");
    let settings = Settings::load().expect("Can't load settings.");
    let profiles = profiles::load_profiles().expect("Can't load profiles.");
    let re_guard = util::ReentranceGuard::new();

    let _hook = Hook::set_keyboard_hook(move |e| {
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
                        .filter_map(|trigger| {
                            match trigger {
                                &Trigger::Window { ref name } => Window::find(name)
                            }
                        })
                        .any(|w| w.is_foreground())
                };

                for binding in &profile.bindings {
                    let vcode_matched = binding.vcode == e.vk_code as u16;
                    let flags_matched = e.flags as u16 & binding.flags == binding.flags;
                    let key_matched = vcode_matched && flags_matched;

                    if key_matched && should_process() {
                        if let Some(_) = re_guard.try_lock() {
                            trace!("Profile \"{}\" blocked key: {:X} + {:X}", profile.name,  e.vk_code, e.flags);
                            for key in &binding.keys {
                                trace!("Sending key: {:X}", key.vcode);
                                windows::send_input_key(key.vcode as i32, e.up());
                                //windows::send_input_key(key.vcode as i32, true);
                            }
                            return HookAction::Block;
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
