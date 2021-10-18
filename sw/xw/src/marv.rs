extern "C" {
    fn _cycles() -> u64;
}

pub fn cycles() -> u64 {
    unsafe {
        _cycles()
    }
}

pub fn sleep_cycles(c: u64) {
    let t = cycles();
    while cycles() - t < c {
        // Do nothing
    }
}
