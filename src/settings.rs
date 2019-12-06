extern crate config;

use config::reader;
use std::path::Path;

pub struct Settings {}

impl Settings {
    pub fn load() -> Result<Settings, &'static str> {
        let path = Path::new("resources/application.conf");
        let cfg = reader::from_file(path);

        match cfg {
            Ok(_cfg) => Ok(Settings {}),
            Err(e) => Err(e.desc),
        }
    }
}
