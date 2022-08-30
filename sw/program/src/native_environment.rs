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
}
