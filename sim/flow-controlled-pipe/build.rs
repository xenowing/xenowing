use kaze::*;
use rtl::*;

use std::env;
use std::fs::File;
use std::io::Result;
use std::path::Path;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("modules.rs");
    let file = File::create(&dest_path).unwrap();

    let c = Context::new();

    let num_inner_pipe_stages = 4;
    let data_bit_width = 32;

    // Inner pipe
    let inner_pipe_mod_name = "InnerPipe";
    generate_inner_pipe(&c, inner_pipe_mod_name, num_inner_pipe_stages, data_bit_width);

    // Pipe
    let mut pipe = flow_controlled_pipe::FlowControlledPipe::new(&c, "Pipe", inner_pipe_mod_name, num_inner_pipe_stages);
    pipe.input("a", data_bit_width);
    pipe.output("b", data_bit_width);
    pipe.output("c", data_bit_width);
    pipe.aux_input("d", 1);
    pipe.aux_output("e");

    sim::generate(pipe.module, sim::GenerationOptions {
        tracing: true,
        ..Default::default()
    }, file)
}

fn generate_inner_pipe<'a, S: Into<String>>(c: &'a Context<'a>, mod_name: S, num_pipe_stages: u32, data_bit_width: u32) -> &Module<'a> {
    let m = c.module(mod_name);

    // Pipeline
    let mut a = m.input("in_a", data_bit_width);
    let mut valid = m.input("in_valid", 1);

    for i in 0..num_pipe_stages {
        a = a.reg_next(format!("stage{}_a", i));
        valid = valid.reg_next_with_default(format!("stage{}_valid", i), false);
    }

    m.output("out_b", a);
    m.output("out_c", !a);
    m.output("out_valid", valid);

    // Aux
    m.output("e", !m.input("d", 1));

    m
}
