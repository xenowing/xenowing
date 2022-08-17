use kaze::*;
use rtl::buster_mig_ui_bridge::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let mut file = File::create(&dest_path).unwrap();

    let c = Context::new();

    let buster_mig_ui_bridge = BusterMigUiBridge::new("buster_mig_ui_bridge", 32, 8, &c);
    sim::generate(buster_mig_ui_bridge.m, sim::GenerationOptions::default(), &mut file)?;
    sim::generate(buster_mig_ui_bridge.m, sim::GenerationOptions {
        override_module_name: Some("TracingBusterMigUiBridge".into()),
        tracing: true,
        ..sim::GenerationOptions::default()
    }, file)
}
