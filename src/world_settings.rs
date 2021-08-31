pub enum Weather {
    Sunny,
    Windy,
    Rainy,
    Stormy
}

pub struct WorldSettings {
    pub weather: Weather
}

impl WorldSettings {
    pub fn new() -> WorldSettings {
        WorldSettings {
            weather: Weather::Sunny
        }
    }
}
