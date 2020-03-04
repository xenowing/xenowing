use kaze::*;
use rtl::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let mut file = File::create(&dest_path).unwrap();

    let c = Context::new();

    sim::generate(buster::generate(&c, "Buster1x2", 1, 2, 17, 1, 32, 2), &mut file)?;
    sim::generate(buster::generate(&c, "Buster2x2", 2, 2, 17, 1, 128, 4), &mut file)?;

    Ok(())
}
