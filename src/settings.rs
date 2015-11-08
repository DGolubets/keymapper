extern crate config;

use std::path::Path;
use config::reader;
use config::types::Value;
use config::types::ScalarValue;

pub struct Settings {
    pub windows: Vec<String>
}

impl Settings {
    pub fn load() -> Result<Settings, &'static str> {
        let path = Path::new("src/application.conf");
        let cfg = reader::from_file(path);

        match cfg {
            Ok(cfg) => {
                let mut windows: Vec<String> = vec![];

                if let Some(&config::types::Value::Array(ref array)) = cfg.lookup("application.windows") {
                    for v in array {
                        if let &Value::Svalue(ScalarValue::Str(ref s)) = v {
                            windows.push(s.clone());
                        }
                    }
                }

                Ok(Settings {
                    windows: windows
                })
            },
            Err(e) => Err(e.desc)
        }
    }
}
