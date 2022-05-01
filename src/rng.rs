extern crate rand;
use rand::{OsRng, Rng};

pub struct RandomBytes {
    rng: OsRng
}

impl RandomBytes {
    pub fn new() -> RandomBytes {
        RandomBytes {
            rng: OsRng::new().expect("Error opening new random number generator")
        }
    }
    pub fn next(&mut self) -> u8 {
        self.rng.gen()
    }
}