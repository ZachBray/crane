use std::time::Duration;
use std::time::Instant;
use rand::prelude::*;
use rand::distributions::Exp;
use std::cmp::min;

pub struct RandomExpBackoffTimer {
    rng: ThreadRng,
    exp: Exp,
    due_time: Instant,
}

impl RandomExpBackoffTimer {
    pub fn new() -> Self {
        RandomExpBackoffTimer {
            rng: rand::thread_rng(),
            exp: Exp::new(1.0),
            due_time: Instant::now(),
        }
    }

    pub fn is_due(&self) -> bool {
        Instant::now() >= self.due_time
    }

    pub fn reset(&mut self) -> Instant {
        let x = self.exp.sample(&mut self.rng);
        let millis_to_wait = min(2000 + ((x * 3000.0).floor() as u64), 20000);
        let duration = Duration::from_millis(millis_to_wait);
        self.due_time = Instant::now() + duration;
        self.due_time
    }
}
