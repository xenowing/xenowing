use std::env;
use std::io::{Read, Write};
use std::fs::File;

fn main() {
    let input_file_name = env::args().skip(1).nth(0).expect("Input file argument missing");
    let output_file_name = env::args().skip(1).nth(1).expect("Output file argument missing");

    let input = {
        let mut input = Vec::new();
        File::open(input_file_name).expect("Couldn't open input file").read_to_end(&mut input).expect("Couldn't read input file");
        input
    };

    if input.is_empty() {
        panic!("Input is empty");
    }

    if (input.len() % 4) != 0 {
        panic!("Input len ({} bytes) is not divisible by 4", input.len());
    }

    if input.len() >= 0x2000 {
        panic!("Input len ({} bytes) is too large", input.len());
    }

    let mut output = File::create(output_file_name).expect("Couldn't open output file");

    writeln!(output, "DEPTH = {};", input.len() / 4).unwrap();
    writeln!(output, "WIDTH = 32;").unwrap();
    writeln!(output, "ADDRESS_RADIX = HEX;").unwrap();
    writeln!(output, "DATA_RADIX = HEX;").unwrap();
    writeln!(output, "").unwrap();

    writeln!(output, "CONTENT BEGIN").unwrap();
    writeln!(output, "").unwrap();

    for i in 0..input.len() / 4 {
        writeln!(output, "{:04x}:{:02x}{:02x}{:02x}{:02x};", i, input[i * 4 + 3], input[i * 4 + 2], input[i * 4 + 1], input[i * 4]).unwrap();
    }

    writeln!(output, "").unwrap();
    writeln!(output, "END;").unwrap();
}
