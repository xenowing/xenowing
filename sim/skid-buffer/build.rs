use kaze::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let file = File::create(&dest_path).unwrap();

    let c = Context::new();

    skid_buffer::generate(&c, "SkidBuffer", 32);

    let m = c.module("SkidBufferPipe");

    // TODO: Multiple stages
    let stage = m.instance("skid_buffer", "SkidBuffer");

    //  Input
    m.output("in_ready", stage.output("in_ready"));
    stage.drive_input("in_data", m.input("in_data", 32));
    stage.drive_input("in_data_valid", m.input("in_data_valid", 1));

    //  Output
    stage.drive_input("out_ready", m.input("out_ready", 1));
    m.output("out_data", stage.output("out_data"));
    m.output("out_data_valid", stage.output("out_data_valid"));

    // TODO: lol
    let _ = m.input("expected", 32);

    sim::generate(m, sim::GenerationOptions {
        tracing: true,
        ..sim::GenerationOptions::default()
    }, file)
}

mod skid_buffer {
    use kaze::*;
    use rtl::helpers::*;

    pub fn generate<'a, S: Into<String>>(c: &'a Context<'a>, mod_name: S, data_bit_width: u32) -> &Module<'a> {
        let mod_name = mod_name.into();

        let m = c.module(&mod_name);

        let in_data = m.reg("in_data_reg", data_bit_width);
        //let in_data_valid = reg_next_with_default("in_data_valid_reg", m.input("in_data_valid", 1), false, m);
        let in_data_valid = m.reg("in_data_valid_reg", 1);
        in_data_valid.default_value(false);

        let out_ready = m.input("out_ready", 1);

        let buffer_data = m.reg("buffer_data", data_bit_width);

        let buffer_occupied = m.reg("buffer_occupied", 1);
        buffer_occupied.default_value(false);

        in_data.drive_next(if_(buffer_occupied.value, {
            in_data.value
        }).else_({
            m.input("in_data", data_bit_width)
        }));
        in_data_valid.drive_next(if_(buffer_occupied.value, {
            in_data_valid.value
        }).else_({
            m.input("in_data_valid", 1) | (in_data_valid.value & !out_ready)
        }));

        buffer_data.drive_next(if_(!buffer_occupied.value & in_data_valid.value & !out_ready, {
            in_data.value
        }).else_({
            buffer_data.value
        }));

        buffer_occupied.drive_next(if_(out_ready, {
            m.low()
        }).else_({
            buffer_occupied.value | in_data_valid.value
        }));

        let in_ready = !in_data_valid.value | out_ready;

        m.output("in_ready", reg_next_with_default("in_ready_reg", in_ready, true, m));

        m.output("out_data", buffer_occupied.value.mux(buffer_data.value, in_data.value));
        m.output("out_data_valid", in_data_valid.value | buffer_occupied.value);

        m
    }
}
