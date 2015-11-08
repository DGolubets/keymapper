extern crate config;

mod settings;
mod windows;

use settings::Settings;
use windows::Window;

fn main() {
    let settings = Settings::load().ok().expect("Can't load settings.");
    for w in settings.windows {
        let wnd = Window::find(&w);
        println!("{}", wnd.is_valid() );
    }
}
