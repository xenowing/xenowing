use abstract_environment::*;

use xw::{marv, stdio};

pub struct NativeEnvironment;

impl Environment<stdio::Stdout> for NativeEnvironment {
    fn cycles(&self) -> u64 {
        marv::cycles()
    }

    fn stdout(&self) -> stdio::Stdout {
        stdio::stdout()
    }

    fn time_seconds(&self) -> f64 {
        marv::cycles() as f64 * (1.0 / 100000000.0)
    }
}
