pub struct WorldSettings {
    events: Vec<WorldEvent>
}

impl WorldSettings {
    pub fn new() -> WorldSettings {
        WorldSettings {
            events: Vec::new()
        }
    }

    pub fn add_event(&mut self, event: WorldEvent) {
        self.events.push(event);
    }

    pub fn flush_events(&mut self) -> Vec<WorldEvent> {
        self.events.drain(0..).collect()
    }
}

pub enum WorldEvent {
    PlayerSunkByCannon(String, String),
    PlayerSunkByRamming(String, String),
    PlayerSunkByAccident(String)
}
