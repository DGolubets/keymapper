extern crate config;

use std::path::Path;
use config::reader;
use config::types::Value;
use config::types::ScalarValue;

pub struct Settings {
}

impl Settings {
    pub fn load() -> Result<Settings, &'static str> {
        let path = Path::new("resources/application.conf");
        let cfg = reader::from_file(path);

        match cfg {
            Ok(cfg) => {
                Ok(Settings {
                })
            },
            Err(e) => Err(e.desc)
        }
    }
}
