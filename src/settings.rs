pub struct Settings {
    pub show_watermark: bool
}

impl Settings {
    pub fn load() -> Settings {
        Settings {
            show_watermark: true
        }
    }
}
