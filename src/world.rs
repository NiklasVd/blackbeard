pub enum Weather {
    Sunny,
    Windy,
    Rainy,
    Stormy
}

pub struct World {
    pub weather: Weather
}

impl World {
    pub fn new() -> World {
        World {
            weather: Weather::Sunny
        }
    }
}
