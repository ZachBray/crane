use std::thread;
use std::time::Duration;
use rand::prelude::*;
use rand::distributions::Exp;
use std::cmp::min;

pub struct RandomSleeper {
    rng: ThreadRng,
    exp: Exp,
}

impl RandomSleeper {
    pub fn new() -> Self {
        RandomSleeper {
            rng: rand::thread_rng(),
            exp: Exp::new(1.0)
        }
    }

    pub fn sleep(&mut self) {
        let x = self.exp.sample(&mut self.rng);
        let millis_to_wait = min(2000 + ((x * 3000.0).floor() as u64), 20000);
        let duration = Duration::from_millis(millis_to_wait);
        println!("Waiting for {}ms before retrying.", millis_to_wait);
        thread::sleep(duration)
    }
}
