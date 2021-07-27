use crate::V2;

pub enum Weather {
    Sunny,
    Windy,
    Rainy,
    Stormy
}

pub struct World {
    pub wind: V2,
    pub weather: Weather
}

impl World {
    pub fn new() -> World {
        World {
            wind: V2::zero(),
            weather: Weather::Sunny
        }
    }
}
