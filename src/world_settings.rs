use binary_stream::Serializable;

pub struct WorldSettings {
    pub weather: Weather,
    events: Vec<WorldEvent>
}

impl WorldSettings {
    pub fn new() -> WorldSettings {
        WorldSettings {
            weather: Weather::Sunny, events: Vec::new()
        }
    }

    pub fn add_event(&mut self, event: WorldEvent) {
        self.events.push(event);
    }

    pub fn flush_events(&mut self) -> Vec<WorldEvent> {
        self.events.drain(0..).collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Weather {
    Sunny = 0,
    Windy = 1,
    Rainy = 2,
    Stormy = 3 // Windy | Rainy
}

impl Serializable for Weather {
    fn to_stream(&self, stream: &mut binary_stream::BinaryStream) {
        stream.write_buffer_single(match self {
            Weather::Sunny => 0,
            Weather::Windy => 1,
            Weather::Rainy => 2,
            Weather::Stormy => 3,
        }).unwrap();
    }

    fn from_stream(stream: &mut binary_stream::BinaryStream) -> Self {
        match stream.read_buffer_single().unwrap() {
            0 => Weather::Sunny,
            1 => Weather::Windy,
            2 => Weather::Rainy,
            3 => Weather::Stormy,
            n @ _ => panic!("Index {} is not assigned to any weather type", n)
        }
    }
}

pub enum WorldEvent {
    PlayerSunkByCannon(String, String),
    PlayerSunkByRamming(String, String),
    PlayerSunkByAccident(String)
}
