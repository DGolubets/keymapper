pub struct Settings {}

impl Settings {
    pub fn load() -> Result<Settings, &'static str> {
        Ok(Settings {})
    }
}
