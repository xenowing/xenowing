pub mod model_device;
mod modules {
    include!(concat!(env!("OUT_DIR"), "/modules.rs"));
}
pub mod sim_device;
