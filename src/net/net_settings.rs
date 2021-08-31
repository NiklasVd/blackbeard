pub struct NetSettings {
    pub max_players: usize
}

impl NetSettings {
    fn new(max_players: usize) -> NetSettings {
        NetSettings {
            max_players
        }
    }
}

impl Default for NetSettings {
    fn default() -> Self {
        NetSettings::new(4)
    }
}
