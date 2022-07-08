mod errors;
mod profiles;
mod settings;
mod util;
mod windows;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use futures::future;
use futures::future::AbortHandle;
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::profiles::*;
use crate::settings::Settings;
use crate::windows::{Hook, HookAction, InputEvent, KeyboardEvent, MouseEvent, Window};

fn main() {
    log4rs::init_file("resources/log.toml", Default::default())
        .expect("Can't load logging config.");
    log::info!("Starting Keymapper..");
    let _settings = Settings::load().expect("Can't load settings.");
    let profiles = profiles::load_profiles().expect("Can't load profiles.");
    let profiles = Arc::new(profiles);

    let last_mouse_wheel_time: RefCell<HashMap<bool, Instant>> = RefCell::new(HashMap::new());

    let rt = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime.");

    let (mut tx, rx) = mpsc::channel(100);

    rt.spawn(async { process_event_loop(rx).await });

    let _hook = Hook::set_input_hook(move |e| {
        let action = profiles.iter().enumerate().fold(
            HookAction::Forward,
            |result, (profile_index, profile)| {
                // return early
                if result == HookAction::Block {
                    return result;
                }

                // try another profile
                let should_process = move || {
                    profile
                        .triggers
                        .iter()
                        .flat_map(|trigger| match trigger {
                            &Trigger::Window { ref name } => Window::find(name),
                        })
                        .any(|w| w.is_foreground())
                };

                match e {
                    InputEvent::Keyboard(e) => {
                        for (binding_index, binding) in profile.bindings.iter().enumerate() {
                            if let Binding::Key(binding) = binding {
                                if is_match(&binding, &e) && !e.syntetic() && should_process() {
                                    log::trace!(
                                        "Profile \"{}\" blocked key: {:X} + {:X}",
                                        profile.name,
                                        e.vk_code,
                                        e.flags
                                    );

                                    if !binding.keys.is_empty() {
                                        let send_result = tx.try_send(MatchedEvent {
                                            profiles: profiles.clone(),
                                            profile_index,
                                            binding_index,
                                            up: e.up(),
                                        });

                                        if let Err(_) = send_result {
                                            log::error!(
                                                "Failed to add key macro to processing queue."
                                            );
                                        }
                                    }

                                    return HookAction::Block;
                                }
                            }
                        }
                    }
                    InputEvent::Mouse(MouseEvent::MouseWheel { delta, .. }) => {
                        for binding in &profile.bindings {
                            if let Binding::MouseWheel(binding) = binding {
                                let up = *delta > 0;
                                let matched = binding.up.iter().all(|v| *v == up);

                                if matched && should_process() {
                                    let now = Instant::now();

                                    let should_throttle =
                                        match last_mouse_wheel_time.borrow().get(&up) {
                                            Some(&last) => binding
                                                .throttle
                                                .iter()
                                                .any(|d| d > &now.duration_since(last)),
                                            _ => false,
                                        };

                                    if should_throttle {
                                        log::trace!(
                                            "Profile \"{}\" throttle mouse wheel (up={})",
                                            profile.name,
                                            up
                                        );
                                        return HookAction::Block;
                                    } else {
                                        last_mouse_wheel_time.borrow_mut().insert(up, now);
                                    }
                                }
                            }
                        }
                    }
                }

                HookAction::Forward
            },
        );
        action
    });

    windows::message_loop();

    log::info!("Shutting down Keymapper..");
}

struct MatchedEvent {
    profiles: Arc<Vec<Profile>>,
    profile_index: usize,
    binding_index: usize,
    up: bool,
}

async fn process_event_loop(mut rx: mpsc::Receiver<MatchedEvent>) {
    let mut event_handle: Option<AbortHandle> = None;
    loop {
        if let Some(event) = rx.recv().await {
            if let Some(event_handle) = event_handle {
                // cancel in progress key event
                event_handle.abort();
            }

            // prepare processing new event
            let event_future = process_event(event);

            // make it abortable
            let (event_future, abort_handle) = future::abortable(event_future);

            // save the abort handle
            event_handle = Some(abort_handle);

            // start processing detached from this async loop
            tokio::spawn(event_future);
        }
    }
}

async fn process_event(e: MatchedEvent) {
    if let Some(profile) = e.profiles.get(e.profile_index) {
        if let Some(Binding::Key(binding)) = profile.bindings.get(e.binding_index) {
            for key in &binding.keys {
                if let Some(duration) = key.delay {
                    log::trace!("Delaying for {:?}", duration);
                    sleep(duration).await;
                }
                let up = key.up.unwrap_or(e.up);
                log::trace!("Sending key: {:X}, up = {:?}", key.vk_code, up);
                windows::send_input_key(key.vk_code as i32, up);
            }
        }
    }
}

fn is_match(binding: &KeyBinding, e: &KeyboardEvent) -> bool {
    let vcode_matched = binding.vk_code == e.vk_code;
    let up_matched = binding.up.into_iter().all(|v| v == e.up());
    let alt_matched = binding.alt.into_iter().all(|v| v == e.alt());
    vcode_matched && up_matched && alt_matched
}
