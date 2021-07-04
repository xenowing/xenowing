use kaze::*;
use rtl::peek_buffer::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let mut file = File::create(&dest_path).unwrap();

    let c = Context::new();

    let peek_buffer = PeekBuffer::new("peek_buffer", 32, &c);
    sim::generate(peek_buffer.m, sim::GenerationOptions::default(), &mut file)?;
    sim::generate(peek_buffer.m, sim::GenerationOptions {
        override_module_name: Some("TracingPeekBuffer".into()),
        tracing: true,
        ..sim::GenerationOptions::default()
    }, file)
}
