pub struct Settings {
    pub show_watermark: bool,
    pub name: String
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            show_watermark: true, name: String::new()
        }
    }

    pub fn load() -> Settings {
        todo!()
    }

    pub fn set_name(&mut self, name: String) -> bool {
        if name.len() >= 4 && name.len() <= 21 && !name.contains(' ') {
            self.name = name;
            true
        } else {
            println!("Invalid name. Must be between 4 and 21 characters, contain only letters and no spaces.");
            false
        }
    }
}
