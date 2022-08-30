use ::env::*;

use core::fmt::{self, Result};

use std::io::{self, Write};

pub struct ModelEnvironment;

impl Environment<Stdout> for ModelEnvironment {
    fn cycles(&self) -> u64 {
        0 // TODO!
    }

    fn stdout(&self) -> Stdout {
        Stdout
    }
}

pub struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        write!(io::stdout(), "{}", s).expect("Failed to write to stdout");

        Ok(())
    }
}
