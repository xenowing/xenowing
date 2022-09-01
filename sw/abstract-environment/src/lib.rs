#![no_std]

use core::fmt::Write;

pub trait Environment<W: Write> {
    fn cycles(&self) -> u64;
    fn stdout(&self) -> W;
    fn time_seconds(&self) -> f64;
}
