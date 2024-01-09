pub struct FpsCounter {
    t0: f64,
    t1: f64,
    delta: f64,
}
impl FpsCounter {
    pub fn new() -> Self {
        Self {
            t1: instant::now(),
            t0: instant::now(),
            delta: 1.0 / 60.0,
        }
    }

    pub fn update(&mut self) -> f64 {
        self.t1 = instant::now();
        self.delta = (self.t1 - self.t0) / 1000.0;
        self.t0 = self.t1;

        self.delta
    }

    pub fn fps(&self) -> f64 {
        if self.delta == 0.0 {
            -1.0
        } else {
            1.0 / self.delta
        }
    }
}