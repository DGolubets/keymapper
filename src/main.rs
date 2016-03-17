#[macro_use]
extern crate log;
extern crate log4rs;
extern crate lazy_static;
extern crate config;
extern crate winapi;
extern crate user32;

mod settings;
mod windows;
mod util;

use winapi::winuser::*;

use settings::Settings;
use windows::{ Window, Hook, HookAction };

fn main() {
    log4rs::init_file("resources/log.toml", Default::default()).expect("Can't load logging config.");
    info!("Starting Keymapper..");
    let settings = Settings::load().expect("Can't load settings.");

    let should_process = move ||{
        settings.windows.iter()
            .filter_map(|w| Window::find(w))
            .any(|w| w.is_foreground())
    };

    let _hook = Hook::set_keyboard_hook(move |e| {
        let action = match e.vk_code {
            VK_LWIN if should_process() => HookAction::Block,
            VK_TAB if e.flags & 0x00000020 > 0 && should_process() => HookAction::Block, // Alt-Tab
            VK_CAPITAL if should_process() => HookAction::Block,
            _ => HookAction::Forward
        };

        trace!("Pressed {}. Action: {}.", e.vk_code, match action {
             HookAction::Forward => "passed",
             HookAction::Block => "blocked"
        });
        action
    });

    windows::message_loop();

    info!("Shutting down Keymapper..");
}
