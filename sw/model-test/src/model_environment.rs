use abstract_environment::*;

use core::fmt::{self, Result};

use std::io::{self, Write};
use std::time::Instant;

pub struct ModelEnvironment {
    start_time: Instant,
}

impl ModelEnvironment {
    pub fn new() -> ModelEnvironment {
        ModelEnvironment {
            start_time: Instant::now(),
        }
    }
}

impl Environment<Stdout> for ModelEnvironment {
    fn cycles(&self) -> u64 {
        0 // TODO!
    }

    fn stdout(&self) -> Stdout {
        Stdout
    }

    fn time_seconds(&self) -> f64 {
        Instant::now().duration_since(self.start_time).as_secs_f64()
    }
}

pub struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        write!(io::stdout(), "{}", s).expect("Failed to write to stdout");

        Ok(())
    }
}
