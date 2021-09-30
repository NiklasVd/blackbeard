pub struct SimulationSettings {
    pub run: bool,
    pub curr_gen: u64,
    pub curr_frames: u64
}

impl SimulationSettings {
    pub fn new() -> SimulationSettings {
        SimulationSettings {
            run: true, curr_gen: 0, curr_frames: 0
        }
    }

    pub fn update(&mut self, gen: u64, frame: u64) {
        self.curr_gen = gen;
        self.curr_frames = frame;
    }
}
