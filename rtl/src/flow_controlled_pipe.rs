use crate::fifo;
use crate::peek_buffer;

use kaze::*;

/// Generates a module that wraps an existing inner pipeline module, and adds credit-based flow control logic.
///
/// # Assumptions
///
/// The inner pipeline module (specified by `inner_pipe_mod_name`) is assumed to be a straightforward pipeline with `num_inner_pipe_stages` stages.
/// Each stage starts by registering its inputs, followed by combinational logic.
///
/// Inputs to the pipe are assumed to be prefixed with `in_`, and likewise, outputs are assumed to be prefixed with `out_`.
/// The pipe can have any nonzero number of inputs and outputs for data, and they need not be symmetrical (eg. a pipe can take 2 input, but produce 3 outputs).
/// These signals should be specified with [`input`] and [`output`], respectively.
///
/// In addition to the data inputs/outputs, the pipe is assumed to propagate a `valid` control signal, which communicates whether or not there's valid data at each stage.
/// Like data inputs/outputs, this signal enters the pipe via an `in_valid` port, and leaves via an `out_valid` port.
/// This signal should **not** be specified with [`input`] / [`output`]; it's assumed to exist.
///
/// Optionally, the inner pipeline module may have auxiliary input/output ports that aren't necessarily related to the pipeline data flow.
/// These may be used to interact with other modules for some stages, or some other logic entirely.
/// The names of these inputs/outputs aren't expected to have any specific prefix.
/// These signals should be specified with [`aux_input`] and [`aux_output`], respectively.
/// No additional logic will be generated for these auxiliary signals - they will simply be forwarded from/to matching ports on the generated module.
///
/// # Generated interface
///
/// ## Ports
///
/// Assuming all inputs/outputs have been correctly specified, the generated module will have the same input/output ports (including auxiliary input/output ports) as the inner pipeline.
/// Four additional flow control signals will be added:
///  - `in_ready` - an output signalling that the pipeline can accept data on this cycle
///  - `in_valid` - an input signalling that valid data is presented to the pipeline inputs on this cycle
///  - `out_ready` - an input signalling that valid data presented from the pipeline outputs will be accepted on this cycle
///  - `out_valid` - an output signalling that valid data is presented from the pipeline outputs on this cycle
///
/// ## Handshake
///
/// Data is only accepted into the pipeline inputs for cycles where both `in_ready` and `in_valid` are high.
/// Likewise, data is only accepted from the pipeline outputs for cycles where both `out_ready` and `out_valid` are high.
///
/// ## Latency
///
/// The generated pipeline inserts some (unspecified) number of additional stages to facilitate flow control.
/// Because of this, the generated pipeline's latency will be strictly greater than the inner pipeline's latency.
/// If `out_ready` is always high, the pipeline will behave exactly as the input pipeline with some additional latency.
///
/// ## Throughput
///
/// The generated pipeline will be able to sustain the full throughput of the input pipeline as long as there aren't stalls or bubbles generated in response to the flow control signals.
///
/// [`input`]: Self::input
/// [`output`]: Self::output
/// [`aux_input`]: Self::aux_input
/// [`aux_output`]: Self::aux_output
pub struct FlowControlledPipe<'a> {
    c: &'a Context<'a>,
    /// The generated module.
    pub module: &'a Module<'a>,
    mod_name: String,
    pipe: &'a Instance<'a>,
    num_credit_stages: u32,
    num_credits_bit_width: u32,
    num_credits: &'a Register<'a>,
    in_handshake: &'a Signal<'a>,
    out_ready: &'a Signal<'a>,
    out_valid: &'a Signal<'a>,

    has_output: bool,
}

impl<'a> FlowControlledPipe<'a> {
    pub fn new<S: Into<String>>(c: &'a Context<'a>, mod_name: S, inner_pipe_mod_name: &str, num_inner_pipe_stages: u32) -> FlowControlledPipe<'a> {
        let mod_name = mod_name.into();

        let module = c.module(&mod_name);

        let num_credit_stages = num_inner_pipe_stages; // TODO: Make adjustable?
        let max_num_credits = num_inner_pipe_stages + 3 + num_credit_stages; // TODO: Document derivation

        let num_credits_bit_width = (max_num_credits as f64).log2().ceil() as u32 + 1; // TODO: Use helpers here?
        let num_credits = module.reg("num_credits", num_credits_bit_width);
        num_credits.default_value(max_num_credits);

        // Inner pipe
        let pipe = module.instance("pipe", inner_pipe_mod_name);

        // Ingress
        let in_ready = num_credits.value.ne(module.lit(0u32, num_credits_bit_width));
        module.output("in_ready", in_ready);

        let in_handshake = in_ready & module.input("in_valid", 1);
        pipe.drive_input("in_valid", in_handshake);

        let out_ready = module.input("out_ready", 1);
        let out_valid = pipe.output("out_valid");

        FlowControlledPipe {
            c,
            module,
            mod_name,
            pipe,
            num_credit_stages,
            num_credits_bit_width,
            num_credits,
            in_handshake,
            out_ready,
            out_valid,

            has_output: false,
        }
    }

    /// Specifies an input to the inner pipeline module.
    ///
    /// An input port will be added to the generated module that forwards data directly to the corresponding inner pipeline module's input.
    ///
    /// Note: `name` is expected **not** to be prefixed with `in_` - this prefix is added automatically.
    pub fn input<S: Into<String>>(&mut self, name: S, bit_width: u32) {
        let name = name.into();

        let name = format!("in_{}", name);
        let input = self.module.input(&name, bit_width);
        self.pipe.drive_input(name, input);
    }

    /// Specifies an output to the inner pipeline module.
    ///
    /// An output port will be added to the generated module that forwards data directly from the corresponding inner pipeline module's output.
    ///
    /// Note: `name` is expected **not** to be prefixed with `out_` - this prefix is added automatically.
    pub fn output<S: Into<String>>(&mut self, name: S, bit_width: u32) {
        let name = name.into();

        // FIFO
        let fifo_mod_name = format!("{}_{}_Fifo", self.mod_name, name);
        let fifo_depth_bit_width = self.num_credits_bit_width - 1;
        fifo::generate(self.c, &fifo_mod_name, fifo_depth_bit_width, bit_width);
        let fifo = self.module.instance(format!("{}_fifo", name), &fifo_mod_name);
        fifo.drive_input("write_enable", self.out_valid);
        fifo.drive_input("write_data", self.pipe.output(format!("out_{}", name)));

        // Peek buffer
        let peek_buffer_mod_name = format!("{}_{}_PeekBuffer", self.mod_name, name);
        peek_buffer::generate(self.c, &peek_buffer_mod_name, bit_width);
        let peek_buffer = self.module.instance(format!("{}_peek_buffer", name), &peek_buffer_mod_name);
        fifo.drive_input("read_enable", peek_buffer.output("ingress_read_enable"));
        peek_buffer.drive_input("ingress_data", fifo.output("read_data"));
        peek_buffer.drive_input(
            "ingress_data_valid",
            (!fifo.output("empty") & peek_buffer.output("ingress_read_enable"))
            .reg_next_with_default(format!("{}_peek_buffer_ingress_data_valid", name), false));

        // Egress
        self.module.output(format!("out_{}", name), peek_buffer.output("egress_data"));
        let out_valid = peek_buffer.output("egress_ready");
        peek_buffer.drive_input("egress_read_enable", self.out_ready);

        if !self.has_output {
            self.module.output("out_valid", out_valid);

            let out_handshake = self.out_ready & out_valid;

            // Credit return path
            let mut credit_inc = out_handshake;

            for i in 0..self.num_credit_stages {
                credit_inc = credit_inc.reg_next_with_default(format!("stage{}_credit_inc", i), false);
            }

            // Credit adjustment
            let credit_dec = self.in_handshake;
            let credit_adjust_bits = credit_inc.concat(credit_dec);
            self.num_credits.drive_next(if_(credit_adjust_bits.eq(self.module.lit(0b10u32, 2)), {
                self.num_credits.value + self.module.lit(1u32, self.num_credits_bit_width)
            }).else_if(credit_adjust_bits.eq(self.module.lit(0b01u32, 2)), {
                self.num_credits.value - self.module.lit(1u32, self.num_credits_bit_width)
            }).else_({
                self.num_credits.value
            }));

            self.has_output = true;
        }
    }

    /// Specifies an auxiliary input to the inner pipeline module.
    ///
    /// An input port will be added to the generated module that forwards data directly to the corresponding inner pipeline module's input.
    ///
    /// Note: `name` isn't expected to have any specific prefix, and no prefix will be added.
    pub fn aux_input<S: Into<String>>(&mut self, name: S, bit_width: u32) {
        let name = name.into();

        let input = self.module.input(&name, bit_width);
        self.pipe.drive_input(name, input);
    }

    /// Specifies an auxiliary output from the inner pipeline module.
    ///
    /// An output port will be added to the generated module that forwards data directly from the corresponding inner pipeline module's output.
    ///
    /// Note: `name` isn't expected to have any specific prefix, and no prefix will be added.
    pub fn aux_output<S: Into<String>>(&mut self, name: S) {
        let name = name.into();

        self.module.output(&name, self.pipe.output(&name));
    }
}
