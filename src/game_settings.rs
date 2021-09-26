use binary_stream::{BinaryStream, Serializable};

#[derive(Debug, Clone, Copy)]
pub struct GameSettings {
    pub mode: GameMode,
    pub weather: Weather
}

impl GameSettings {
    pub fn new(mode: GameMode, weather: Weather) -> GameSettings {
        GameSettings {
            mode, weather
        }
    }

    pub fn default() -> GameSettings {
        Self::new(GameMode::Raid(500), Weather::Sunny)
    }
}

impl Serializable for GameSettings {
    fn to_stream(&self, stream: &mut BinaryStream) {
        self.mode.to_stream(stream);
        self.weather.to_stream(stream);
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        let mode = GameMode::from_stream(stream);
        let weather = Weather::from_stream(stream);
        GameSettings::new(mode, weather)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameMode {
    Raid(u32), // First player to collect required amount of escudos wins
    Deathmatch(u16) // First player with required amount of sinkings wins
}

impl Serializable for GameMode {
    fn to_stream(&self, stream: &mut BinaryStream) {
        match self {
            GameMode::Raid(escudos_goal) => {
                stream.write_buffer_single(0).unwrap();
                stream.write_u32(*escudos_goal).unwrap();
            },
            GameMode::Deathmatch(sinkings_goal) => {
                stream.write_buffer_single(1).unwrap();
                stream.write_u16(*sinkings_goal).unwrap();
            }
        }
    }

    fn from_stream(stream: &mut BinaryStream) -> Self {
        match stream.read_buffer_single().unwrap() {
            0 => GameMode::Raid(stream.read_u32().unwrap()),
            1 => GameMode::Deathmatch(stream.read_u16().unwrap()),
            n @ _ => panic!("Index {} is not assigned to any weather type", n)
        }
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
