extern crate serialport;

use serialport::prelude::*;

use std::env;
use std::io::Read;
use std::fs::File;

fn main() {
    let input_file_name = env::args().skip(1).nth(0).expect("Input file argument missing");

    let input = {
        let mut input = Vec::new();
        File::open(input_file_name).expect("Couldn't open input file").read_to_end(&mut input).expect("Couldn't read input file");
        input
    };
    let input_len = input.len();

    println!("Input len: 0x{:08x} bytes", input_len);

    if input.is_empty() {
        panic!("Input is empty");
    }

    if (input_len % 4) != 0 {
        panic!("Input len ({} bytes) is not divisible by 4", input_len);
    }

    if input_len >= 0x10000 {
        panic!("Input len ({} bytes) is too large", input_len);
    }

    let mut port = serialport::open("COM4").expect("Couldn't open serial port");
    port.set_baud_rate(BaudRate::Baud115200).expect("Couldn't set serial port baud rate");
    port.write_data_terminal_ready(true).expect("Couldn't set serial port DTR:");

    let input_len_bytes = [
        input_len as u8,
        (input_len >> 8) as u8,
        (input_len >> 16) as u8,
        (input_len >> 24) as u8,
    ];
    port.write_all(&input_len_bytes).expect("Couldn't write out length bytes");
    port.write_all(&input).expect("Couldn't write rom");
}
