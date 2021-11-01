use std::env;
use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tab.rs");
    let mut file = File::create(&dest_path).unwrap();

    const NUM_ENTRIES_BITS: usize = 12;
    const NUM_ENTRIES: usize = 1 << NUM_ENTRIES_BITS;
    writeln!(file, "pub const NUM_ENTRIES_BITS: usize = {};", NUM_ENTRIES_BITS)?;
    writeln!(file, "pub const NUM_ENTRIES: usize = {};", NUM_ENTRIES)?;
    writeln!(file, "pub static SIN_TAB: [u32; NUM_ENTRIES] = [")?;
    for i in 0..NUM_ENTRIES {
        let phase = i as f64 / NUM_ENTRIES as f64 * std::f64::consts::TAU;
        let entry = (phase.sin() as f32).to_bits();
        writeln!(file, "0x{:08x}, ", entry)?;
    }
    writeln!(file, "];")?;

    Ok(())
}
